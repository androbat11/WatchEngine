// Config — Group 3 of the roadmap
use std::path::{PathBuf};
pub struct Config {
    pub root: PathBuf,
    pub debounce_ms: u64,
    pub plugins: Vec<String>
}
