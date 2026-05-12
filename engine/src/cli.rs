use clap::{Parser, ValueEnum};

// cargo run -p engine -- --language typescript
#[derive(Debug, Clone, ValueEnum)]
pub enum Language {
    Typescript,
    Javascript,
}

/// Watches filesystem events, filters them by language or path rules, and triggers pluggable handlers
#[derive(Debug, Parser)]
#[command(name = "Watch-engine", version)]
pub struct Args {
    /// Name of the programming language to watch
    #[arg(short, long)]
    pub language: Language,
    #[arg(short, long)]
    pub recursive: bool
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}