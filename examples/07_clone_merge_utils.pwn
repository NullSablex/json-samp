// 07_clone_merge_utils.pwn — clone, merge, compare, clear and pool diagnostics.
//
// These helpers operate on whole handles: json_clone (deep copy),
// json_merge (deep merge of two objects), json_equals (structural compare),
// json_clear (empty in place) and json_count (open handles, for leak hunting).

#include <a_samp>
#include <json_samp>

public OnGameModeInit()
{
    // Optional: control verbosity (0=none .. 4=all). Default is JSON_LOG_ALL.
    json_log(JSON_LOG_INFO);

    new base;
    json_create(base);
    json_set_string(base, "name", "Erick");
    json_set_int(base, "level", 1);

    // Deep copy: changes to the clone never touch the original.
    new copy;
    json_clone(base, copy);
    json_set_int(copy, "level", 99);
    printf("equals after edit? %d", json_equals(base, copy));   // 0

    // Deep merge: overlay defaults/overrides onto base.
    new patch;
    json_create(patch);
    json_set_int(patch, "level", 50);
    json_set_bool(patch, "vip", true);
    json_merge(base, patch);   // base now has level=50, vip=true, name=Erick

    new dump[128];
    json_to_string(base, dump);
    printf("merged: %s", dump);

    // Empty a handle while keeping its id valid.
    json_clear(patch);
    new patchLen;
    json_len(patch, patchLen);
    printf("patch length after clear: %d", patchLen);   // 0

    // How many handles are still open right now?
    printf("open handles before free: %d", json_count());

    json_free(base);
    json_free(copy);
    json_free(patch);

    printf("open handles after free: %d", json_count());   // 0
    return 1;
}
