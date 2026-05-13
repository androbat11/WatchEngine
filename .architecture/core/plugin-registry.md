# Plugin Registry

## What It Is

The Plugin Registry is the kernel's catalogue.
It holds every plugin that was loaded at startup and answers one question
when the Dispatcher asks: **which plugins handle this file extension?**

It also manages the plugin lifecycle:

- Calls `setup()` on every plugin once, at startup
- Calls teardown when the process exits

---

## Why It Exists Separately From the Dispatcher

The Registry and the Dispatcher are separated by responsibility:

| Component  | Responsibility                           |
|------------|------------------------------------------|
| Registry   | Knows WHAT plugins exist and WHICH match |
| Dispatcher | Knows HOW to call them                   |

If they were merged, a single component would be responsible for both
cataloguing and execution — two different concerns. Keeping them separate
means you can change how dispatching works (sync, async, ordered) without
touching how plugins are stored and looked up.

---

## Its Place in the Kernel

```text
CLI
  │
  │  registers plugins at startup
  ▼
Plugin Registry  ◄── you are here
  │
  │  Dispatcher queries it at runtime
  ▼
  returns: Vec<&dyn WatchPlugin>
```

The Registry is populated **once** at startup by the CLI.
At runtime, it is read-only — the Dispatcher only ever queries it, never modifies it.

---

## What It Holds

In **specific mode** the Registry holds the user-selected language plugins:

```text
plugins: Vec<Box<dyn WatchPlugin>>

[0]  TypeScriptPlugin  { id: "typescript", extensions: [".ts", ".tsx"] }
[1]  JavaScriptPlugin  { id: "javascript", extensions: [".js", ".mjs", ".cjs"] }
```

In **generic mode** (no `--plugin` flag passed) the Registry holds exactly
one plugin:

```text
plugins: Vec<Box<dyn WatchPlugin>>

[0]  GenericPlugin     { id: "generic",    extensions: ["*"] }
```

The two modes are mutually exclusive — see
[plugins/overview.md](../plugins/overview.md). The Registry itself does
not enforce this; the CLI guarantees only one shape ever reaches the
Registry.

`Box<dyn WatchPlugin>` is how Rust stores a value of unknown concrete type
behind a known interface (trait). Each slot in the `Vec` can hold a completely
different struct — as long as it implements `WatchPlugin`.

---

## How find_by_extension Works

```text
Query: ".ts"

Scan every plugin:
  TypeScriptPlugin.extensions() = [".ts", ".tsx"]  →  ".ts" found  ✓
  TypeScriptTestPlugin.extensions() = [".ts"]      →  ".ts" found  ✓
  JavaScriptPlugin.extensions() = [".js", ".mjs"]  →  ".ts" not found  ✗

Returns: [TypeScriptPlugin, TypeScriptTestPlugin]
```

When more than one plugin claims the same extension, **all** matching
plugins are returned and called in registration order — Chain of
Responsibility. Each plugin does its own independent work.

### Generic vs specific plugins

`GenericPlugin` is the fallback that runs when the user passes no
`--plugin` flag, declared with `extensions: ["*"]` so it matches every
event. By design, `GenericPlugin` and specific plugins **never coexist**
in the same Registry: the CLI enforces an XOR rule (specific plugins, or
generic, never both). See [plugins/overview.md](../plugins/overview.md)
for the full rationale. As a result, `find_by_extension` never has to
reason about generic-vs-specific priority — only one or the other is ever
loaded.

Wildcard matching is **deferred for the MVP**. The first version of the
Registry only handles concrete extensions; `"*"` support is added when
`GenericPlugin` is implemented.

---

## How setup_all Works

At startup, after all plugins are registered, the kernel calls `setup_all()`.
Each plugin's `setup()` returns a list of directories it wants watched.

```text
TypeScriptPlugin.setup() → ["/project/src", "/project/tests"]
JavaScriptPlugin.setup() → ["/project/scripts"]
GenericPlugin.setup()    → ["/project"]

setup_all() returns: ["/project/src", "/project/tests", "/project/scripts", "/project"]
```

The kernel then passes this flat list to the `NotifyAdapter`, which registers
each path with the OS watcher. Plugins never touch the OS watcher directly.

---

## Rust Concepts Involved

