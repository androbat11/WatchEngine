// TypeScriptPlugin — Group 13 of the roadmap
use std::{collections::HashSet, path::{Path, PathBuf}};
use watch_core::{event::FileEvent, plugin::WatchPlugin};

pub struct TypescriptPlugin {
    root: PathBuf,
}

impl TypescriptPlugin {
    pub fn new(root: PathBuf) -> Self {
        TypescriptPlugin { root }
    }
}

impl WatchPlugin for TypescriptPlugin {
    fn name(&self) -> &str { "typescript" }

    fn extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx"]
    }

    // * Make sure which directories the watcher is supposed to watch
    // * It runs once at start up
    fn setup(&mut self) -> Vec<PathBuf> {
        let mut dirs = HashSet::new();

        // * WalkDir creates a new directory and returns the file
        //   names modified within 24h
        let new_directory = walkdir::WalkDir::new(&self.root);
        for entry in new_directory {
            let Ok(entry) = entry else { continue };
            let path = entry.path();

            if is_typescript(path) {
                if let Some(parent) = path.parent() {
                    dirs.insert(parent.to_path_buf());
                }
            }
        }

        dirs.into_iter().collect()
    }

    fn on_event(&self, event: &FileEvent) {
        println!("[typescript] {}", event.path.display());
    }
}

fn is_typescript(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts") | Some("tsx")
    )
}