# API reference

Every native, grouped by topic. The canonical, always-up-to-date signatures live
in [`include/json_samp.inc`](https://github.com/NullSablex/json-samp/blob/master/include/json_samp.inc).

Unless noted otherwise, natives return `1` on success and `0` on failure, and
deliver values through `&output` parameters.

## Parse / free — [details](reading.md)

| Native | Description |
|---|---|
| `json_parse(const json_str[], &out_id)` | Parse a string into a new handle |
| `json_is_valid(const json_str[])` | `1` if the string is valid JSON (no handle) |
| `json_free(ptr)` | Release a handle |

## Logging — [details](utilities.md#json_log)

| Native | Description |
|---|---|
| `bool:json_log(log_level)` | Set log level (`JSON_LOG_*`) |

## Getters by key — [details](reading.md#typed-getters-by-key)

| Native | Description |
|---|---|
| `json_get_string(ptr, key[], output[], size)` | Read a string key |
| `json_get_int(ptr, key[], &output)` | Read an integer key |
| `json_get_float(ptr, key[], &Float:output)` | Read a float key |
| `json_get_bool(ptr, key[], &output)` | Read a bool key |
| `json_has_key(ptr, key[])` | `1` if key exists |
| `json_exists_key(ptr, key[])` | Synonym of `json_has_key` |

## Create / set — [details](building.md)

| Native | Description |
|---|---|
| `json_create(&out_id)` | New empty object |
| `json_create_array(&out_id)` | New empty array |
| `json_set_string(ptr, key[], value[])` | Set a string key |
| `json_set_int(ptr, key[], value)` | Set an integer key |
| `json_set_float(ptr, key[], Float:value)` | Set a float key |
| `json_set_bool(ptr, key[], value)` | Set a bool key |
| `json_set_null(ptr, key[])` | Set a key to JSON null |

## Serialize — [details](building.md#serializing)

| Native | Description |
|---|---|
| `json_to_string(ptr, output[], size)` | Compact serialization |
| `json_to_string_pretty(ptr, output[], size)` | Indented serialization |

## File I/O — [details](files.md)

| Native | Description |
|---|---|
| `json_open_file(path[], &out_id)` | Load + parse a file into a new handle |
| `json_save_file(ptr, path[])` | Save a handle (pretty) to a file |
| `json_create_file(path[])` | Create an empty `{}` file if missing |
| `json_reload_file(ptr, path[])` | Re-read a file into the same handle |

## Arrays — [details](arrays.md)

| Native | Description |
|---|---|
| `json_append_array(ptr, key[], value_json[])` | Append raw JSON to `key`'s array |
| `json_array_len(ptr, key[], &out_len)` | Length of `key`'s array |
| `json_array_get_string(ptr, key[], index, field[], output[], size)` | `key[index].field` as string |
| `json_array_get_int(ptr, key[], index, field[], &output)` | `key[index].field` as int |
| `json_array_get_float(ptr, key[], index, field[], &Float:output)` | `key[index].field` as float |
| `json_array_get_bool(ptr, key[], index, field[], &output)` | `key[index].field` as bool |
| `json_array_append(ptr, value_json[])` | Append raw JSON to an array handle |
| `json_array_append_string(ptr, value[])` | Append a string |
| `json_array_append_int(ptr, value)` | Append an int |
| `json_array_append_float(ptr, Float:value)` | Append a float |
| `json_array_append_bool(ptr, value)` | Append a bool |
| `json_array_append_null(ptr)` | Append null |
| `json_array_remove(ptr, index)` | Remove element at index |

## Path API — [details](paths.md)

| Native | Description |
|---|---|
| `json_exists_at(ptr, path[])` | `1` if the path resolves |
| `json_get_string_at(ptr, path[], output[], size)` | Read string at path |
| `json_get_int_at(ptr, path[], &output)` | Read int at path |
| `json_get_float_at(ptr, path[], &Float:output)` | Read float at path |
| `json_get_bool_at(ptr, path[], &output)` | Read bool at path |
| `json_set_at(ptr, path[], value_json[])` | Set raw JSON at path |
| `json_set_string_at(ptr, path[], value[])` | Set string at path |
| `json_set_int_at(ptr, path[], value)` | Set int at path |
| `json_set_float_at(ptr, path[], Float:value)` | Set float at path |
| `json_set_bool_at(ptr, path[], value)` | Set bool at path |
| `json_set_null_at(ptr, path[])` | Set null at path |
| `json_delete_at(ptr, path[])` | Delete node at path |

## Inspection & navigation — [details](inspection.md)

| Native | Description |
|---|---|
| `json_type(ptr, &out_type)` | Node type (`JSON_TYPE_*`) |
| `json_len(ptr, &out_len)` | Container length, else 0 |
| `json_is_array(ptr)` | `1` if the node is an array |
| `json_is_object(ptr)` | `1` if the node is an object |
| `json_object_len(ptr, &out_len)` | Object key count |
| `json_object_key_at(ptr, index, output[], size)` | Object key by position |
| `json_key_at(ptr, index, output[], size)` | Object key by position (node) |
| `json_item(ptr, index, &out_ptr)` | Array element as a new handle |
| `json_at(ptr, path[], &out_ptr)` | Node at path as a new handle |

## Handle-level operations — [details](utilities.md)

| Native | Description |
|---|---|
| `json_clone(ptr, &out_id)` | Deep copy into a new handle |
| `json_clear(ptr)` | Empty an object/array in place |
| `json_merge(dest_ptr, src_ptr)` | Deep-merge `src` object into `dest` |
| `json_equals(ptr_a, ptr_b)` | Structural deep comparison |
| `json_count()` | Number of open handles |

## Enums

```pawn
enum { JSON_TYPE_NULL = 0, JSON_TYPE_BOOL, JSON_TYPE_NUMBER, JSON_TYPE_STRING, JSON_TYPE_ARRAY, JSON_TYPE_OBJECT }
enum { JSON_LOG_NONE = 0, JSON_LOG_ERROR, JSON_LOG_WARNING, JSON_LOG_INFO, JSON_LOG_ALL }
```

See also: [Error codes](errors.md).
