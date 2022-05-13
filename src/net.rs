use reqwest::blocking;
use crate::json::{LauncherConfiguration, GameVersion, PatchList};
use std::sync::mpsc::Sender;
use std::thread;
use crate::files::{LoadingFileSource, LoadedFileSource};
use std::boxed::Box;
use std::io::Read;
use sha2::{Sha512, Digest};

fn os_header() -> &'static str {
    #[cfg(windows)]
    { "windows" }

    #[cfg(not(windows))]
    { "linux" }
}

pub fn with_os_header(req: reqwest::blocking::RequestBuilder) -> reqwest::blocking::RequestBuilder {
    req.header("X-Operating-System", os_header())
}

pub fn hash_url(url: &str) -> String {
    let re = regex::Regex::new(r"^(.*)\?(.)$").unwrap();
    if url.contains("?") {
        let caps = re.captures_iter(url).next().unwrap();
        format!("{}/hash?{}", &caps[0], &caps[1])
    } else {
        format!("{}/hash", url)
    }
}

pub fn get_config(send: Sender<Result<LauncherConfiguration, anyhow::Error>>) {
    thread::spawn(move || {
        let res = blocking::get(format!("{}/launcher/config", crate::defs::URL));
        let res = res.and_then(|x| x.json::<LauncherConfiguration>());
        send.send(res.map_err(|x|x.into())).unwrap();
    });
}

pub fn get_args(dev_enabled: bool) -> Result<String, anyhow::Error> {
    let args = reqwest::blocking::get(format!("{}/launcher/args?dev={}", crate::defs::URL, dev_enabled))?;
    Ok(args.text()?)
}

pub fn get_update_path(current: &crate::json::InstallManifest, dev_enabled: bool) -> Result<PatchList, anyhow::Error> {
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    let data = with_os_header(client.get(format!("{}/game/update?version={}&dev={}", crate::defs::URL, crate::defs::version_str(&current.version), dev_enabled)))
        .send()?.text()?;
    let path = serde_json::from_str::<PatchList>(&data)?;
    Ok(path)
}

pub fn get_current(dev_enabled: bool) -> Result<GameVersion, anyhow::Error> {
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    let data = with_os_header(client.get(format!("{}/game/current?dev={}", crate::defs::URL, dev_enabled)))
        .send()?.text()?;
    let version = serde_json::from_str::<GameVersion>(&data)?;
    Ok(version)
}

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

fn redownload_internal() -> Result<(), anyhow::Error> {
    let url = format!("{}/launcher/build", crate::defs::URL);
    let mut data = Vec::new();
    get_file(&url, Some(hash_url(&url)), None)?.as_reader().read_to_end(&mut data)?;

    let curr_name = std::env::current_exe()?;
    let mut new_name = curr_name.clone();
    new_name.pop();
    let mut nn = curr_name.components().last().unwrap().as_os_str().to_os_string();
    nn.push(".tmp");
    new_name.push(nn);

    std::fs::rename(&curr_name, new_name)?;
    std::fs::write(curr_name, data)?;
    Ok(())
}

pub fn redownload(send: Sender<Result<(), anyhow::Error>>) {
    thread::spawn(move || {//"ProcelioLauncher.exe"
        send.send(redownload_internal()).unwrap();
    });
}
