# json_samp examples

Runnable Pawn snippets showing how to use the plugin on **SA-MP** and **Open Multiplayer**. Every `json_*` native behaves identically on both servers — the same compiled library and the same `json_samp.inc` are used everywhere.

| File | Topic |
|---|---|
| [`01_parse_and_read.pwn`](01_parse_and_read.pwn) | `json_parse`, typed `json_get_*`, `json_has_key`, `json_is_valid`, `json_free` |
| [`02_build_and_serialize.pwn`](02_build_and_serialize.pwn) | `json_create`, `json_set_*`, `json_set_null`, `json_to_string` / `json_to_string_pretty` |
| [`03_paths.pwn`](03_paths.pwn) | Path API: `json_get_*_at`, `json_set_*_at`, `json_set_at`, `json_exists_at`, `json_delete_at` |
| [`04_arrays.pwn`](04_arrays.pwn) | Array handles (`json_create_array`, `json_array_append_*`, `json_array_remove`) and arrays under object keys (`json_append_array`, `json_array_len`, `json_array_get_*`, `json_item`) |
| [`05_files.pwn`](05_files.pwn) | `json_open_file`, `json_save_file`, `json_create_file`, `json_reload_file` |
| [`06_inspect_and_iterate.pwn`](06_inspect_and_iterate.pwn) | `json_type`, `json_len`, `json_object_len` / `json_object_key_at`, `json_is_array` / `json_is_object`, `json_at` |
| [`07_clone_merge_utils.pwn`](07_clone_merge_utils.pwn) | `json_clone`, `json_merge`, `json_equals`, `json_clear`, `json_count`, `json_log` |

## Compiling

The examples assume the include path is set up so that `<json_samp>` resolves to [`../include/json_samp.inc`](../include/json_samp.inc).

```bash
pawncc -i../include 01_parse_and_read.pwn
```

Or copy `json_samp.inc` into your `pawno/include/` (SA-MP) or `qawno/include/` (open.mp) folder and compile from inside the gamemode tree.

## Installing the plugin

### SA-MP

Drop `json_samp.so` (Linux) or `json_samp.dll` (Windows) into `plugins/` and register it in `server.cfg`:

```
plugins json_samp.so
```

### Open Multiplayer — native component (recommended)

Drop the binary into the `components/` folder. open.mp auto-discovers it on start and loads it via `ComponentEntryPoint`. **No `config.json` entry is required** — the folder itself IS the registration.

### Open Multiplayer — legacy mode

Drop the binary into `plugins/` and declare it under `legacy_plugins` in `config.json`:

```json
{
  "pawn": {
    "legacy_plugins": ["json_samp"]
  }
}
```

## Conventions used across the examples

- A document **handle** is the integer id returned by `json_parse`, `json_create`, `json_create_array`, `json_open_file`, `json_clone`, `json_item` and `json_at`. Every handle you obtain must be released with `json_free()`, or it leaks — `json_count()` reports how many are currently open.
- Most natives return `1` on success and `0` on failure; the actual value is delivered through an `&output` reference parameter. Failures are logged to `logs/json_samp.log` (tune verbosity with `json_log`).
- Paths accept both **JSON Pointer** (`"/a/0/b"`) and a **friendly** dotted/bracket form (`"a[0].b"`) interchangeably.
- Relative file paths are resolved against the server's `scriptfiles/` directory.
