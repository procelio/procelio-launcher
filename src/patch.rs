use std::io::BufRead;
use std::thread;
use std::boxed::Box;
use crate::json::InstallManifest;
use std::io::Seek;

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
    manifest.map(|x|Some(x)).map_err(|x|x.into())
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

fn download_fresh(dir: &std::path::PathBuf, dev: bool, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>) -> Result<Option<InstallManifest>, anyhow::Error> {
    let version = crate::net::get_current(dev)?;
    println!("DONWLOADING FRESH");
    let fresh_url = format!("{}/game/build/{}", crate::defs::URL, version.to_string());
    let file = crate::net::get_file(&fresh_url, Some(crate::net::hash_url(&fresh_url)), Some(process.clone()))?;
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

fn apply_patch(dir: &std::path::PathBuf, patch: String, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>) -> Result<Option<InstallManifest>, anyhow::Error> {
    let fresh_url = format!("{}/game/patch/{}", crate::defs::URL, patch);
    let file = crate::net::get_file(&fresh_url, Some(crate::net::hash_url(&fresh_url)), Some(process.clone()))?;
    patch_to(dir.to_owned(), file.as_reader(), Some(&|a, b| {
        let mut lock = process.lock().unwrap();
        lock.0 = a;
        lock.1 = format!("Patch {}: {}", patch, b);
    }))?;

    Ok(get_installed_version(dir)?)
}

fn launch_game(manifest: Option<InstallManifest>, dir: std::path::PathBuf) -> Result<(), anyhow::Error> {
    let manifest = match manifest {
        Some(s) => s,
        None => { return Err(anyhow::Error::msg("Unable to load launch manifest")); }
    };
    
    let args = crate::net::get_args(manifest.dev)?;
    std::process::Command::new(dir.join(manifest.exec))
        .args(shell_words::split(&args)?)
        .spawn()?;
    println!("Launch Game");
    Ok(())
}


pub fn play_clicked_internal(dir: std::path::PathBuf, use_dev: bool, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>) -> Result<(), anyhow::Error> {
    proceliotool::tools::patch::check_rollback(&dir)?;
    let mut installed_version = match get_installed_version(&dir)? {
        Some(s) => s,
        None => {
           return launch_game(download_fresh(&dir, use_dev, process)?, dir);
        }
    };

    let path = crate::net::get_update_path(&installed_version, use_dev)?;
    if path.most_recent == installed_version {
        return launch_game(Some(installed_version), dir);
    }

    if path.patches.is_empty() {
        uninstall_internal(&dir, process.clone())?;
        return launch_game(download_fresh(&dir, use_dev, process)?, dir);
    }

    for patch in &path.patches {
        if let Some(s) = apply_patch(&dir, patch.to_owned(), process.clone())? {
            installed_version = s;
        }
    }
    launch_game(Some(installed_version), dir)?;
    Ok(())
}

pub fn play_clicked(
    dir: std::path::PathBuf,
    use_dev: bool, 
    process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>,
    send: std::sync::mpsc::Sender<Result<(), anyhow::Error>>
) {
    thread::spawn(move || {
        if let Err(e) = play_clicked_internal(dir, use_dev, process.clone()) {
            process.lock().unwrap().2 = Some(Box::new(e));
        } 
        send.send(Ok(())).unwrap();
    });
}



fn uninstall_internal(dir: &std::path::PathBuf, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>) -> Result<(), anyhow::Error> {
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
            std::fs::remove_dir(f.path())?;
        }
        else if f.path().is_file() {
            std::fs::remove_file(f.path())?;
        }
    }
    Ok(())
}

pub fn uninstall(dir: std::path::PathBuf, process: std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>, send: std::sync::mpsc::Sender<Result<(), anyhow::Error>>) {
    thread::spawn(move || {
        if let Err(e) = uninstall_internal(&dir, process.clone()) {
            process.lock().unwrap().2 = Some(Box::new(e));
        }
        send.send(Ok(())).unwrap();
    });
}
