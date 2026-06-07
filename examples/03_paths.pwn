// 03_paths.pwn — read and write deep values with the path API.
//
// Paths accept two syntaxes, interchangeably:
//   - JSON Pointer:   "/players/0/name"
//   - friendly:       "players[0].name"
// They work on any nested object/array without allocating intermediate handles.

#include <a_samp>
#include <json_samp>

public OnGameModeInit()
{
    new const src[] = "{\"server\":{\"name\":\"NullSablex\",\"players\":[{\"name\":\"Erick\",\"score\":10}]}}";

    new doc;
    if (!json_parse(src, doc))
        return 1;

    // --- Reading ------------------------------------------------------------
    new sv_name[32];
    json_get_string_at(doc, "server.name", sv_name);

    new score;
    json_get_int_at(doc, "/server/players/0/score", score);   // JSON Pointer form

    printf("server=%s first player score=%d", sv_name, score);

    if (json_exists_at(doc, "server.players[0].name"))
        print("first player has a name");

    // --- Writing (typed, no value_json string needed) -----------------------
    json_set_int_at(doc, "server.players[0].score", 99);
    json_set_string_at(doc, "server.name", "NullSablex RP");
    json_set_bool_at(doc, "server.online", true);   // creates the key

    // --- Writing raw JSON at a path -----------------------------------------
    json_set_at(doc, "server.tags", "[\"rp\",\"survival\"]");

    // --- Deleting -----------------------------------------------------------
    json_delete_at(doc, "server.players[0].score");

    new dump[256];
    json_to_string(doc, dump);
    printf("result: %s", dump);

    json_free(doc);
    return 1;
}
