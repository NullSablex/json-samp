// 06_inspect_and_iterate.pwn — inspect node types and iterate object keys.
//
// json_type() returns one of the JSON_TYPE_* constants. json_object_len /
// json_object_key_at let you walk an object's keys in insertion order, and
// json_key_at does the same for the current node handle.

#include <a_samp>
#include <json_samp>

stock TypeName(type)
{
    switch (type)
    {
        case JSON_TYPE_NULL:   return "null";
        case JSON_TYPE_BOOL:   return "bool";
        case JSON_TYPE_NUMBER: return "number";
        case JSON_TYPE_STRING: return "string";
        case JSON_TYPE_ARRAY:  return "array";
        case JSON_TYPE_OBJECT: return "object";
    }
    return "unknown";
}

public OnGameModeInit()
{
    new const src[] = "{\"name\":\"Erick\",\"level\":42,\"items\":[\"sword\",\"shield\"]}";

    new doc;
    if (!json_parse(src, doc))
        return 1;

    new type;
    json_type(doc, type);
    printf("root is %s, is_object=%d, is_array=%d",
        TypeName(type), json_is_object(doc), json_is_array(doc));

    // Iterate every top-level key.
    new keyCount;
    json_object_len(doc, keyCount);
    for (new i = 0; i < keyCount; i++)
    {
        new key[32];
        json_object_key_at(doc, i, key);
        printf("  key[%d] = %s", i, key);
    }

    // Navigate into the array and report its length / element type.
    new items;
    if (json_at(doc, "items", items))   // returns a handle (clone) — must be freed
    {
        new n;
        json_len(items, n);
        printf("items has %d elements", n);
        json_free(items);
    }

    json_free(doc);
    return 1;
}
