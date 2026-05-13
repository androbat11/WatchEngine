use clap::Parser;
use watch_core::config::Config;
use std::path::PathBuf;

/// Lightweight file watcher
#[derive(Debug, Parser)]
#[command(name = "watch-engine", version)]
pub struct Args {
    /// Directory to watch
    #[arg(long)]
    pub watch: PathBuf,

    /// Watch TypeScript files (.ts, .tsx)
    #[arg(long)]
    pub typescript: bool,

    /// Watch JavaScript files (.js, .mjs, .cjs)
    #[arg(long)]
    pub javascript: bool,

    /// Debounce window in milliseconds
    #[arg(long, default_value = "300")]
    pub debounce: u64,

    /// Shell command to run after each change
    #[arg(long)]
    pub exec: Option<String>,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Config {
            root: args.watch,
            debounce_ms: args.debounce,
            plugins: Vec::new(),
        }
    }
}
