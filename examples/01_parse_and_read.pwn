// 01_parse_and_read.pwn — parse a JSON string and read top-level keys.
//
// json_parse() stores the document in the plugin pool and hands back a handle
// (an integer id) through its &out_id argument. Every handle you obtain must be
// released with json_free() once you are done, or it leaks (see json_count()).

#include <a_samp>
#include <json_samp>

public OnGameModeInit()
{
    new const src[] = "{\"name\":\"Erick\",\"level\":42,\"vip\":true,\"balance\":1500.75}";

    // Cheap validity check that does not allocate a handle.
    if (!json_is_valid(src))
    {
        print("[json] invalid JSON literal");
        return 1;
    }

    new doc;
    if (!json_parse(src, doc))
    {
        print("[json] parse failed (see logs/json_samp.log)");
        return 1;
    }

    new name[24];
    json_get_string(doc, "name", name);   // size defaults to sizeof(name)

    new level, Float:balance, vip;
    json_get_int(doc, "level", level);
    json_get_float(doc, "balance", balance);
    json_get_bool(doc, "vip", vip);

    printf("name=%s level=%d vip=%d balance=%.2f", name, level, vip, balance);

    // Guard reads with json_has_key when a field is optional.
    if (json_has_key(doc, "clan"))
        print("player belongs to a clan");
    else
        print("no clan field present");

    json_free(doc);   // always release the handle
    return 1;
}
