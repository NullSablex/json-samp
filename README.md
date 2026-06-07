# json_samp

> JSON plugin for SA-MP and Open Multiplayer, written in Rust — by [NullSablex](https://github.com/NullSablex)

![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
![SA-MP](https://img.shields.io/badge/SA--MP-0.3.7+-orange)
![Open Multiplayer](https://img.shields.io/badge/Open%20Multiplayer-native%20%26%20legacy-orange)
![Build](https://img.shields.io/badge/build-Linux%20%7C%20Windows-green)
![Architecture](https://img.shields.io/badge/arch-x86%20(32--bit)-lightgrey)
[![Release](https://img.shields.io/github/v/release/NullSablex/json-samp?label=download)](https://github.com/NullSablex/json-samp/releases/latest)

## Overview

**json_samp** is a modern JSON plugin for SA-MP (San Andreas Multiplayer) and [Open Multiplayer](https://open.mp) (open.mp), written entirely in Rust on top of [`serde_json`](https://github.com/serde-rs/json). It exposes a complete API for parsing, building, querying and persisting JSON documents, with zero external runtime dependencies.

The same binary loads on SA-MP and on Open Multiplayer — natively as a component (recommended) or via legacy mode.

### Highlights

- **Zero external dependencies** — the JSON engine is compiled directly into the binary. No system libraries to install.
- **Handle-based pool** — documents live in the plugin and are referenced by an integer id; `json_count` helps you spot leaks.
- **Two path syntaxes** — read and write deep values with JSON Pointer (`/a/0/b`) or a friendly dotted/bracket form (`a[0].b`), interchangeably.
- **Typed and raw access** — typed getters/setters for the common cases, raw `value_json` when you need full control.
- **File I/O** — load, save (pretty-printed), create and reload documents, with paths resolved against `scriptfiles/`.
- **Universal binary** — built on [rust-samp](https://github.com/NullSablex/rust-samp) v3.0.0; one `.so`/`.dll` runs on SA-MP and on Open Multiplayer (native component or legacy).
- **Simple deploy** — drop the `.so` or `.dll` in and you are done.

## Installation

1. Download the latest release for your platform:
   - `json_samp.so` (Linux i686)
   - `json_samp.dll` (Windows i686, MSVC ABI)
   - `json_samp.inc` (Pawn include, shared between SA-MP and Open Multiplayer)
2. Place the binary in the server's `plugins/` directory.
3. Copy `json_samp.inc` to your compiler's include folder:
   - **Windows:** `pawno/include/` or `qawno/include/`
   - **Linux:** `include/` (at the server root)
4. Register the plugin:
   - **SA-MP** — add to `server.cfg`:
     ```
     plugins json_samp.so
     ```
     (or `json_samp.dll` on Windows)
   - **Open Multiplayer (native, recommended)** — drop the binary into the `components/` folder. open.mp auto-discovers it on start and loads it via `ComponentEntryPoint`. No `config.json` entry required.
   - **Open Multiplayer (legacy)** — same binary works as a legacy plugin. Drop it into `plugins/` and add it to `legacy_plugins` in `config.json` (this one DOES need to be declared, otherwise open.mp skips legacy plugins).

> [!IMPORTANT]
> No system library is required. The plugin is self-contained.

## Quick start

```pawn
#include <a_samp>
#include <json_samp>

public OnGameModeInit() {
    // Parse a document
    new doc;
    if (!json_parse("{\"name\":\"Erick\",\"level\":42}", doc))
        return 1;

    new name[24], level;
    json_get_string(doc, "name", name);
    json_get_int(doc, "level", level);
    printf("%s is level %d", name, level);

    // Mutate and serialize
    json_set_int(doc, "level", level + 1);

    new out[128];
    json_to_string(doc, out);
    printf("updated: %s", out);

    json_free(doc);   // release the handle
    return 1;
}
```

Browse the [examples/](examples/) folder for self-contained `.pwn` scripts covering parsing, building, the path API, arrays, file I/O, inspection and the clone/merge utilities. Every `json_*` native is identical across SA-MP and Open Multiplayer, so all examples build and run on both — only the installation path differs.

## Documentation

The plugin documentation lives in [docs/](docs/) and the generated Pawn include is the canonical native reference:

| Resource | Contents |
|---|---|
| [docs/index.md](docs/index.md) | Introduction, install and a worked example |
| [include/json_samp.inc](include/json_samp.inc) | Every native, grouped by topic, with signatures |
| [examples/](examples/) | Runnable snippets per feature area |

### Handle lifecycle

A **handle** is the integer id returned by `json_parse`, `json_create`, `json_create_array`, `json_open_file`, `json_clone`, `json_item` and `json_at`. Always release it with `json_free()` when done — `json_count()` reports how many are currently open. Most natives return `1` on success and `0` on failure, delivering their result through an `&output` parameter; failures are logged to `logs/json_samp.log` (verbosity tunable with `json_log`).

## Building from source

### Requirements

- Rust stable toolchain with the targets `i686-unknown-linux-gnu` and `i686-pc-windows-msvc`
- `cargo-xwin` for cross-compiling the Windows `.dll` from Linux (installed automatically by the script)
- No system libraries — the build is 100% Rust

### Development build

```bash
cargo build --target i686-unknown-linux-gnu
cargo test          # run the unit test suite
```

### Release build (Linux + Windows)

From Linux:

```bash
./scripts/build-linux.sh
```

From Windows (Git Bash):

```bash
./scripts/build-windows.sh
```

Both scripts produce `dist/json_samp.so` and `dist/json_samp.dll`, each with full SA-MP + Open Multiplayer native support. The Pawn include `include/json_samp.inc` is regenerated from `include/json_samp.inc.in` on every build.

> [!CAUTION]
> This plugin is distributed under the AGPL v3. Any derivative work — including one offered over a network — must keep the source code open under the same license.

## License

Copyright (c) 2026 NullSablex

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE).
