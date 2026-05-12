use std::time::{Duration, Instant};
use crate::event::{FileEvent};
use std::collections::{HashMap};
use std::path::PathBuf;


/*

 // Create a debouncer with a 500ms delay
    // Events within this window get collapsed into one notification
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |result: Result<Vec<_>, _>| {
            tx.send(result).expect("Failed to send");
        },
    )?;

    use debounce::EventDebouncer;
use std::thread::sleep;
use std::time::Duration;

let delay = Duration::from_millis(10);
let debouncer = EventDebouncer::new(delay, move |data: String| {
    println!("{}", data);
});

debouncer.put(String::from("foo"));
debouncer.put(String::from("foo"));
debouncer.put(String::from("bar"));
sleep(delay);
*/
struct Debouncer {
    timeout: Duration,
    // Instant takes the current "time"
    last_seen: HashMap<PathBuf, Instant>
}

impl Debouncer {
    pub fn new(timeout: Duration) -> Self {
        Debouncer { 
            timeout,
            last_seen: HashMap::new()
         }
    }

    pub fn is_time_out(&self, path: &PathBuf) -> bool {
        match self.last_seen.get(path) {
            None => true,
            Some(last) => last.elapsed() >= self.timeout // time since last event on this path exceeds the debounce window
        }
    }

    pub fn record(&mut self, path: PathBuf){
        self.last_seen.insert(path, Instant::now());
    }
}