// JavaScriptPlugin — Group 14 of the roadmap
use std::{collections::HashSet, path::{Path, PathBuf}};
use watch_core::{event::FileEvent, plugin::WatchPlugin};
// Repeated code. 
// Find a way to create a general implementation
pub struct JavascriptPlugin {
    root: PathBuf
}

impl JavascriptPlugin {
    pub fn new(root: PathBuf) -> Self {
        JavascriptPlugin { root }
    }
}

impl WatchPlugin for JavascriptPlugin {
    fn name(&self) -> &str { "javascript"}
    fn extensions(&self) -> &'static [&'static str] {
        &["js", "jsx", "mjs", "cjs"]
    }

    fn setup(&mut self) -> Vec<PathBuf> {
        let mut dirs = HashSet::new();

        let new_directory = walkdir::WalkDir::new(&self.root);
        for entry in new_directory {
            let Ok(entry) = entry else { continue };
            let path = entry.path();

            if is_javascript(path){
                if let Some(parent) = path.parent() {
                    dirs.insert(parent.to_path_buf());
                }
            }
        }
        dirs.into_iter().collect()
    }

    fn on_event(&self, event: &FileEvent) {
        println!("[javascript] {}", event.path.display());
    }
}

fn is_javascript(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("js") | Some("jsx")
    )
}