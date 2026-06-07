# Installation

json_samp ships as a single self-contained library plus a Pawn include. The same
binary runs on **SA-MP** and on **Open Multiplayer** (native component or legacy).

## Download

Grab the latest release for your platform:

- `json_samp.so` — Linux i686 (`i686-unknown-linux-gnu`)
- `json_samp.dll` — Windows i686 (`i686-pc-windows-msvc`)
- `json_samp.inc` — Pawn include, identical for both servers

## Place the include

Copy `json_samp.inc` into your compiler's include folder:

- **Windows:** `pawno/include/` or `qawno/include/`
- **Linux:** `include/` at the server root

Then include it in your script:

```pawn
#include <a_samp>
#include <json_samp>
```

## Register the plugin

### SA-MP

Drop the binary into `plugins/` and add it to `server.cfg`:

```
plugins json_samp.so
```

(or `json_samp.dll` on Windows)

### Open Multiplayer — native component (recommended)

Drop the binary into the `components/` folder. open.mp auto-discovers it on start
and loads it via `ComponentEntryPoint`. **No `config.json` entry is required.**

### Open Multiplayer — legacy mode

Drop the binary into `plugins/` and declare it under `legacy_plugins` in
`config.json` (legacy plugins must be listed explicitly):

```json
{
  "pawn": {
    "legacy_plugins": ["json_samp"]
  }
}
```

!!! note "No system libraries"
    The JSON engine is compiled into the binary. There is nothing else to install.

## Logging

The plugin writes to `logs/json_samp.log` and to the server console. Control the
verbosity at runtime with [`json_log`](utilities.md#json_log):

```pawn
json_log(JSON_LOG_ERROR);   // 0=none, 1=error, 2=warning, 3=info, 4=all (default)
```
