
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

pub enum LoadStatus<T> {
    AppLoad,
    AwaitingApproval,
    Pending(std::sync::mpsc::Receiver<Result<T, anyhow::Error>>),
    Loaded(T)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct GameVersion {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    #[serde(alias = "dev_build")]
    pub dev: bool
}

impl ToString for GameVersion {
    fn to_string(&self) -> std::string::String { 
        format!("{}.{}.{}{}", self.major, self.minor, self.patch, if self.dev { "dev" } else { "" })
    }
}

impl GameVersion {
    pub fn new(major: i32, minor: i32, patch: i32, dev: bool) -> GameVersion {
        GameVersion {
            major,
            minor,
            patch,
            dev
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatchList {
    #[serde(alias = "update_to")]
    pub most_recent: GameVersion,
    pub patches: Vec<String>
}

#[derive(Serialize, Deserialize)]
pub struct InstallManifest {
    pub exec: String,
    pub version: Vec<i32>,
    pub dev: bool,
}

impl std::cmp::PartialEq<InstallManifest> for GameVersion {
    fn eq(&self, other: &InstallManifest) -> bool { 
        self.dev == other.dev &&
        self.major == other.version[0] &&
        self.minor == other.version[1] &&
        self.patch == other.version[2]
    }
}

#[test]
fn test_game_version_ordering() {
    let g1 = GameVersion::new(1, 1, 3, true);
    let g1_2 = GameVersion::new(1, 1, 3, false);
    let g2 = GameVersion::new(1, 2, 0, false);
    let g3 = GameVersion::new(2, 0, 0, false);
    let g4 = GameVersion::new(0, 0, 5, false);
    let g5 = GameVersion::new(1, 1, 5, false);

    assert!(g1 < g2);
    assert!(g1 < g3);
    assert!(g1 > g4);
    assert!(g1 < g5);
    assert!(g1 != g1_2);
    assert!(g1.to_string() == "1.1.3dev")
}