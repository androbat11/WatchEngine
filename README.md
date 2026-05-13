# WatchEngine

WatchEngine provides a modular and extensible way to observe filesystem events, filter them using user-defined rules (such as file type or path patterns), and react through pluggable handlers.

## How to use

```bash
cargo run -p engine -- --watch <path> [options]
```

## Options

| Flag             | Description                                       |
| ---------------- | ------------------------------------------------- |
| `--watch <path>` | Directory to watch (required)                     |
| `--typescript`   | Watch `.ts` and `.tsx` files                      |
| `--javascript`   | Watch `.js`, `.mjs`, and `.cjs` files             |
| `--debounce <ms>`| Debounce window in milliseconds (default: 300)    |
| `--exec <cmd>`   | Shell command to run after each change            |

## Examples

Watch the current directory for TypeScript changes:

```bash
cargo run -p engine -- --watch . --typescript
```

Watch a specific path and run a command on change:

```bash
cargo run -p engine -- --watch ./src --typescript --exec "echo file changed"
```

Watch both TypeScript and JavaScript:

```bash
cargo run -p engine -- --watch . --typescript --javascript
```

## Notes

- `-p engine` tells Cargo which crate to run in the workspace
- `--debounce` collapses rapid save events into one notification (editor auto-save storms)
- `--exec` runs non-blocking — the watcher keeps running while the command executes
