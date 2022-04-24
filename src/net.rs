use reqwest::blocking;
use crate::json::LauncherConfiguration;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

pub fn get_config(send: Sender<Result<LauncherConfiguration, reqwest::Error>>) {
    thread::spawn(move || {
        let res = blocking::get("https://files.procelio.com:8677/launcher/config");
        let res = res.and_then(|x| x.json::<LauncherConfiguration>());
        send.send(res).unwrap();
    });
}
