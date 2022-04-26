pub const CURRENT_README: i32 = 4534;
pub const LICENSE: &str = include_str!("resources/licenses.txt");
pub const URL: &str = "https://files.procelio.com:8677";
pub fn version() -> Vec<i32> {
    vec!(0, 3, 7)//1, 0, 1)
}

pub fn version_str(version: &[i32]) -> String {
    version.iter().map(|x|x.to_string()).collect::<Vec<String>>().join(".")
}