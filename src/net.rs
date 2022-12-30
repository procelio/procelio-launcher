use reqwest::blocking;
use crate::json::{LauncherConfig, ConfigResponse, UpgradePath};
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
    Ok(blocking::get(format!("{}/v1/paths/release/{cdn}/{channel}/{}/{name}", crate::defs::URL, platform()))?.text()?)
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

pub fn download_file(url: &str, status: Option<std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>>)  -> Result<LoadedFileSource, anyhow::Error>{
    if let Some(s) = status {
        for i in 0..=100 {
            let q = (i as f32 / 100.0, format!("{i}/100"), None);
            *s.lock().unwrap() = q;
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
    
    println!("DOWNLOADING {:?}", url);
    let data = std::fs::read(url.replace("/", "\\"  ));

    Ok(LoadedFileSource::InMemory(data?))
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

    let src = match download_file(&url, None) {
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