**`Vec<Box<dyn Trait>>`**
This is the core Rust mechanism for heterogeneous collections.
`Vec` is Rust's growable array. `Box` allocates on the heap.
`dyn WatchPlugin` means "some type that implements WatchPlugin, resolved at runtime."
Without `Box`, the compiler would need to know the size of every element at compile
time — but different plugins have different sizes. `Box` fixes that by storing
a pointer (fixed size) that points to the actual data.
Reading: The Rust Book, Chapter 15.1 — `Box<T>`.
Reading: The Rust Book, Chapter 17.2 — Trait Objects.

**Dynamic dispatch vs static dispatch**
When you call `plugin.on_event()` through `Box<dyn WatchPlugin>`, Rust uses
a vtable — a table of function pointers — to find the right implementation
at runtime. This is called dynamic dispatch. It has a tiny overhead vs
static dispatch, but it is the correct tool here because the set of plugins
is determined at runtime (by CLI flags), not at compile time.

**`&mut self` vs `&self`**
`setup()` takes `&mut self` because a plugin may store discovered paths in its
own fields during setup (mutating itself).
`on_event()` takes `&self` because handling an event should not change the
plugin's configuration — it is a read-only operation.

---

## Pseudocode

```text
struct PluginRegistry {
    plugins: Vec<Box<dyn WatchPlugin>>,
}

fn register(plugin: Box<dyn WatchPlugin>) {
    plugins.push(plugin)
}

fn find_by_extension(ext: &str) -> Vec<&dyn WatchPlugin> {
    plugins
        .filter(|p| p.extensions().contains(ext) || p.extensions().contains("*"))
        .collect()
}

fn setup_all(config: &Config) -> Vec<PathBuf> {
    plugins
        .flat_map(|p| p.setup(config))
        .collect()
}
```

---

## Recommended Reading

### The Pattern Behind the Registry

**Registry Pattern — Martin Fowler**
The Plugin Registry is a direct implementation of Fowler's Registry pattern.
A Registry is a well-known object that other objects can use to find common
objects and services. Read this before anything else on this list.
Search: "Martin Fowler Registry pattern" — available on martinfowler.com

**Service Locator vs Dependency Injection — Martin Fowler**
The Registry is related to the Service Locator pattern. Fowler's article
explains the difference and why Service Locator can hide dependencies in
ways that make code harder to test. Understanding the tradeoff tells you
exactly why the Registry in WatchEngine is structured the way it is.
Search: "Martin Fowler Inversion of Control Containers Service Locator"

**Plugin Architecture — how real plugin systems are designed**
Mozilla's extension system, VS Code's extension API, and Vim's plugin model
all solve the same problem differently. Reading about one real-world plugin
system gives context for why the `WatchPlugin` trait looks the way it does.
Search: "VS Code extension API architecture" or "plugin system design patterns"

### How Rust Implements This Internally

**Vtables and Dynamic Dispatch in Rust — Jon Gjengset**
When you call `plugin.on_event()` through `Box<dyn WatchPlugin>`, Rust uses
a vtable. This article explains exactly what a vtable is, how Rust lays it
out in memory, and what the cost of a virtual call is compared to a direct call.
Search: "Rust vtables Jon Gjengset" or read his book "Rust for Rustaceans", Chapter 2.

**Rust Reference — Trait Objects**
The authoritative low-level explanation of how `dyn Trait` works in Rust,
including the fat pointer representation (data pointer + vtable pointer).
<https://doc.rust-lang.org/reference/types/trait-object.html>

**`std::raw::TraitObject` (nightly) — understanding fat pointers**
A fat pointer is two words: one points to the data, one points to the vtable.
This is why `Box<dyn WatchPlugin>` is twice the size of a regular pointer.
Search: "Rust fat pointer trait object internals"

### The GoF Patterns at Play Here

**Strategy Pattern (Gang of Four)**
The Plugin Registry uses Strategy under the hood — each plugin is a strategy
for handling a particular file type. The kernel selects the strategy at runtime
based on file extension.
Read: "Design Patterns" by Gamma, Helm, Johnson, Vlissides — Strategy chapter.
Or search: "Strategy pattern explained" for a free explainer.

**Chain of Responsibility Pattern (Gang of Four)**
When multiple plugins match the same extension (TypeScript + Generic),
the Dispatcher calls them in sequence. This is Chain of Responsibility —
each handler in the chain processes the event independently.
Read: GoF "Design Patterns" — Chain of Responsibility chapter.
