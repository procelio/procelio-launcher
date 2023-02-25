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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Release {
    pub channel: String,
    pub platform: String,
    pub name: String,
    pub download_size: u64,
    pub title: String,
    pub description: String,
    pub changelog: String
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    pub download_size: u64,
    pub platform: String,
    pub from_channel: String,
    pub to_channel: String,
    pub from_name: String,
    pub to_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UpgradePath {
    NoChangesRequired,
    FreshDownload(Release),
    PatchRoute(Vec<Patch>)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LauncherMetadata {
    pub version: String,
    pub website_url: String,
    pub message_of_the_day: String,
    pub motd_author: String,
    pub bg_image: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub channels: Vec<String>,
    pub metadata: LauncherMetadata,
    pub cdn_regions: Vec<String>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangelogElement {
    pub title: String,
    pub description: String,
    pub hyperlink: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub newest_release_name: String,
    pub args: Vec<String>,
    pub changelog: Vec<ChangelogElement>
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

#[derive(Serialize, Deserialize, Clone)]
pub struct InstallManifest {
    pub exec: String,
    pub version: String,
    pub channel: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OldInstallManifest {
    pub exec: String,
    pub dev: bool,
    pub version: Vec<u32>
}

impl From<OldInstallManifest> for InstallManifest {
    fn from(item: OldInstallManifest) -> Self {
        InstallManifest { 
            exec: item.exec,
            version: item.version.iter().map(|x|x.to_string()).collect::<Vec<String>>().join("."),
            channel: (if item.dev { "dev" } else { "prod"}).to_owned()
        }
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