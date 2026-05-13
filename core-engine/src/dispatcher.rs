use std::process::Command;
use std::time::Duration;

use crate::{debouncer::Debouncer, event::{EventHandler, FileEvent}, registry::Registry};

pub struct Dispatcher {
    registry: Registry,
    exec: Option<String>,
    debouncer: Debouncer,
}

impl Dispatcher {
    pub fn new(registry: Registry, exec: Option<String>, debounce_ms: u64) -> Self {
        Dispatcher {
            registry,
            exec,
            debouncer: Debouncer::new(Duration::from_millis(debounce_ms)),
        }
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

impl EventHandler for Dispatcher {
    fn handle(&mut self, event: &FileEvent) {
        if self.debouncer.is_time_out(&event.path) {
            self.debouncer.record(event.path.clone());
            self.dispatch(event);
        }
    }
}