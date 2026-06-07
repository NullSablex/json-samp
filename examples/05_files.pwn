// 05_files.pwn — persist and load documents from disk.
//
// Relative paths are resolved against the server's scriptfiles/ folder, so
// "data/players.json" means "scriptfiles/data/players.json". Parent
// directories are created automatically on save/create.

#include <a_samp>
#include <json_samp>

#define DATA_FILE "data/players.json"

public OnGameModeInit()
{
    // Ensure the file exists (creates an empty {} if missing; no-op otherwise).
    json_create_file(DATA_FILE);

    // Build a document and save it (pretty-printed).
    new doc;
    json_create(doc);
    json_set_string(doc, "name", "Erick");
    json_set_int(doc, "level", 42);

    if (!json_save_file(doc, DATA_FILE))
        print("[json] save failed (see logs/json_samp.log)");

    json_free(doc);

    // Load it back into a fresh handle.
    new loaded;
    if (json_open_file(DATA_FILE, loaded))
    {
        new name[24];
        json_get_string(loaded, "name", name);
        printf("loaded name=%s", name);

        // Re-read the file from disk into the SAME handle (e.g. after an external
        // edit), discarding in-memory changes.
        json_reload_file(loaded, DATA_FILE);

        json_free(loaded);
    }
    return 1;
}
