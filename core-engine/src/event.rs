// FileEvent and FileEventKind — Group 3 of the roadmap
// https://dev.to/luisccc/learning-by-doing-event-loop-in-rust-hf1

use std::path::PathBuf;
use std::io::Write;

// use notify::{Event, EventKind}; already does the detection
// of the event kind, but we're doing this in order to 
// decouple the engine from any dependency
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct FileEvent {
    pub kind: FileEventKind,
    pub path: PathBuf,
}

impl FileEvent {
    pub fn new(kind: FileEventKind, path: PathBuf) -> Self {
        FileEvent { kind, path }
    }
}

pub trait EventHandler {
    fn handle(&mut self, event: &FileEvent);
}

// Find out a better way
pub struct Logger<W: Write> {
    writer: W
}

// @TODO: See if the pattern in Rust is to put the struct then the impl
// right after the definition of the struct.
impl<W: Write> Logger<W> {
    pub fn new(writer: W) -> Self {
        Logger { writer }
    }
}

// TODO: Create a Error struct that maps the errors
// We can find a better way to re-write that without
// writting the expect part.
impl <W: Write> EventHandler for Logger<W> {
    fn handle(&mut self, event: &FileEvent) {
        match &event.kind {
            FileEventKind::Created => writeln!(self.writer, "Created: {:?}", event.path),
            FileEventKind::Modified => writeln!(self.writer, "Modified: {:?}", event.path),
            FileEventKind::Renamed { from, to } => writeln!(self.writer, "Renamed: {:?} -> {:?}", from, to),
            FileEventKind::Deleted => writeln!(self.writer, "Deleted: {:?}", event.path)
        }.expect("Failed to write event log");     
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logger_writes_each_kind(){
        let mut buf: Vec<u8> = Vec::new();
        let events: Vec<FileEvent> = vec![
            FileEvent::new(FileEventKind::Created, "/tmp/a".into()),
            FileEvent::new(FileEventKind::Modified, "/tmp/b".into()),
            FileEvent::new(FileEventKind::Deleted, "/tmp/c".into()),
            FileEvent::new(
                FileEventKind::Renamed { from: "/tmp/d".into(), to: "/tmp/e".into() },
                "/tmp/f".into(),
            ),
        ];

        {
            let mut logger = Logger::new(&mut buf);
            for event in &events {
                logger.handle(event);
            }
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Created"));
        assert!(output.contains("Modified"));
        assert!(output.contains("Deleted"));
        assert!(output.contains("Renamed"));
    }
}