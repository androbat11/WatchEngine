use notify::{RecursiveMode, Watcher, recommended_watcher};
use std::{path::Path, sync::mpsc::{Sender}};
use watch_core::event::{FileEvent, FileEventKind};

pub struct NotifyAdapter {
    watcher: notify::RecommendedWatcher
}

impl NotifyAdapter {
    pub fn new(transmiter: Sender<FileEvent>) -> Self {
        let watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
            if let Ok(event) = result {
                if let Some(file_event) = parse(event) {
                    transmiter.send(file_event).ok();
                }
            }
        }).expect("Failed to create watcher");

        NotifyAdapter { watcher }
    }

    pub fn watch(&mut self, path: &Path) -> Option<()>{
        self.watcher.watch(path, RecursiveMode::Recursive).ok()
    }
}

fn parse(event: notify::Event) -> Option<FileEvent> {
    let path = event.paths.into_iter().next()?;
    let kind = match event.kind {
        notify::EventKind::Create(_) => FileEventKind::Created,
        notify::EventKind::Modify(_) => FileEventKind::Modified,
        notify::EventKind::Remove(_) => FileEventKind::Deleted,
        _ => return None,
    };

    Some(FileEvent::new(kind, path))
}

