# File I/O

Load, save, create and reload JSON files. **Relative paths resolve against the
server's `scriptfiles/` directory** — so `"data/players.json"` means
`scriptfiles/data/players.json`. Absolute paths and `./`, `../` or
`scriptfiles/...` prefixes are used as-is. Parent directories are created
automatically on save/create.

## json_open_file

```pawn
native json_open_file(const path[], &out_id);
```

Reads and parses the file into a new [handle](handles.md). Returns `1` on success;
`0` on empty path (`E010`), read error (`E011`) or invalid JSON (`E012`).

```pawn
new doc;
if (json_open_file("data/players.json", doc))
{
    new name[24];
    json_get_string(doc, "name", name);
    json_free(doc);
}
```

## json_save_file

```pawn
native json_save_file(ptr, const path[]);
```

Serializes the handle (**pretty-printed**) and writes it to `path`. Returns `1` on
success; `0` on empty path (`E013`), missing handle (`E002`), serialization
failure (`E014`), directory creation failure (`E015`) or write failure (`E016`).

```pawn
json_save_file(doc, "data/players.json");
```

## json_create_file

```pawn
native json_create_file(const path[]);
```

Ensures the file exists, writing an empty `{}` if it does not. If the file already
exists it is left untouched. Returns `1` on success; `0` on empty path (`E017`),
directory creation failure (`E018`) or write failure (`E019`).

```pawn
json_create_file("data/players.json");   // safe to call every start
```

## json_reload_file

```pawn
native json_reload_file(ptr, const path[]);
```

Re-reads `path` from disk into the **same** handle, discarding in-memory changes.
Useful after an external edit. Returns `1` on success; `0` on empty path (`E020`),
missing handle (`E002`), read error (`E021`) or invalid JSON (`E022`).

```pawn
json_reload_file(doc, "data/players.json");
```

## Typical pattern

```pawn
json_create_file("data/config.json");   // first run

new cfg;
if (json_open_file("data/config.json", cfg))
{
    json_set_int(cfg, "version", 2);
    json_save_file(cfg, "data/config.json");
    json_free(cfg);
}
```
