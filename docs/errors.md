# Error codes

When a native fails it returns `0` (or the documented sentinel) and logs a coded
message to the console and `logs/json_samp.log`. Codes are stable, so you can grep
logs for them. Tune which severities are emitted with
[`json_log`](utilities.md#json_log).

Severity legend: **error** (hard failure, often with an underlying cause written
to the log file) · **warning** (recoverable / caller mistake, e.g. a missing key
or wrong type).

| Code | Severity | Meaning |
|---|---|---|
| `E001` | error | Failed to parse a JSON string (`json_parse`). |
| `E002` | warning | Handle ID not found (level-1 getters, `json_*_len`, `json_save_file`, `json_reload_file`). |
| `E003` | warning | Key value is missing or not an integer (`json_get_int`). |
| `E004` | warning | Key value is missing or not a float (`json_get_float`). |
| `E005` | warning | Key not found / not convertible to string (`json_get_string`, `json_array_*`). |
| `E006` | warning | Key value is missing or not a bool (`json_get_bool`). |
| `E010` | error | Empty path (`json_open_file`). |
| `E011` | error | Failed to read file (`json_open_file`). |
| `E012` | error | Invalid JSON in file (`json_open_file`). |
| `E013` | error | Empty path (`json_save_file`). |
| `E014` | error | Failed to serialize document (`json_save_file`). |
| `E015` | error | Failed to create parent directories (`json_save_file`). |
| `E016` | error | Failed to write file (`json_save_file`). |
| `E017` | error | Empty path (`json_create_file`). |
| `E018` | error | Failed to create parent directories (`json_create_file`). |
| `E019` | error | Failed to create file (`json_create_file`). |
| `E020` | error | Empty path (`json_reload_file`). |
| `E021` | error | Failed to read file (`json_reload_file`). |
| `E022` | error | Invalid JSON in file (`json_reload_file`). |
| `E023` | error | Invalid `value_json` (`json_append_array`). |
| `E024` | warning | Key exists but is not an array (`json_append_array`). |
| `E030` | warning | Root is not an object (`json_object_len`). |
| `E031` | warning | Root is not an object (`json_object_key_at`). |
| `E032` | warning | Invalid/out-of-range index (`json_object_key_at`, `json_key_at`, `json_item`, `json_array_remove`). |
| `E034` | warning | Key value is not an array (`json_array_len`). |
| `E035` | warning | Key value is not an array (`json_array_get_*`). |
| `E037` | warning | Invalid index or array element is not an object (`json_array_get_*`). |
| `E038` | warning | Field missing in the array element (`json_array_get_*`). |
| `E040` | warning | Handle ID not found (path API). |
| `E041` | warning | Path not found (path getters, `json_at`). |
| `E042` | warning | Value at path is not a string / not serializable (`json_get_string_at`). |
| `E043` | warning | Value at path is not an integer (`json_get_int_at`). |
| `E044` | warning | Value at path is not a float (`json_get_float_at`). |
| `E045` | warning | Value at path is not a bool (`json_get_bool_at`). |
| `E046` | warning | Handle ID not found (`json_type`). |
| `E047` | warning | Handle ID not found (handle-level ops: `json_len`, `json_item`, `json_key_at`, `json_clone`, `json_clear`, `json_array_*`, `json_merge`). |
| `E048` | warning | Node is not an object (`json_key_at`). |
| `E049` | warning | Node is not an array (`json_item`, `json_array_append*`, `json_array_remove`). |
| `E050` | error | Invalid `value_json` (`json_set_at`). |
| `E051` | warning | Parent path not found (`json_set_at`, `json_delete_at`). |
| `E052` | warning | Invalid or out-of-bounds array index in a path (`json_set_at`, `json_delete_at`). |
| `E053` | warning | Parent node is not a container (`json_set_at`, `json_delete_at`). |
| `E054` | warning | Deleting the root is not allowed (`json_delete_at`). |
| `E055` | warning | Non-finite float rejected (`json_set_float_at`, `json_array_append_float`). |
| `E056` | error | Invalid `value_json` (`json_array_append`). |
| `E057` | warning | Node is not a container (`json_clear`). |
| `E058` | warning | Source/destination is not an object (`json_merge`). |

!!! tip "Errors with detail"
    File and parse failures (`E001`, `E011`, `E012`, `E014`–`E016`, `E018`,
    `E019`, `E021`, `E022`, `E023`, `E050`, `E056`) print a concise line to the
    console and the full underlying cause to `logs/json_samp.log`.
