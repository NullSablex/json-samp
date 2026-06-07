// 04_arrays.pwn — work with arrays, both as standalone handles and as object keys.
//
// Two distinct styles appear here:
//   * Array HANDLE: json_create_array + json_array_append_* + json_array_remove.
//   * Array under an object KEY: json_append_array + json_array_len + json_array_get_*.

#include <a_samp>
#include <json_samp>

public OnGameModeInit()
{
    // --- Standalone array handle --------------------------------------------
    new arr;
    json_create_array(arr);

    json_array_append_string(arr, "alpha");
    json_array_append_int(arr, 7);
    json_array_append_float(arr, 3.14);
    json_array_append_bool(arr, true);
    json_array_append_null(arr);
    json_array_append(arr, "{\"nested\":1}");   // raw JSON value

    new len;
    json_len(arr, len);
    printf("array length: %d", len);   // 6

    // Pull an element out as its own handle (deep clone) — remember to free it.
    // Indices: 0 "alpha", 1 7, 2 3.14, 3 true, 4 null, 5 {"nested":1}
    new item;
    if (json_item(arr, 5, item))   // the nested object we appended
    {
        new sub[32];
        json_to_string(item, sub);
        printf("item[5] = %s", sub);
        json_free(item);
    }

    json_array_remove(arr, 4);   // drop the null element (shifts later indices left)

    // Read a scalar element of the array handle through the path API ("0" = index 0).
    new first[16];
    json_get_string_at(arr, "0", first);
    printf("first element: %s", first);

    new dump[160];
    json_to_string(arr, dump);
    printf("array: %s", dump);
    json_free(arr);

    // --- Array stored under an object key (array of objects) ----------------
    new doc;
    json_create(doc);
    json_append_array(doc, "players", "{\"name\":\"Erick\",\"score\":10}");
    json_append_array(doc, "players", "{\"name\":\"Ana\",\"score\":25}");

    new count;
    json_array_len(doc, "players", count);
    printf("players: %d", count);

    for (new i = 0; i < count; i++)
    {
        new pname[24], pscore;
        json_array_get_string(doc, "players", i, "name", pname);
        json_array_get_int(doc, "players", i, "score", pscore);
        printf("  player %d: %s (%d)", i, pname, pscore);
    }

    json_free(doc);
    return 1;
}
