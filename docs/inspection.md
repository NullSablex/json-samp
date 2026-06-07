# Inspection & navigation

## json_type

```pawn
native json_type(ptr, &out_type); // see JSON_TYPE_* enum
```

Writes the node's type into `out_type`. Returns `1` on success, `0` if the handle
is missing.

```pawn
enum {
    JSON_TYPE_NULL = 0,
    JSON_TYPE_BOOL,
    JSON_TYPE_NUMBER,
    JSON_TYPE_STRING,
    JSON_TYPE_ARRAY,
    JSON_TYPE_OBJECT,
}
```

## json_is_array / json_is_object

```pawn
native json_is_array(ptr);
native json_is_object(ptr);
```

Return `1`/`0` directly. Handy shortcuts over `json_type`.

## json_len

```pawn
native json_len(ptr, &out_len);
```

Writes the element count into `out_len`: array length, object key count, or `0`
for scalars. Returns `1` on success, `0` if the handle is missing.

## Iterating object keys

```pawn
native json_object_len(ptr, &out_len);
native json_object_key_at(ptr, index, output[], size = sizeof(output));
native json_key_at(ptr, index, output[], size = sizeof(output)); // object only
```

`json_object_len` writes the key count; `json_object_key_at` / `json_key_at` read
the key name at a given position (insertion order). They return `0` (and log
`E030`/`E031`/`E048`) if the handle is not an object, or `E032` for an
out-of-range index.

```pawn
new n;
json_object_len(doc, n);
for (new i = 0; i < n; i++)
{
    new key[32];
    json_object_key_at(doc, i, key);
    printf("key[%d] = %s", i, key);
}
```

## json_item

```pawn
native json_item(ptr, index, &out_ptr);   // array only
```

Deep-copies the element at `index` of an **array** handle into a new handle
written to `out_ptr`. Returns `1` on success; `0` for a non-array (`E049`),
missing handle (`E047`) or out-of-range index (`E032`).

!!! warning
    The result is a new handle — free it with `json_free`.

```pawn
new item;
if (json_item(arr, 0, item))
{
    new buf[64];
    json_to_string(item, buf);
    json_free(item);
}
```

## json_at

```pawn
native json_at(ptr, const path[], &out_ptr);
```

Deep-copies the node reachable at `path` into a new handle. Returns `1` on
success; `0` for a missing handle (`E040`) or unresolved path (`E041`). The result
is a new handle — free it.

```pawn
new players;
if (json_at(doc, "server.players", players))
{
    new total;
    json_len(players, total);
    json_free(players);
}
```
