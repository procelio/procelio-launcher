pub const CURRENT_README: i32 = 4534;
pub const LICENSE: &str = include_str!("resources/licenses.txt");
pub const URL: &str = "http://localhost:9630";//"https://files.procelio.com:8677";

pub fn version() -> &'static str {
    "1.0.0"
}

pub fn launcher_name() -> &'static str {
    "procelio_launcher_bin.exe"
}