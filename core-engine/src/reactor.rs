use crate::event::{EventHandler, FileEvent};
use std::sync::mpsc::Receiver;


struct Reactor<H: EventHandler> {
    receiver: Receiver<FileEvent>,
    handler: H
}

/// Event loop for capturing the events.
impl<H: EventHandler> Reactor<H> {
    pub fn new(receiver: Receiver<FileEvent>, handler: H) -> Self {
        Reactor { receiver, handler }
    }

    pub fn run(&mut self){
        for event in self.receiver.iter(){
            self.handler.handle(&event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{FileEvent, FileEventKind, Logger};
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn reactor_processs_events(){
        // Need some sort of way to transmit the events
        let (sender, receiver) = mpsc::channel::<FileEvent>();
        // Got to store the events 
        let mut buf: Vec<u8> = Vec::new();

        // Transmit the event with the change to the receiver
        let producer = thread::spawn(move || {
            sender.send(FileEvent::new(FileEventKind::Created, "/tmp/a".into())).unwrap();
            sender.send(FileEvent::new(FileEventKind::Modified, "/tmp/b".into())).unwrap();
        });

        {
            let logger = Logger::new(&mut buf);
            let mut reactor = Reactor::new(receiver, logger);
            reactor.run();
        }

        producer.join().unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Created:"));
        assert!(output.contains("Modified"));
    }
}