# json-samp

**json-samp** is a JSON plugin for **SA-MP** and **Open Multiplayer**, written in
Rust on top of [rust-samp](https://github.com/NullSablex/rust-samp) and
[`serde_json`](https://github.com/serde-rs/json).

The same compiled library (`json_samp.so` / `json_samp.dll`) runs on SA-MP and on
Open Multiplayer (native component or legacy mode).

## Features

- Parse and validate JSON strings.
- Build JSON documents from scratch.
- Read and write values by key (object root) or by **path**
  (JSON Pointer `/a/b` and friendly syntax `a.b[0].c`).
- Load, save, create and reload JSON files (paths resolve under `scriptfiles/`).
- Inspect objects and arrays (length, keys, items, types).
- Document pool indexed by integer ID, freed explicitly with `json_free`.

## Quick example

```pawn
#include <a_samp>
#include <json_samp>

public OnGameModeInit()
{
    new id;
    if (json_parse("{\"name\":\"Erick\",\"level\":7}", id))
    {
        new name[24], level;
        json_get_string(id, "name", name);
        json_get_int(id, "level", level);
        printf("name=%s level=%d", name, level);
        json_free(id);
    }
    return 1;
}
```

## Next steps

- [Installation](installation.md)
- [Handles & conventions](handles.md)
- [Parsing & reading](reading.md)
- [Building & serializing](building.md)
- [Path API](paths.md)
- [Arrays](arrays.md)
- [File I/O](files.md)
- [Inspection & navigation](inspection.md)
- [Utilities & diagnostics](utilities.md)
- [Error codes](errors.md)
- [API reference](api-reference.md)

## License

Distributed under the **AGPL-3.0-or-later** license.
