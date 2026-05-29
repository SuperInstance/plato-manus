# plato-manus

The "hands" module — file operations, API calls, and device control translated into a text-based interface for agents.

## Overview

Plato-Manus provides a text-based interface for AI agents to interact with the world through:

- **FileHand** — File system operations (`ls`, `cat`, `write`, `mkdir`, `rm`) with policy-checked paths
- **ApiHand** — HTTP API calls (`get`, `post`, `put`, `delete`) with domain allowlists
- **DeviceHand** — Device control abstraction (`on`, `off`, `status`, `configure`)
- **HandPolicy** — YAML-based access control for paths, domains, and devices
- **ActionResult** — Structured results that agents can parse

## Usage

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

## Policy

All operations are policy-checked. Configure allowed/denied paths, domains, and devices:

```yaml
allowed_paths:
  - /tmp
  - /home/agent/workspace
denied_paths:
  - /etc
  - /root
allowed_domains:
  - api.example.com
denied_domains:
  - evil.example.com
allowed_devices:
  - light
  - thermostat
```

## License

MIT
