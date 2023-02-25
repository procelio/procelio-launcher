use std::io::BufRead;
use std::thread;
use std::boxed::Box;
use crate::json::{InstallManifest, OldInstallManifest};
use std::io::Seek;

#[derive(Clone)]
pub struct PlayGameConfig {
    pub cdn: String,
    pub channel: String,
    pub latest_build: String,
    pub args: Vec<String>,
}

pub fn delete_old_launcher() -> Result<(), anyhow::Error> {
    let curr_name = std::env::current_exe()?;
    let mut new_name = curr_name.clone();
    new_name.pop();
    let mut nn = curr_name.components().last().unwrap().as_os_str().to_os_string();
    nn.push(".tmp");
    new_name.push(nn);
    if new_name.is_file() {
        std::fs::remove_file(&new_name).unwrap();
    }
    Ok(())
}

fn get_installed_version(install_dir: &std::path::PathBuf) -> Result<Option<InstallManifest>, anyhow::Error> {
    let mut path = install_dir.to_owned();
    path.push("manifest.json");
    if !path.is_file() {
        return Ok(None);
    }
    let data = std::fs::read(path)?;
    let manifest: Result<InstallManifest, serde_json::Error> = serde_json::from_slice(&data);
    let old_manifest: Result<OldInstallManifest, serde_json::Error> = serde_json::from_slice(&data);

    manifest.or(old_manifest.map(|x|x.into())).map(|x|Some(x)).map_err(|x|x.into())
}

