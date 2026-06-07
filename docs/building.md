# Building & serializing

## json_create

```pawn
native json_create(&out_id);
```

Allocates an empty **object** handle and writes it to `out_id`. Always returns `1`.

```pawn
new doc;
json_create(doc);
```

For an empty array, use [`json_create_array`](arrays.md#json_create_array).

## Typed setters (by key)

These insert or overwrite a **top-level key** of an object handle. They return
`1` on success, or `0` if the handle is missing or is not an object.

```pawn
native json_set_string(ptr, const key[], const value[]);
native json_set_int(ptr, const key[], value);
native json_set_float(ptr, const key[], Float:value);
native json_set_bool(ptr, const key[], value);
native json_set_null(ptr, const key[]);
```

```pawn
new doc;
json_create(doc);
json_set_string(doc, "name", "Erick");
json_set_int(doc, "level", 42);
json_set_float(doc, "balance", 1500.75);
json_set_bool(doc, "vip", true);
json_set_null(doc, "clan");      // explicit JSON null
```

!!! tip "Nested writes"
    To set values deep inside a document, use the
    [typed path setters](paths.md#typed-setters) (`json_set_int_at`, ...) or
    [`json_set_at`](paths.md#json_set_at) with a raw JSON value.

## Serializing

```pawn
native json_to_string(ptr, output[], size = sizeof(output));
native json_to_string_pretty(ptr, output[], size = sizeof(output));
```

Render a handle back to text — compact or indented. Return `1` on success, `0` if
the handle is missing.

```pawn
new compact[256], pretty[256];
json_to_string(doc, compact);          // {"name":"Erick","level":42}
json_to_string_pretty(doc, pretty);    // multi-line, 2-space indent
```

!!! warning "Buffer size"
    The output is truncated to fit `size`. Size your buffer for the largest
    document you expect to serialize.
