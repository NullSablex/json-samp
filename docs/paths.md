# Path API

The `*_at` natives read, write, test and delete values **anywhere** in a document
without allocating intermediate handles.

## Path syntax

Two interchangeable forms are accepted:

| Form | Example | Notes |
|---|---|---|
| JSON Pointer | `/server/players/0/name` | leading `/`; `~1` = `/`, `~0` = `~` |
| Friendly | `server.players[0].name` | dotted keys, `[n]` for array indices |

An empty path (`""`) refers to the **root** of the document. A path also works on
an array handle directly — index `0` is `"0"` (friendly) or `"/0"` (pointer).

## Existence

```pawn
native json_exists_at(ptr, const path[]);
```

Returns `1` if the path resolves to a node, `0` otherwise.

## Typed getters

```pawn
native json_get_string_at(ptr, const path[], output[], size = sizeof(output));
native json_get_int_at(ptr, const path[], &output);
native json_get_float_at(ptr, const path[], &Float:output);
native json_get_bool_at(ptr, const path[], &output);
```

Return `1` on success. Unlike the level-1 getters, the integer/float path getters
also coerce booleans to `0/1`. `json_get_string_at` returns strings verbatim and
serializes any other node to JSON text.

```pawn
new score, name[32];
json_get_int_at(doc, "/server/players/0/score", score);
json_get_string_at(doc, "server.name", name);
```

## Typed setters

```pawn
native json_set_string_at(ptr, const path[], const value[]);
native json_set_int_at(ptr, const path[], value);
native json_set_float_at(ptr, const path[], Float:value);
native json_set_bool_at(ptr, const path[], value);
native json_set_null_at(ptr, const path[]);
```

Return `1` on success, `0` on failure (missing handle, missing parent, bad array
index, or a non-container parent — see [error codes](errors.md)). The parent node
must already exist; an existing object key is created or overwritten.

```pawn
json_set_int_at(doc, "server.players[0].score", 99);
json_set_bool_at(doc, "server.online", true);   // creates "online"
```

## json_set_at (raw value)

```pawn
native json_set_at(ptr, const path[], const value_json[]);
```

Like the typed setters, but `value_json` is parsed as JSON, so you can write any
shape — objects, arrays, nested structures. Logs `(E050)` if `value_json` is
invalid. With an empty path it replaces the whole document.

```pawn
json_set_at(doc, "server.tags", "[\"rp\",\"survival\"]");
json_set_at(doc, "server.meta", "{\"region\":\"BR\"}");
```

## json_delete_at

```pawn
native json_delete_at(ptr, const path[]);
```

Removes the node at `path`. Returns `1` if something was removed, `0` otherwise.
Deleting the **root** (empty path) is not allowed and logs `(E054)`.

```pawn
json_delete_at(doc, "server.players[0].score");
```
