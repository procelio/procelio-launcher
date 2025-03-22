pub const CURRENT_README: i32 = 4534;
pub const LICENSE: &str = include_str!("resources/licenses.txt");
pub const URL: &str = "https://releases.procelio.com:9630";
pub const KILLSWITCHURL: &str = "https://releases.procelio.com:9987";

pub const KILLSWITCHCERT: &[u8] =
"-----BEGIN CERTIFICATE-----
MIIB6zCCAXGgAwIBAgIUK5JfMIkLopp2OlqwoemLRRuAhUcwCgYIKoZIzj0EAwIw
FzEVMBMGA1UEAwwMcHJvY2VsaW8uY29tMCAXDTI1MDEwODAzMTAzOFoYDzIxMjQx
MjE1MDMxMDM4WjAXMRUwEwYDVQQDDAxwcm9jZWxpby5jb20wdjAQBgcqhkjOPQIB
BgUrgQQAIgNiAASHzA7d4pj/Ga8LvBntXJZUytfhHjxcjAgvnD43uv52+54S+Vk8
PDKfcEUAHFLr4jcnqgYvjJCeMc5qlGHjiIWGz/unbokXlPaj4I9XvVNHzmRCJDkf
sGPVhvUkEuX47C6jfDB6MB0GA1UdDgQWBBTKJhIMLUHzOTWoZROtsz0O2LDfjTAf
BgNVHSMEGDAWgBTKJhIMLUHzOTWoZROtsz0O2LDfjTAPBgNVHRMBAf8EBTADAQH/
MCcGA1UdEQQgMB6CDHByb2NlbGlvLmNvbYIOKi5wcm9jZWxpby5jb20wCgYIKoZI
zj0EAwIDaAAwZQIxAJTuNg5IAYuSmEVc9j3gweeopM3Uot/RvqwV6n7EumuICQcR
r2a2tigPMS9ZuY1iAAIwMzSJd+YTnsjDqQiuUu+iEPvKi17e+YdPK2ySiCWS0iPK
b4GNriZpvQjaVhYT6hMi
-----END CERTIFICATE-----".as_bytes(); // GOOD CERT

pub fn version() -> &'static str {
    "1.1.2"
}

pub fn launcher_name() -> &'static str {
    "procelio_launcher.exe"
}