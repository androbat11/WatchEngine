// WatchPlugin trait — Group 4 of the roadmap



use crate::event::FileEvent;
use std::path::PathBuf;


pub struct EchoPlugin {
    // The name is the name of the language that 
    // will be configured.
    name: String,
    extensions: &'static[&'static str]
}

impl EchoPlugin {
    pub fn new(extensions: &'static [&'static str], name: String) -> Self {
        EchoPlugin { name, extensions }
    }
}

pub trait WatchPlugin {
    fn name(&self) -> &str;
    fn extensions(&self) -> &'static [&'static str];
    fn setup(&mut self) -> Vec<PathBuf>;
    fn on_event(&self, event: &FileEvent);
}

impl WatchPlugin for EchoPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn extensions(&self) -> &'static [&'static str]{
        self.extensions
    }
    fn setup(&mut self) -> Vec<PathBuf> {
        Vec::new()
    }
    fn on_event(&self, event: &FileEvent) {
        println!("[{}] event for {:?}", self.name, event.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::FileEventKind;

    #[test]
    fn echo_plugin_exposes_metadata() {
        let plugin = EchoPlugin::new(&[".ts", ".tsx"], "typescript".to_string());
        assert_eq!(plugin.name(), "typescript");
        assert_eq!(plugin.extensions(), &[".ts", ".tsx"]);
    }

    #[test]
    fn echo_plugin_handles_event_without_panicking() {
        let plugin = EchoPlugin::new(&[".rs"], "echo".to_string());
        let event = FileEvent::new(FileEventKind::Created, "/tmp/x.rs".into());
        plugin.on_event(&event);
    }

    #[test]
    fn echo_plugin_setup_returns_empty_by_default() {
        let mut plugin = EchoPlugin::new(&[".rs"], "echo".to_string());
        assert!(plugin.setup().is_empty());
    }
}
