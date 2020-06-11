<h1 align="center">Spartan MQ</h1>

<h4 align="center">
  <a href="https://ivan770.me/spartan">Website</a> |
  Documentation (<a href="https://ivan770.github.io/spartan/spartan/">Server</a> | <a href="https://ivan770.github.io/spartan/spartan_lib/">Library</a>)
</h4>

<p align="center">
  <img src="https://github.com/ivan770/spartan/workflows/Test%20workspace/badge.svg">
</p>

<p align="center">
  Spartan MQ is a fast and easy to use message queue, written in Rust 🦀
</p>

## Features
* SQS-like message dispatching
* Rich messages, with support for timezone, timeout, delay, max tries, and states
* Integrated time handling
* Redis-like database persistence using timers
* Background GC that helps you keep your queues tidy 
* Simple API

## Installation

### Download binary from GitHub

1. Download latest release from [GitHub](https://github.com/ivan770/spartan/releases/latest).
2. Create Spartan.toml configuration file with example configuration below:
```
queues = ["default"]
```
3. Create empty directory with name `db` (you may change directory name using `Spartan.toml`).
4. Run binary using `./spartan`, or `./spartan.exe` if you are using Windows.

### Build from source

Make sure you have Rust toolchain installed on your system.

```
git clone https://github.com/ivan770/spartan
cd spartan
cargo run --release
```

## Configuration

### Executable flags

* `--config` - Change configuration file path (default: `Spartan.toml`).
* `--host` - Change server host (default: `127.0.0.1:5680`).

### Spartan.toml keys

* `queues` - Array of queue names (required).
* `path` - Database path (default: `./db`).
* `persistence_timer` - Amount of seconds between each database write to disk (default: `900`).
* `gc_timer` - Amount of seconds between each GC job wake (GC cycle times vary, default: `300`).
