<h1 align="center">Spartan MQ</h1>

<h4 align="center">
  Documentation (<a href="https://ivan770.github.io/spartan/spartan/">Server</a> | <a href="https://ivan770.github.io/spartan/spartan_lib/">Library</a>) |
  <a href="https://github.com/ivan770/spartan/wiki/REST-API">REST API</a>
</h4>

<p align="center">
  <img src="https://github.com/ivan770/spartan/workflows/Test%20workspace/badge.svg">
</p>

<p align="center">
  Spartan MQ is a fast and easy to use message queue, written in Rust 🦀
</p>

<p align="center">
  <img src="./images/curl.png" width="75%">
</p>

## Features
* SQS-like message dispatching
* Rich messages, with support for timezone, timeout, delay, max tries, and states
* Integrated time handling
* Queue replication
* Redis-like database persistence using timers
* Background GC that helps you keep your queues tidy
* Key-based queue authorization
* Simple API

## Installation

### Download binary from GitHub

1. Download latest release from [GitHub](https://github.com/ivan770/spartan/releases/latest).
2. Create configuration file using `./spartan init`.
3. Start server with `./spartan start`.

### Build from source

Make sure you have latest **nightly** Rust toolchain installed on your system.

```
git clone https://github.com/ivan770/spartan
cd spartan
cargo build --release
```

## Configuration

### Generic flags

* `--config` - Change configuration file path (default: `Spartan.toml`).

### `start` command flags

* `--host` - Change server host (default: `127.0.0.1:5680`).

### Spartan.toml keys

* `queues` - Array of queue names (required).
* `body_size` - Max body size in bytes (default: 32 Kb).
* `gc_timer` - Amount of seconds between each GC job wake (GC cycle times vary, default: `300`).
* `persistence` - Persistence configuration for both log and snapshot drivers.
* `access_keys` - Table of queue access keys. Anonymous access to queues will not be permitted if this key has any value.
* `replication` - Replication configuration for both primary and replica nodes.

#### `persistence`
There are two available persistence drivers, that Spartan supports - `log` and `snapshot`.

|                    | `log` | `snapshot`         |
|--------------------|-------|--------------------|
| Performance        | -     | +                  |
| Small disk usage   | -     | +                  |
| Reliability        | +     | -                  |
| Compaction support | +     | Always compacted   |

By default, after executing `spartan init` command you'll receive config with `snapshot` driver, which is an optimal variant for most cases.

Both drivers support these configuration keys:
* `mode` - Persistence mode (default: `snapshot`).
* `path` - Database path (default: `./db`).
* `timer` - Timer between each queue persistence cycle for `snapshot` driver, and replication storage persistence cycle for `log` (default: 900 seconds).
* `compaction` - Enable `log` driver compaction on Spartan startup (default: true).

#### `access_keys`
Spartan has authentication and authorization mechanism using access keys.

To get access to protected queue, you need to have valid `Authorization` header in your request, with access key in it.

Keys may have multiple queues attached to them (you may also use `*` to create a wildcard key).

Example of configuration:
```toml
[[access_keys]]
key = "IHaveAccessToAllQueues"
queues = ["*"]

[[access_keys]]
key = "IHaveAccessToTestQueue"
queues = ["test"]

[[access_keys]]
key = "HelloWorld"
queues = ["test", "test2"]
```

Example of valid HTTP Authorization header:
```
Authorization: Bearer IHaveAccessToAllQueues
```

#### `replication`
Spartan also has support for queue replication.

Replication process will be restarted in case of any minor error (protocol or queue config mismatch).

If there is any problem with TCP socket, then connection will be dropped and re-opened with each replica.

##### Primary

The following config will start primary node that communicates with one replica every 180 seconds (default value):
```toml
replication = { Primary = { destination = ["127.0.0.1:12345"] } }
```

You may also use `replication_timer` key to change amount of seconds between each replication:
```toml
replication = { Primary = { destination = ["127.0.0.1:12345"], replication_timer = 30 } }
```

##### Replica

Change your replication config to following example:
```toml
replication = { Replica = { host = "127.0.0.1:12345" } }
```

Then, start replica node with `spartan replica` command.
