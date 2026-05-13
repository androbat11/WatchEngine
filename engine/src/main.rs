mod cli;
mod adapter;
mod plugins;

use std::sync::mpsc;
use cli::Args;
use adapter::NotifyAdapter;
use plugins::{typescript::TypescriptPlugin, javascript::JavascriptPlugin};
use watch_core::{
    config::Config,
    dispatcher::Dispatcher,
    reactor::Reactor,
    registry::Registry,
    plugin::WatchPlugin,
};

fn main() {
    let args = Args::new();
    let typescript = args.typescript;
    let javascript = args.javascript;
    let config = Config::from(args);

    let (tx, rx) = mpsc::channel();

    let mut adapter = NotifyAdapter::new(tx);

    let mut registry = Registry::new();

    if typescript {
        let mut plugin = TypescriptPlugin::new(config.root.clone());
        for path in plugin.setup() {
            adapter.watch(&path);
        }
        registry.register(Box::new(plugin));
    }

    if javascript {
        let mut plugin = JavascriptPlugin::new(config.root.clone());
        for path in plugin.setup() {
            adapter.watch(&path);
        }
        registry.register(Box::new(plugin));
    }

    // fallback: always watch root so events from unregistered dirs still flow
    adapter.watch(&config.root);

    let dispatcher = Dispatcher::new(registry, None, config.debounce_ms);
    let mut reactor = Reactor::new(rx, dispatcher);
    reactor.run();
}
