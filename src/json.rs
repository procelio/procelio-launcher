
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Update {
    pub title: String,
    pub version: Vec<i32>,
    pub dev: bool,
    pub description: String,
    pub hyperlink: String,
    pub image: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LauncherConfiguration {
    #[serde(alias = "websiteUrl")]
    pub website_url: String,
    #[serde(default, alias = "launcherArguments")]
    pub launcher_arguments: Vec<String>,
    pub updates: Vec<Update>,
    #[serde(alias = "launcherVersion")]
    pub launcher_version: Vec<i32>,
    #[serde(alias = "quoteOfTheDay")]
    pub quote_of_the_day: String,
    #[serde(alias = "quoteAuthor")]
    pub quote_author: String
}

pub enum LauncherConfigStatus {
    AppLoad,
    Pending(std::sync::mpsc::Receiver<Result<LauncherConfiguration, reqwest::Error>>),
    Loaded(LauncherConfiguration)
}