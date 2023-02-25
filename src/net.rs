use reqwest::blocking;
use crate::json::{LauncherConfig, ConfigResponse, UpgradePath};
use std::io::BufReader;
use std::io::Write;
use std::sync::mpsc::Sender;
use std::thread;
use crate::files::LoadedFileSource;
use std::boxed::Box;
use std::io::Read;

fn platform() -> &'static str {
    #[cfg(windows)]
    { "win" }

    #[cfg(not(windows))]
    { "linux" }
}

pub fn get_config(send: Sender<Result<LauncherConfig, anyhow::Error>>) {
    thread::spawn(move || {
        let res = blocking::get(format!("{}/v1/launcher/config", crate::defs::URL));
        let res = res.and_then(|x| x.json::<LauncherConfig>());
        send.send(res.map_err(|x|x.into())).unwrap();
    });
}

pub fn get_data(channel: String, send: Sender<Result<ConfigResponse, anyhow::Error>>) {
    thread::spawn(move || {
        let res = blocking::get(format!("{}/v1/launcher/config/{channel}/{}", crate::defs::URL, platform()));
        println!("Data: {:?}", res);
        let res = res.and_then(|x| x.json::<ConfigResponse>());
        send.send(res.map_err(|x|x.into())).unwrap();
    });
}

pub fn get_image(curr: String, image: String, send: Sender<Result<Vec<u8>, anyhow::Error>>) {
    thread::spawn(move || {
        let data = load_image(curr, image);
        let data = match data {
            Some(s) => Ok(s),
            None => Err(anyhow::anyhow!("missing image"))
        };
        send.send(data).unwrap();
    });
}

pub fn get_latest_build(channel: &str) -> Result<String, anyhow::Error> {
    Ok(blocking::get(format!("{}/v1/latest/{channel}/{}", crate::defs::URL, platform()))?.text()?)
}

pub fn get_stat_url(cdn: &str, channel: &str) -> Result<String, anyhow::Error> {
    Ok(blocking::get(format!("{}/v1/paths/stats/{cdn}/{channel}", crate::defs::URL))?.text()?)
}

pub fn get_release_url(cdn: &str, channel: &str, name: &str) -> Result<String, anyhow::Error> {
    Ok(blocking::get(format!("{}/v1/paths/release/{cdn}/{channel}/{}/{name}", crate::defs::URL, platform()))?.text()?)
}

pub fn get_patch_url(cdn: &str, channel: &str, name: &str) -> Result<String, anyhow::Error> {
    Ok(blocking::get(format!("{}/v1/paths/patch/{cdn}/{channel}/{}/{name}", crate::defs::URL, platform()))?.text()?)
}

pub fn get_image_url(cdn: &str, image: &str) -> Result<String, anyhow::Error> {
    Ok(blocking::get(format!("{}/v1/paths/image/{cdn}/{image}", crate::defs::URL))?.text()?)
}

pub fn get_update_path(from_channel: &str, to_channel: &str, from_release: &str) -> Result<UpgradePath, anyhow::Error> {
    let res = blocking::get(format!("{}/v1/route/{to_channel}/{}/{from_release}/{from_channel}", crate::defs::URL, platform()));
    Ok(res.and_then(|x| x.json::<UpgradePath>())?)
}

pub fn get_launcher_url(cdn: &str, name: &str) -> Result<String, anyhow::Error> {
    Ok(blocking::get(format!("{}/v1/paths/launcher/{cdn}/{name}", crate::defs::URL))?.text()?)
}

fn download_to_buffer<T: Write>(size: usize, mut read: BufReader<reqwest::blocking::Response>, write: T, status: Option<std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>>) -> Result<(), anyhow::Error>{
    let mut writer = std::io::BufWriter::new(write);

    let iter = (size / 1000) as u64;

    let mut buf = vec![0; 8192];

    let mut k = 0;
    let mut m = 0;
    loop {
        let n = read.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[0..n])?;
        let n = n as u64;
        k += n;
        m += n;
        
        if k > iter {
            k -= iter;
            if let Some(s) = &status {
                let mut lock = s.lock().unwrap();
                lock.0 = m as f32 / size as f32;
            }
        }
    }

    Ok(())
}


pub fn download_file(exp_size: Option<u64>, url: &str, status: Option<std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>>)  -> Result<LoadedFileSource, anyhow::Error>{
    if let Some(s) = &status {
        let mut lock = s.lock().unwrap();
        // https://host/bucket/file/name.name?awspresign
        let reg = regex::Regex::new("^.*/([^/\\?]*)\\?")?;
        if let Some(ss) = reg.captures(url) {
            lock.1 = format!("Downloading file {}", ss.get(1).unwrap().as_str());
        }
    }

    let mut resp = blocking::get(url)?;
    let exp_size = exp_size.or(resp.content_length());

    if let None = exp_size {
        let file = tempfile::tempfile()?;
        let mut writer = std::io::BufWriter::new(file.try_clone()?);
        resp.copy_to(&mut writer)?;
        return Ok(LoadedFileSource::OnDisk(file));
    }

    let reader = std::io::BufReader::new(resp);

    let size = exp_size.unwrap();
    if size < 512_000_000 {
        let mut buf = vec![0u8; size as usize];
        let cs = std::io::Cursor::new(&mut buf);
        download_to_buffer(size as usize, reader, cs, status)?;
        return Ok(LoadedFileSource::InMemory(buf));
    }

    let file = tempfile::tempfile()?;
    let f2 = file.try_clone()?;
    download_to_buffer(size as usize, reader, f2, status)?;
    Ok(LoadedFileSource::OnDisk(file))
}

pub fn load_image(curr_name: String, image_name: String) -> Option<Vec<u8>> {
    let mut path = match platform_dirs::AppDirs::new(Some("Procelio Launcher"), true) {
        None => { return None; }
        Some(s) => s.config_dir
    };

    path.push("bg.png");

    let data = std::fs::read(&path).ok();
    if data.is_some() && curr_name == image_name {
        return Some(data.unwrap());
    }

    let url = match get_image_url("nyc3", &image_name) {
        Ok(a) => a,
        Err(_) => { return None; }
    };

    let src = match download_file(None, &url, None) {
        Ok(s) => s,
        Err(_) => { return None; }
    };

    let bytes = match src {
        LoadedFileSource::InMemory(v) => v,
        LoadedFileSource::OnDisk(mut f) => {
            let mut b = Vec::new();
            if let Err(_) = f.read_to_end(&mut b) {
                return None;
            }
            b
        }
    };

    let _ = std::fs::write(path, bytes.as_slice());
    Some(bytes)
}
