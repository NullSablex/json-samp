// 02_build_and_serialize.pwn — build an object from scratch and serialize it.
//
// json_create() allocates an empty object handle. The typed json_set_* natives
// write top-level keys; json_to_string / json_to_string_pretty render it back
// to text. The handle must be freed afterwards.

#include <a_samp>
#include <json_samp>

public OnGameModeInit()
{
    new doc;
    json_create(doc);

    json_set_string(doc, "name", "Erick");
    json_set_int(doc, "level", 42);
    json_set_float(doc, "balance", 1500.75);
    json_set_bool(doc, "vip", true);
    json_set_null(doc, "clan");   // explicit JSON null

    new compact[256];
    json_to_string(doc, compact);
    printf("compact: %s", compact);

    new pretty[256];
    json_to_string_pretty(doc, pretty);
    printf("pretty:\n%s", pretty);

    json_free(doc);
    return 1;
}
