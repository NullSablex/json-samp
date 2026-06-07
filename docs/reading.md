# Parsing & reading

## json_parse

```pawn
native json_parse(const json_str[], &out_id);
```

Parses `json_str` and stores it in the pool. On success returns `1` and writes
the new [handle](handles.md) into `out_id`; on failure returns `0` and logs
`(E001)`.

```pawn
new doc;
if (json_parse("{\"name\":\"Erick\",\"level\":42}", doc))
{
    // ... read from doc ...
    json_free(doc);
}
```

## json_is_valid

```pawn
native json_is_valid(const json_str[]);
```

Returns `1` if `json_str` is syntactically valid JSON, `0` otherwise. Does **not**
allocate a handle — ideal for validating input before parsing.

## json_free

```pawn
native json_free(ptr);
```

Releases a handle. Returns `1` if it existed, `0` otherwise. See
[Handles](handles.md).

## Typed getters (by key)

These read a **top-level key** of an object handle. They return `1` on success
and write the value into the reference parameter; on failure they return `0` and
log a code.

```pawn
native json_get_string(ptr, const key[], output[], size = sizeof(output));
native json_get_int(ptr, const key[], &output);
native json_get_float(ptr, const key[], &Float:output);
native json_get_bool(ptr, const key[], &output);
```

| Native | Accepts | Notes |
|---|---|---|
| `json_get_string` | any value | strings verbatim; other types are serialized to JSON text |
| `json_get_int` | number / numeric string | booleans are **not** coerced here |
| `json_get_float` | number / numeric string | booleans are **not** coerced here |
| `json_get_bool` | bool | strict — only a JSON boolean matches |

```pawn
new name[24], level, Float:balance, vip;
json_get_string(doc, "name", name);
json_get_int(doc, "level", level);
json_get_float(doc, "balance", balance);
json_get_bool(doc, "vip", vip);
```

!!! tip "Deep reads"
    To read nested values use the [path API](paths.md)
    (`json_get_int_at`, ...), which additionally coerces booleans to `0/1`.

## json_has_key / json_exists_key

```pawn
native json_has_key(ptr, const key[]);
native json_exists_key(ptr, const key[]); // synonym
```

Return `1` if the object handle contains `key`, `0` otherwise. Guard optional
fields before reading them:

```pawn
if (json_has_key(doc, "clan"))
{
    new clan[32];
    json_get_string(doc, "clan", clan);
}
```
