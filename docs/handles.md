# Handles & conventions

## What is a handle?

Every JSON document lives **inside the plugin**, in a pool indexed by an integer
**handle** (also called `ptr` / `id` in the signatures). Natives that produce a
new document return a handle through an `&out_id` parameter:

| Native | Produces a handle |
|---|---|
| [`json_parse`](reading.md#json_parse) | from a JSON string |
| [`json_create`](building.md#json_create) | empty object |
| [`json_create_array`](arrays.md#json_create_array) | empty array |
| [`json_open_file`](files.md#json_open_file) | from a file |
| [`json_clone`](utilities.md#json_clone) | deep copy of another handle |
| [`json_item`](inspection.md#json_item) | deep copy of an array element |
| [`json_at`](inspection.md#json_at) | deep copy of a node at a path |

!!! warning "Always free your handles"
    A handle stays in the pool until you release it with
    [`json_free`](reading.md#json_free). Handles produced by `json_item`,
    `json_at` and `json_clone` are **independent copies** — freeing or editing
    them never touches the source document. Use
    [`json_count`](utilities.md#json_count) to detect leaks.

```pawn
new doc;
json_parse("{\"a\":1}", doc);
// ... use doc ...
json_free(doc);     // mandatory
```

## Return values

Most natives return `1` on success and `0` on failure. The actual value you ask
for is delivered through an `&output` reference parameter:

```pawn
new level;
if (json_get_int(doc, "level", level))   // returns 1/0
    printf("level = %d", level);         // value is in `level`
```

Predicate natives (`json_has_key`, `json_is_valid`, `json_is_array`,
`json_is_object`, `json_equals`) return the boolean result directly.

## Paths

Every `*_at` native accepts two interchangeable path syntaxes:

- **JSON Pointer** — `"/server/players/0/name"`
- **Friendly** — `"server.players[0].name"`

An empty path (`""`) refers to the document root. See [Paths](paths.md).

## Failure & logging

On failure, natives log a coded message (e.g. `(E002) ID 5 not found ...`) to the
console and `logs/json_samp.log`. The full list is in [Error codes](errors.md).
Tune verbosity with [`json_log`](utilities.md#json_log).
