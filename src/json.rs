
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Update {
    pub title: String,
    pub version: Vec<i32>,
    pub dev: bool,
    pub description: String,
    pub hyperlink: String,
    pub image: Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LauncherConfiguration {
    pub websiteUrl: String,
    #[serde(default)]
    pub launcherArguments: Vec<String>,
    pub updates: Vec<Update>,
    pub launcherVersion: Vec<i32>,
    pub quoteOfTheDay: String,
    pub quoteAuthor: String
}

pub enum LauncherConfigStatus {
    AppLoad,
    Pending(std::sync::mpsc::Receiver<Result<LauncherConfiguration, reqwest::Error>>),
    Loaded(LauncherConfiguration)
}