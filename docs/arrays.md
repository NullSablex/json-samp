# Arrays

There are two complementary ways to work with arrays:

- **Array handle** — a document whose root is an array.
- **Array under an object key** — convenience natives for the common
  "array of objects" shape.

## Array handles

### json_create_array

```pawn
native json_create_array(&out_id);
```

Allocates an empty **array** handle. Always returns `1`.

### Appending

```pawn
native json_array_append(ptr, const value_json[]); // raw JSON value
native json_array_append_string(ptr, const value[]);
native json_array_append_int(ptr, value);
native json_array_append_float(ptr, Float:value);
native json_array_append_bool(ptr, value);
native json_array_append_null(ptr);
```

Push onto an array handle. Return `1` on success, `0` if the handle is missing
(`E047`) or is not an array (`E049`). `json_array_append` parses `value_json`
(logs `E056` if invalid).

```pawn
new arr;
json_create_array(arr);
json_array_append_string(arr, "alpha");
json_array_append_int(arr, 7);
json_array_append(arr, "{\"nested\":1}");
```

### json_array_remove

```pawn
native json_array_remove(ptr, index);
```

Removes the element at `index` (shifting later elements left). Returns `1` on
success, `0` on out-of-range/negative index (`E032`), non-array (`E049`) or
missing handle (`E047`).

### Reading array elements

Scalar elements of an array handle are read/written through the
[path API](paths.md), using the index as the path:

```pawn
new first[16];
json_get_string_at(arr, "0", first);   // element 0
json_set_int_at(arr, "1", 99);         // overwrite element 1
```

To pull an element out as its own handle, use [`json_item`](inspection.md#json_item).

## Arrays under an object key

### json_append_array

```pawn
native json_append_array(ptr, const key[], const value_json[]);
```

Appends a JSON value to the array stored at `key` inside an object handle,
creating the array if the key is absent. Returns `1` on success; `0` if the key
exists but is not an array (`E024`) or the handle is not an object.

```pawn
new doc;
json_create(doc);
json_append_array(doc, "players", "{\"name\":\"Erick\",\"score\":10}");
json_append_array(doc, "players", "{\"name\":\"Ana\",\"score\":25}");
```

### json_array_len

```pawn
native json_array_len(ptr, const key[], &out_len);
```

Writes the length of the array at `key` into `out_len`. Returns `1` on success;
`0` if the key is missing (`E005`), not an array (`E034`) or the handle is missing
(`E002`).

### Typed element getters (array of objects)

```pawn
native json_array_get_string(ptr, const key[], index, const field[], output[], size = sizeof(output));
native json_array_get_int(ptr, const key[], index, const field[], &output);
native json_array_get_float(ptr, const key[], index, const field[], &Float:output);
native json_array_get_bool(ptr, const key[], index, const field[], &output);
```

Resolve `ptr[key][index][field]` — i.e. read `field` from the object at position
`index` of the array stored under `key`. Return `1` on success, `0` otherwise.

```pawn
new count;
json_array_len(doc, "players", count);
for (new i = 0; i < count; i++)
{
    new name[24], score;
    json_array_get_string(doc, "players", i, "name", name);
    json_array_get_int(doc, "players", i, "score", score);
    printf("%s: %d", name, score);
}
```
