use reqwest::blocking;
use crate::json::{GameVersion, PatchList, LauncherConfig, ConfigResponse, UpgradePath};
use std::sync::mpsc::Sender;
use std::thread;
use crate::files::{LoadingFileSource, LoadedFileSource};
use std::boxed::Box;
use std::io::Read;
use sha2::{Sha512, Digest};

fn platform() -> &'static str {
    #[cfg(windows)]
    { "win" }

    #[cfg(not(windows))]
    { "linux" }
}

/*
    .route("/v1/launcher/files/:cdn/:version", get(launcher_files)) // LAUNCHER AUTOUPDATE TODO
    .route("/v1/stats/:channel", get(stats_metadata)) // NOT USED BY LAUNCHER
    .route("/v1/route/:channel/:platform/:from/:from_channel", get(upgrade_path))
     */

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
    Ok(blocking::get(format!("{}/v1/paths/release/{cdn}/{channel}/{}/{name}", crate::defs::URL, platform()))?.text()?)
}

pub fn get_update_path(from_channel: &str, to_channel: &str, from_release: &str) -> Result<UpgradePath, anyhow::Error> {
    let res = blocking::get(format!("{}/v1/route/{to_channel}/{}/{from_release}/{from_channel}", crate::defs::URL, platform()));
    Ok(res.and_then(|x| x.json::<UpgradePath>())?)
}

pub fn download_file(url: &str, status: Option<std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>>)  -> Result<LoadedFileSource, anyhow::Error>{
    
    todo!();
}

/* 
pub fn get_file(url: &str, hash_url: Option<String>, status: Option<std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>>) -> Result<LoadedFileSource, anyhow::Error>{
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    let data = with_os_header(client.get(url)).send()?;

    let len = data.content_length();
    let mut reader = std::io::BufReader::with_capacity(64000, data);
    
    if let Some(s) = &status {
        let mut lock = s.lock().unwrap();
        println!("downloading {}", &url);
        lock.1 = format!("Downloading {}", &url);
    }

    println!("FILE OF SIZE {:?}", len);
    let mut save = LoadingFileSource::new(len)?;
    let mut buf = [0u8; 4096];
    let mut n = 0;

    let mut hasher = Sha512::new();
    loop {
        let k = reader.read(&mut buf)?;
        if k == 0 {
            break;
        }

        hasher.update(&buf[0..k]);
        save.add(&buf[0..k])?;

        n += k;
        if let Some(s) = &status {
            let mut lock = s.lock().unwrap();
            lock.0 = (n as f32) / (len.unwrap_or(n as u64) as f32);
        }
    }

    if let Some(path) = hash_url {
        let hash = with_os_header(client.get(&path)).send()?.text()?;
        let chash =  hex::encode(hasher.finalize());
        if chash.to_ascii_uppercase() != hash.to_ascii_uppercase() {
            return Err(anyhow::Error::msg(format!("Hashes for {} did not match", &url)));
        }
    }
    Ok(LoadedFileSource::new(save))
}
*/