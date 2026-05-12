use std::process::Command;

use crate::{event::FileEvent, registry::Registry};

struct Dispatcher {
    registry: Registry,
    exec: Option<String>
}

impl Dispatcher {
    pub fn new(registry: Registry, exec: Option<String>) -> Self {
        Dispatcher { registry, exec }
    }

    pub fn dispatch(&self, event: &FileEvent){
        let ext = event.path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        for plugin in self.registry.find_by_extension(ext) {
            plugin.on_event(event);
        }

        if let Some(cmd) = &self.exec {
            Command::new("sh")
                     .arg("-c")
                     .arg(cmd)
                     .spawn()
                     .ok(); // non-blocking: start and move on
        }
    }
}