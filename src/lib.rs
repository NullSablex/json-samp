use samp::initialize_plugin;

mod logger;
mod natives;
mod plugin;

use plugin::Plugin;

initialize_plugin!(
    natives: [
        Plugin::json_parse,
        Plugin::json_free,
        Plugin::json_log,
        Plugin::json_get_string,
        Plugin::json_get_int,
        Plugin::json_get_float,
        Plugin::json_get_bool,
        Plugin::json_is_valid,
        Plugin::json_has_key,
        Plugin::json_create,
        Plugin::json_set_string,
        Plugin::json_set_int,
        Plugin::json_set_float,
        Plugin::json_set_bool,
        Plugin::json_to_string,

        Plugin::json_open_file,
        Plugin::json_save_file,
        Plugin::json_create_file,
        Plugin::json_reload_file,
        Plugin::json_append_array,
        Plugin::json_delete_key,
        Plugin::json_exists_key,
        Plugin::json_object_len,
        Plugin::json_object_key_at,
        Plugin::json_array_len,
        Plugin::json_array_get_string,
        Plugin::json_array_get_int,
        Plugin::json_array_get_float,
        Plugin::json_array_get_bool,
        Plugin::json_exists_at,
        Plugin::json_get_string_at,
        Plugin::json_get_int_at,
        Plugin::json_get_float_at,
        Plugin::json_get_bool_at,
        Plugin::json_type,
        Plugin::json_len,
        Plugin::json_key_at,
        Plugin::json_item,
        Plugin::json_at,
        Plugin::json_set_at,
        Plugin::json_set_string_at,
        Plugin::json_set_int_at,
        Plugin::json_set_float_at,
        Plugin::json_set_bool_at,
        Plugin::json_set_null_at,
        Plugin::json_delete_at,
        Plugin::json_to_string_pretty,
        Plugin::json_create_array,
        Plugin::json_array_append,
        Plugin::json_array_append_string,
        Plugin::json_array_append_int,
        Plugin::json_array_append_float,
        Plugin::json_array_append_bool,
        Plugin::json_array_append_null,
        Plugin::json_array_remove,
        Plugin::json_clone,
        Plugin::json_clear,
        Plugin::json_merge,
        Plugin::json_set_null,
        Plugin::json_is_array,
        Plugin::json_is_object,
        Plugin::json_count,
        Plugin::json_equals,
    ],
    {
        return Plugin::new();
    }
);
