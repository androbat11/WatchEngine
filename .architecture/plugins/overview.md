# Plugin System Overview

## What a Plugin Is

A plugin is the unit of language-specific or tool-specific behavior in
WatchEngine. The kernel watches files and delivers events; plugins decide
what should happen when an event arrives. A plugin for TypeScript reacts to
`.ts` changes by running `tsc --noEmit`. A plugin for Go reacts to `.go`
changes by running `go build`. The kernel itself knows nothing about
TypeScript, Go, or any other language — that knowledge lives entirely in
the plugin.

Plugins are user-installable: which plugins are active in a given run is
determined by CLI flags, not baked into the engine.

---

## Plugin vs Handler — Why a Separate Concept

The kernel already has an `EventHandler` trait (used by the internal
`Logger`). At first glance, a plugin looks like just another handler — it
reacts to events. But a plugin carries three things a generic handler does
not:

| | Handler (e.g. `Logger`) | Plugin (e.g. `TypeScriptPlugin`) |
|---|---|---|
| Filters by file type? | No — sees every event | Yes — declares which extensions |
| Has lifecycle metadata? | No | Yes — declares what to watch |
| Wired up by | Engine code, internally | User, via CLI flags |
| Lifetime | Always on | Optional, opt-in |

A plugin is a richer contract than a handler: it adds **selectivity**
(extensions), **self-description** (the directories it needs watched), and
**user-facing identity** (an `id` for logs, CLI references, registry
lookup). The two traits are kept distinct so internal handlers do not have
to invent fake metadata, and so plugins cannot be confused with kernel
machinery.

---

## What Plugins Do — and What They Do Not Do

> **Plugins react to events. They do not watch files themselves.**

This rule exists because WatchEngine's value proposition is **one unified
watcher**. The user runs WatchEngine instead of `tsc --watch` plus
`nodemon` plus `go run -watch` plus a Python file watcher. If a plugin
spawned its own watcher, the user would gain nothing over running that
language's native tool directly — and they would be paying for two
watchers monitoring the same files.

The rule also matters for resource cost. `tsc --watch`, for example, holds
the entire TypeScript program graph in memory continuously and becomes
expensive on large codebases. A WatchEngine plugin instead invokes
`tsc --noEmit` reactively, scoped to the affected files. The kernel does
the watching work once, centrally, and plugins do narrowly-scoped work
only when an event fires.

```text
✓ TypeScriptPlugin → runs tsc --noEmit on changed file
✗ TypeScriptPlugin → spawns tsc --watch in the background

✓ GoPlugin → runs go build on the affected package
✗ GoPlugin → starts a Go-specific file watcher

✓ TestPlugin → runs the test for the changed file
✗ TestPlugin → starts pytest --watch
```

If a plugin needs richer triggering than "an event arrived for one of my
extensions," that logic lives inside the plugin's `on_event` method, not
in a separate watcher.

---

## The Trait Contract

```rust
pub trait WatchPlugin {
    fn id(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn on_event(&self, event: &FileEvent);
}
```

Every method exists for a reason that maps directly to the plugin's job:

- `id()` — distinguishes plugins for logging, error messages, and the
  registry. A short string identifier the user might also see in CLI
  flags (`--plugin typescript`).
- `extensions()` — declarative metadata the kernel uses to decide which
  events to deliver to this plugin. The plugin does not filter events
  itself; the kernel filters on its behalf using this list.
- `on_event(&self, ...)` — the reaction. Takes `&self`, not `&mut self`,
  because handling an event should not mutate the plugin's configuration.
  Any per-event state lives on the stack.

`setup()` will be added later, once `Config` exists. It will return the
list of directories the plugin wants watched, so the kernel can configure
the OS watcher without the user having to repeat that knowledge on the
CLI.

---

## Configuration — Generic vs Specific (XOR at the CLI Layer)

WatchEngine ships with two operating modes:

- **Specific mode** — the user passes one or more `--plugin` flags
  (`--plugin typescript --plugin go`). Only those plugins are loaded.
- **Generic mode** — the user passes no `--plugin` flag at all. A single
  `GenericPlugin` is loaded as the default fallback, watching every file
  and applying a generic action (e.g., a user-supplied `--exec` command).

The two modes are **mutually exclusive**. A user cannot run generic and
specific plugins at once.

```text
if any --plugin flags were given:
    plugins = [those specific plugins]
else:
    plugins = [GenericPlugin::new()]
```

### Why XOR

Generic plugins watch every file; specific plugins watch a subset. If both
were active, every `.ts` change would trigger the TypeScript plugin's
work *and* the generic action. That is duplicated effort the user
probably did not intend, and it muddles the mental model: a user running
`--plugin typescript` is making a clear statement that they want
TypeScript handling — not "TypeScript handling, plus also some generic
catch-all behavior." Forcing the choice keeps the user's intent
unambiguous.

### Where the rule lives

The XOR rule is enforced **at the CLI layer**, before any plugin reaches
the Registry. The kernel — Registry, Reactor, Dispatcher — does not know
this rule exists. It just receives whatever set of plugins the CLI hands
it. This keeps the kernel minimal and policy-free; the user-facing rule
lives in the user-facing layer, where it belongs.

---

## Routing — Chain of Responsibility

When multiple plugins are active in specific mode and more than one claims
the same extension, **all matching plugins fire**, in registration order.
Each does its own independent work. No plugin is "primary" or "fallback"
within the matched set.

This is the [Chain of Responsibility][cor] pattern: each handler in the
chain processes the event independently, and the chain runs to completion.
It is the right model here because plugins typically do non-overlapping
work on the same event (e.g., compile + log + notify), and forcing one
plugin to dominate the others would prevent useful composition.

The XOR rule above makes the routing question simpler in practice: at
runtime the kernel never holds a generic + specific mix, so the only way
multiple plugins can match an event is if the user explicitly loaded two
specific plugins that overlap. That is a deliberate user choice, and the
kernel honours it by calling all matching plugins.

[cor]: https://en.wikipedia.org/wiki/Chain-of-responsibility_pattern

---

## MVP Scope

For the initial implementation, defer the following:

- **Wildcard matching (`"*"`)** — the kernel does not need to support a
  catch-all extension yet. Build specific plugins first.
- **`GenericPlugin`** — not needed until wildcard matching exists. The
  XOR rule is documented but only the specific branch is implemented.
- **`setup()`** — depends on `Config`, which does not yet exist.
- **Multiple specific plugins claiming the same extension** — supported by
  the design (Chain of Responsibility) but not exercised by any built-in
  plugin yet.

This keeps the first plugin implementation small: define the trait,
implement it for one language (e.g., TypeScript), wire it into the
Registry, and watch it react to events through the Reactor.

---

## Why This Shape Was Chosen

Each design choice traces back to a property of the system:

| Property | Design choice |
|---|---|
| Plugins target specific file types | `extensions()` on the trait |
| Plugins should not duplicate watching effort | "react, do not watch" rule |
| Kernel must stay minimal and language-agnostic | Routing uses declarative metadata, not kernel-level language code |
| User intent should be unambiguous | Generic / specific are XOR at the CLI |
| Plugins are user-installable, not engine-internal | Trait separate from `EventHandler` |

The `WatchPlugin` trait is the smallest contract that supports all five
properties simultaneously.

---

## Related Documents

- [Plugin Registry](../core/plugin-registry.md) — how the kernel stores
  and queries the active plugin set.
- [Event Loop](../event.loop.md) — how events flow from the OS to the
  Reactor and into plugin `on_event` calls.
