# agent-mcp-adb

Exposes the Android devices you pick (connected via `adb`) to AI agents over MCP.

The MCP server (streamable HTTP, on a random local port by default) starts right away. A terminal UI lists devices
reported by `adb devices -l` and lets you choose which ones the agent may control; selection changes apply immediately,
and disconnected devices are dropped automatically.

Each device has the following fields:

| Field   | Description                                                                                    |
|---------|------------------------------------------------------------------------------------------------|
| id      | Monotonically increasing number used by tools; never reused, a reconnected device keeps its id |
| serial  | Device id reported by `adb devices`                                                            |
| name    | `device:` reported by `adb devices -l` (falls back to the serial when unknown)                 |
| product | `product:` reported by `adb devices -l` (absent for unauthorized/offline devices)              |
| model   | `model:` reported by `adb devices -l` (absent for unauthorized/offline devices)                |

## Tools

| Tool           | Description                                                                           |
|----------------|---------------------------------------------------------------------------------------|
| `list_devices` | Lists exposed devices with their ids, names, serials, products and models             |
| `screenshot`   | PNG of the device screen (secure content may come out black)                          |
| `ui_dump`      | XML from `uiautomator dump`, streamed without a file; `compressed` for smaller output |
| `wait`         | Sleep for the given number of milliseconds                                            |
| `tap`          | Tap at screen coordinates                                                             |
| `swipe`        | Swipe between two screen points, e.g. to scroll                                       |

All coordinates are device pixels, the same as in screenshots and `ui_dump` bounds.

`ui_dump` runs `uiautomator dump /dev/tty` so nothing is written to the device. On devices where that doesn't work it
falls back to a temporary file in `/data/local/tmp`, which is removed right after reading.

## Building

```sh
cargo build --release
```

The binary is written to `target/release/agent-mcp-adb`.

## Usage

```sh
agent-mcp-adb [OPTIONS]
# or
agent-mcp-adb --help
# or
agent-mcp-adb --address ::1 --port 8080 --adb ~/Library/Android/sdk/platform-tools/adb
```

| Option                      | Description                  | Default          |
|-----------------------------|------------------------------|------------------|
| `-a`, `--address <ADDRESS>` | MCP server IPv4/IPv6 address | `127.0.0.1`      |
| `-p`, `--port <PORT>`       | MCP server port              | random free port |
| `--adb <ADB>`               | Path to the adb executable   | `adb`            |
| `-h`, `--help`              | Print help                   |                  |

The MCP server URL is shown in the device list title. Press `c` to copy the command that starts Claude Code with this
server:

```sh
claude --mcp-config '{"mcpServers":{"mdev_adb":{"type":"http","url":"http://127.0.0.1:<port>/mcp"}}}' \
  --strict-mcp-config \
  --allowedTools mcp__mdev_adb
```

The server is passed via `--mcp-config`, so nothing is saved to the Claude Code configuration. All of its tools are
allowed without prompting, and other configured MCP servers are ignored for that session.

| Key     | Action                     |
|---------|----------------------------|
| `↑` `↓` | Previous / next device     |
| `←` `→` | First / last device        |
| `enter` | Toggle device exposure     |
| `c`     | Copy Claude launch command |
| `q`     | Quit                       |

Devices that are not ready (e.g. `unauthorized`, `offline`) are shown with their state in red.
