// What registry does:
// - Hold a list of registered plugins.
// - Register new plugins (called once at startup by CLI)
// - Find plugins whose extensions() matches given ext
use crate::plugin::WatchPlugin;

// @TODO: See I don't have any validations for failure cases
pub struct Registry {
    // dyn WatchPlugin -> Let's see that there.
    plugins: Vec<Box<dyn WatchPlugin>>
}

impl Registry {
    pub fn new() -> Self {
        Registry { plugins: Vec::new() } 
    }
   
    pub fn find_by_extension(&self, ext: &str) -> Vec<&dyn WatchPlugin> {
       self.plugins
           .iter()
           .filter(|p| p.extensions().iter().any(|e| *e == ext))
           .map(|p| p.as_ref())
           .collect()
    }

    pub fn register(&mut self, plugin: Box<dyn WatchPlugin>) -> () {
        self.plugins.push(plugin)
    }
    
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::event::{FileEvent, FileEventKind};
  use std::path::PathBuf;

  struct MockPlugin {
    exts: &'static [&'static str],
  }

  impl WatchPlugin for MockPlugin {
    fn name(&self) -> &str { "mock" }
    fn extensions(&self) -> &'static [&'static str] { self.exts }
    fn setup(&mut self) -> Vec<PathBuf> { Vec::new() }
    fn on_event(&self, _event: &FileEvent) {}
  }

  #[test]
  fn find_by_ext_test(){
    let mut registry = Registry::new();
    let plugin = Box::new(MockPlugin { exts: &["rs"] });
    registry.register(plugin);

    let found = registry.find_by_extension("rs");
    assert_eq!(found.len(), 1);

    let not_found = registry.find_by_extension("ts");
    assert_eq!(not_found.len(), 0);
  }

}