# Event Loop

## What It Is

The event loop is the runtime model of WatchEngine. It is the reason the
process stays alive after startup: instead of running once and exiting, it
blocks waiting for the next event, handles it, and blocks again — forever.

This document explains the concept. The implementation lives in the Reactor.
See `core/.reactor.md` for the Rust-specific details.

---

## The Model

```text
START
  │
  ▼
[Initialize: CLI, plugins, adapter]
  │
  ▼
┌─────────────────────────────────┐
│                                 │
│   BLOCK — wait for next event   │◄──┐
│                                 │   │
└──────────────┬──────────────────┘   │
               │ event arrives        │
               ▼                      │
        [Debounce check]              │
               │                      │
        ┌──────┴──────┐               │
        │ drop        │ forward        │
        │             ▼               │
        │      [Dispatch to plugins]  │
        │             │               │
        │             ▼               │
        │      [Run --exec if set]    │
        │             │               │
        └─────────────┴───────────────┘
                      loop
```

The loop runs until the process receives a shutdown signal (Ctrl+C) or the
channel is closed. There is no timer, no polling, no "check every N ms".
The loop is genuinely idle between events.

---

## Why Not Poll?

The alternative to a blocking event loop is polling:

```text
loop:
    sleep(100ms)
    check every file for changes
    handle any changes found
```

Polling wastes CPU every 100ms regardless of whether anything changed.
On a project with 10,000 files, checking all of them every 100ms would
peg a CPU core even when nothing is happening.

The blocking event loop uses zero CPU between events because `recv()` yields
the thread to the OS scheduler. The OS wakes the thread only when an event
arrives. This is the fundamental efficiency advantage of WatchEngine over
naive file watchers.

---

## Single-Threaded vs Multi-Threaded Event Loops

WatchEngine's event loop is single-threaded: the Reactor processes one event
at a time, in sequence.

**Advantages:**

- Simple to reason about — no race conditions in the kernel
- No mutex overhead for shared state
- Event order is preserved

**Tradeoff:**

- If a plugin's `on_event()` takes too long, it delays the next event

This tradeoff is acceptable here because plugins only print to stdout.
They do not do slow I/O or computation. The bottleneck is always the OS
delivering events, not the plugins processing them.

If plugins ever needed to do slow work, the Dispatcher would spawn a thread
per plugin call — but that complexity is not needed today.

---

## The Shutdown Path

When the user presses Ctrl+C:

```text
OS delivers SIGINT to the process
  │
  ▼
Rust's default SIGINT handler exits the process
  │
  ▼
All threads are dropped
  │
  ▼
The notify adapter (Sender) is dropped
  │
  ▼
Reactor's recv() returns Err(RecvError)
  │
  ▼
Reactor breaks out of the loop
  │
  ▼
main() returns, process exits cleanly
```

This is idiomatic Rust shutdown: the channel closing is the shutdown signal.
No explicit stop flags, no global booleans, no unsafe code.

---

## Recommended Reading

**Event Loop — JavaScript (Node.js) explainer**
Node.js made the event loop famous. The Node.js event loop explainer by
Bert Belder (libuv author) is one of the clearest explanations of how a
single-threaded event loop handles concurrency.
Search: "Bert Belder Node.js event loop" or "libuv event loop explained"

**`epoll` — Linux's blocking I/O multiplexer**
Under the hood, the blocking `recv()` in the Reactor ultimately calls `epoll_wait`
on Linux. Understanding `epoll` gives you the OS-level picture of how the
process sleeps efficiently and wakes precisely when an event arrives.
Search: "epoll explained Linux I/O multiplexing"
<https://man7.org/linux/man-pages/man7/epoll.7.html>

**The C10K Problem — Dan Kegel**
The paper that popularized non-blocking, event-driven I/O. Explains why
thread-per-connection models fail at scale and why event loops are the answer.
The problem is about network connections, not files, but the architectural
lesson is identical.
Search: "C10K problem Dan Kegel"
<http://www.kegel.com/c10k.html>

**Cooperative vs Preemptive Multitasking**
WatchEngine's event loop is cooperative: the Reactor voluntarily yields the
CPU by calling `recv()`. A preemptive scheduler (the OS) would interrupt it.
Understanding the difference explains why the event loop is efficient but
also why a slow plugin could theoretically stall it.
Search: "cooperative vs preemptive multitasking explained"
