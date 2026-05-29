# plato-manus — The Hands Module

File operations, API calls, and device control translated into a text-based interface for agents. All actions are policy-checked before execution.

**Part of the [Plato](https://github.com/SuperInstance/plato-shell) ecosystem.**

## What This Gives You

- **FileHand** — `ls`, `cat`, `write`, `mkdir`, `rm` with path-based access control
- **ApiHand** — `get`, `post`, `put`, `delete` with domain allowlists
- **DeviceHand** — `on`, `off`, `status`, `configure` for IoT and hardware
- **YAML policy** — configure allowed/denied paths, domains, and devices
- **ActionResult** — structured results that agents can parse

## Quick Start

```rust
use plato_manus::{Manus, HandPolicy, DeviceState};

let policy = HandPolicy::permissive();
let mut manus = Manus::new(policy);

// File operations
manus.write("/tmp/hello.txt", "world")?;
let content = manus.read("/tmp/hello.txt")?;
let listing = manus.ls("/tmp")?;

// Device control
manus.register_device("light", DeviceState::Off);
manus.device_on("light");
let status = manus.device_status("light")?;

// Generic command
let result = manus.execute("read /tmp/hello.txt");
```

## Policy Configuration

```yaml
allowed_paths:
  - /tmp
  - /home/agent/workspace
denied_paths:
  - /etc
  - /root
allowed_domains:
  - api.example.com
allowed_devices:
  - light
  - thermostat
```

## How It Fits

The "hands" of [plato-shell](https://github.com/SuperInstance/plato-shell). When an agent needs to interact with the real world — read a file, call an API, turn on a light — manus handles it with policy guardrails. Used alongside [plato-policy](https://github.com/SuperInstance/plato-policy) for access control.

## Installation

```toml
[dependencies]
plato-manus = "0.1"
```

## License

MIT