fn unzip_to<T: Seek + BufRead>(dir: std::path::PathBuf, reader: T, cb: Option<&dyn Fn(f32, String)>) -> Result<(), anyhow::Error>{
    let mut strm = zip::ZipArchive::new(reader)?;

    let len = strm.len();
    println!("unzip for {}", len);
    for i in 0..len {
        let mut file = strm.by_index(i)?;
        println!("Unzip {:?}", &file.enclosed_name());

        let name = match file.enclosed_name() {
            Some(a) => {
                a
            }
            None => { continue; }
        };
        

        if let Some(s) = cb {
            s((i as f32) / (len as f32), format!("Extracting {} ({})", name.display(), file.size()));
        }

        let outpath = dir.join(name.components());
        if file.is_dir() {
            std::fs::create_dir_all(outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut ondisk = std::fs::File::create(outpath)?;
            std::io::copy(&mut file, &mut ondisk)?;
        }
    }

    Ok(())
}

fn download_fresh(config: PlayGameConfig, dir: &std::path::PathBuf, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>) -> Result<Option<InstallManifest>, anyhow::Error> {
    let path = crate::net::get_release_url(&config.cdn, &config.channel, &config.latest_build)?;
    let file = crate::net::download_file(None, &path, Some(process.clone()))?;

    println!("File downloaded");
    unzip_to(dir.to_owned(), file.as_reader(), Some(&|a, b| {
        let mut lock = process.lock().unwrap();
        lock.0 = a;
        lock.1 = b;
    }))?;
    Ok(get_installed_version(dir)?)
}

fn patch_to<T: Seek + BufRead>(dir: std::path::PathBuf, reader: T, cb: Option<&dyn Fn(f32, String)>) -> Result<(), anyhow::Error> {
    let mut zip = zip::read::ZipArchive::new(reader)?;
    proceliotool::tools::patch::from_zip(dir.to_owned(),&mut zip, cb)
}

fn apply_patch(config: PlayGameConfig, dir: &std::path::PathBuf, patch: String, size: u64, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>) -> Result<Option<InstallManifest>, anyhow::Error> {
    let path = crate::net::get_patch_url(&config.cdn, &config.channel, &patch)?;
    println!("Download patch {:?}", &path);
    let file = crate::net::download_file(Some(size), &path, Some(process.clone()))?;

    let dd = patch_to(dir.to_owned(), file.as_reader(), Some(&|a, b| {
        let mut lock = process.lock().unwrap();
        lock.0 = a;
        lock.1 = format!("Patch {}: {}", patch, b);
    }));
    println!("{:?}", dd);
    let _ = dd?;
    Ok(get_installed_version(dir)?)
}

fn launch_game(config: PlayGameConfig, manifest: Option<InstallManifest>, dir: std::path::PathBuf, version_send: std::sync::mpsc::Sender<Result<InstallManifest, anyhow::Error>>) -> Result<Option<std::process::Child>, anyhow::Error> {
    let manifest = match manifest {
        Some(s) => s,
        None => { return Err(anyhow::Error::msg("Unable to load launch manifest")); }
    };
    println!("Launch Game");
    version_send.send(Ok(manifest.clone()))?;
    thread::spawn(move || {
        std::process::Command::new(dir.join(manifest.exec))
        .args(config.args)
        .output().unwrap();
    });
    Ok(None)
}


pub fn play_clicked_internal(
    dir: std::path::PathBuf,
    config: PlayGameConfig,
    process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>,
    version_send: std::sync::mpsc::Sender<Result<InstallManifest, anyhow::Error>>) -> Result<Option<std::process::Child>, anyhow::Error> {
    proceliotool::tools::patch::check_rollback(&dir)?;

    let installed_version = match get_installed_version(&dir)? {
        Some(s) => s,
        None => {
           return launch_game(config.clone(), download_fresh(config.clone(), &dir, process)?, dir, version_send);
        }
    };

    let path = crate::net::get_update_path(&installed_version.channel, &config.channel, &installed_version.version)?;

    let manifest = match path {
        crate::json::UpgradePath::NoChangesRequired => Some(installed_version),
        crate::json::UpgradePath::FreshDownload(d) => {
            uninstall_internal(&dir, process.clone())?;
            println!("{:?}", &d);
            download_fresh(config.clone(), &dir, process)?
        },
        crate::json::UpgradePath::PatchRoute(pr) => {
            let mut m = None;
            for p in pr {
                m = apply_patch(config.clone(), &dir, p.name, p.download_size, process.clone())?;
            }
            m
        },
    };

    launch_game(config, manifest, dir, version_send)
}


pub fn play_clicked(
    dir: std::path::PathBuf,
    config: PlayGameConfig,
    process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>,
    send: std::sync::mpsc::Sender<Result<(), anyhow::Error>>,
    version_send: std::sync::mpsc::Sender<Result<InstallManifest, anyhow::Error>>
) {
    thread::spawn(move || {
        let res = play_clicked_internal(dir, config, process.clone(), version_send);
        match res {
            Err(e) => {
                send.send(Err(e)).unwrap();
            }
            Ok(c) => {
                if let Some(mut z) = c {
                    z.wait().unwrap();
                }
                send.send(Ok(())).unwrap();
            }
        }
    });
}



fn uninstall_internal(dir: &std::path::PathBuf, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>) -> Result<(), anyhow::Error> {
    if !dir.ends_with("Procelio") {
        let msg = format!("Cannot guarantee path '{:?}' only contains Procelio files. Please delete manually.", dir.display());
        return Err(anyhow::anyhow!(msg));
    }

    let version = get_installed_version(&dir)?;
    if let None = version {
        return Ok(());
    }

    let num = std::fs::read_dir(dir)?.count();
    let mut i = 0;
    for file in std::fs::read_dir(dir)? {
        let f = file?;
        let mut lock = process.lock().unwrap();
        lock.0 = (i as f32) / (num as f32);
        lock.1 = format!("Removing {}", f.path().display());
        i += 1;
        drop(lock);
        if f.path().is_dir() {
            std::fs::remove_dir_all(f.path())?;
        }
        else if f.path().is_file() {
            std::fs::remove_file(f.path())?;
        }
    }
    Ok(())
}

pub fn uninstall(dir: std::path::PathBuf, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>, send: std::sync::mpsc::Sender<Result<(), anyhow::Error>>) {
    println!("Invoke uninstall");
    thread::spawn(move || {
        if let Err(e) = uninstall_internal(&dir, process.clone()) {
            send.send(Err(anyhow::anyhow!(format!("Uninstallation failed: {:?}", &e)))).unwrap();
            process.lock().unwrap().2 = Some(Box::new(e));
            return;
        }
        println!("Uninstall ok");
        send.send(Ok(())).unwrap();
    });
}
