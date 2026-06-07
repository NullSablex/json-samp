# Utilities & diagnostics

## json_clone

```pawn
native json_clone(ptr, &out_id);
```

Deep-copies a handle into a new one written to `out_id`. The copy is fully
independent — editing or freeing it never affects the source. Returns `1` on
success, `0` if the source is missing (`E047`). Free the clone when done.

```pawn
new copy;
json_clone(doc, copy);
json_set_int(copy, "level", 99);   // doc is untouched
json_free(copy);
```

## json_merge

```pawn
native json_merge(dest_ptr, src_ptr);
```

**Deep-merges** the `src` object into the `dest` object: nested objects are merged
key by key, any other value type overwrites. Both handles must be objects.
Merging a handle into itself is a no-op success. Returns `1` on success; `0` if a
side is missing (`E047`) or not an object (`E058`).

```pawn
// dest = {"name":"Erick","level":1}, patch = {"level":50,"vip":true}
json_merge(dest, patch);
// dest -> {"name":"Erick","level":50,"vip":true}
```

## json_clear

```pawn
native json_clear(ptr);
```

Empties an object/array handle **in place**, keeping the same id. Returns `1` on
success; `0` for a scalar (`E057`) or missing handle (`E047`).

```pawn
json_clear(doc);   // {} or []
```

## json_equals

```pawn
native json_equals(ptr_a, ptr_b);
```

Structural deep comparison. Returns `1` when both handles exist and are deeply
equal, `0` otherwise.

```pawn
if (json_equals(a, b))
    print("identical documents");
```

## json_count

```pawn
native json_count();
```

Returns the number of handles currently open in the pool. Useful to catch leaks —
call it before and after a block of work; the delta should return to baseline once
everything is freed.

```pawn
printf("open handles: %d", json_count());
```

## json_log

```pawn
native bool:json_log(log_level);
```

Sets the runtime log level. Always returns `true`.

```pawn
enum {
    JSON_LOG_NONE = 0,
    JSON_LOG_ERROR,
    JSON_LOG_WARNING,
    JSON_LOG_INFO,
    JSON_LOG_ALL,
}
```

```pawn
json_log(JSON_LOG_ERROR);   // only errors reach console / logs/json_samp.log
```
