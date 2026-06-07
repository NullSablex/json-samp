use samp::native;
use samp::prelude::*;
use serde_json::Value;
use std::path::Path;

use crate::logger::Logger;

fn amx_cstr(s: &AmxString) -> String {
    s.to_string().trim_end_matches(char::from(0)).to_string()
}

/// Converts a Pawn index into a slice index. Negative values are rejected so
/// callers can treat `None` as "out of range".
fn to_index(index: i32) -> Option<usize> {
    usize::try_from(index).ok()
}

/// Prefixes "scriptfiles/" when the path is not absolute or explicit.
fn resolve_scriptfiles_path(raw: &str) -> String {
    let p = raw.replace('\\', "/").trim().to_string();
    if p.is_empty() {
        return p;
    }
    let is_absolute = Path::new(&p).is_absolute() || (p.len() > 1 && p.as_bytes()[1] == b':');
    let explicit_root = p.starts_with("./") || p.starts_with("../");
    let explicit_scriptfiles = p.to_ascii_lowercase().starts_with("scriptfiles/");
    if is_absolute || explicit_root || explicit_scriptfiles {
        p
    } else {
        format!("scriptfiles/{}", p)
    }
}

// ---------- Path helpers (JSON Pointer and friendly "a.b[0].c" syntax) ----------

fn unescape_pointer_token(tok: &str) -> String {
    tok.replace("~1", "/").replace("~0", "~")
}

fn tokens_from_pointer(path: &str) -> Vec<String> {
    if path.is_empty() || path == "/" {
        return Vec::new();
    }
    path.split('/')
        .skip(1) // skip the leading empty segment
        .map(unescape_pointer_token)
        .collect()
}

// Converts "radios[0].genres[1]" into tokens ["radios","0","genres","1"]
fn tokens_from_friendly(path: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = path.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '.' => {
                if !cur.is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
            }
            '[' => {
                if !cur.is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
                i += 1;
                let mut idx = String::new();
                while i < chars.len() && chars[i] != ']' {
                    idx.push(chars[i]);
                    i += 1;
                }
                tokens.push(idx);
            }
            _ => cur.push(ch),
        }
        i += 1;
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

fn normalize_tokens(path: &str) -> Vec<String> {
    let p = path.trim();
    if p.starts_with('/') {
        tokens_from_pointer(p)
    } else {
        tokens_from_friendly(p)
    }
}

// Immutable lookup by tokens
fn get_at<'a>(root: &'a Value, tokens: &[String]) -> Option<&'a Value> {
    let mut cur = root;
    for t in tokens {
        match cur {
            Value::Object(map) => {
                cur = map.get(t)?;
            }
            Value::Array(arr) => {
                let idx: usize = t.parse().ok()?;
                cur = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

// ---------- Value inspection / coercion ----------

/// Whether `s` is syntactically valid JSON.
fn is_valid_json(s: &str) -> bool {
    serde_json::from_str::<Value>(s).is_ok()
}

/// Numeric type tag returned by `json_type`.
fn value_type_code(v: &Value) -> i32 {
    match v {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 2,
        Value::String(_) => 3,
        Value::Array(_) => 4,
        Value::Object(_) => 5,
    }
}

/// Length of a container value; 0 for scalars (matches `json_len`).
fn value_len(v: &Value) -> i32 {
    match v {
        Value::Array(a) => a.len() as i32,
        Value::Object(m) => m.len() as i32,
        _ => 0,
    }
}

/// Coerces a value to i64. Numbers and numeric strings always count; booleans
/// only when `allow_bool` is set (path-based getters accept them, level-1 ones
/// do not).
fn value_as_i64(v: &Value, allow_bool: bool) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse::<i64>().ok(),
        Value::Bool(b) if allow_bool => Some(i64::from(*b)),
        _ => None,
    }
}

/// Coerces a value to f64. See [`value_as_i64`] for the `allow_bool` rule.
fn value_as_f64(v: &Value, allow_bool: bool) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        Value::Bool(b) if allow_bool => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Renders a value as text: strings are returned verbatim, everything else is
/// serialized back to JSON. `None` only if serialization fails.
fn value_as_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        _ => serde_json::to_string(v).ok(),
    }
}

// Walks down to the parent of the last token
fn get_parent_mut<'a>(root: &'a mut Value, tokens: &[String]) -> Option<(&'a mut Value, String)> {
    if tokens.is_empty() {
        return Some((root, String::new()));
    }
    let mut cur = root;
    for t in &tokens[..tokens.len() - 1] {
        match cur {
            Value::Object(map) => {
                cur = map.get_mut(t)?;
            }
            Value::Array(arr) => {
                let idx: usize = t.parse().ok()?;
                cur = arr.get_mut(idx)?;
            }
            _ => return None,
        }
    }
    Some((cur, tokens.last().unwrap().clone()))
}

impl crate::Plugin {
    // ---------- ID/pool helpers ----------

    fn alloc_id(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn alloc_id_and_insert(&mut self, val: Value) -> i32 {
        let id = self.alloc_id();
        self.pool.insert(id, val);
        id
    }

    // ---------- Pure cores (no AMX types; unit-tested directly) ----------

    fn create_object(&mut self) -> i32 {
        self.alloc_id_and_insert(Value::Object(serde_json::Map::new()))
    }

    fn create_array(&mut self) -> i32 {
        self.alloc_id_and_insert(Value::Array(Vec::new()))
    }

    fn free_doc(&mut self, id: i32) -> bool {
        self.pool.remove(&id).is_some()
    }

    fn has_key(&self, id: i32, key: &str) -> bool {
        self.pool
            .get(&id)
            .and_then(|v| v.as_object())
            .is_some_and(|m| m.contains_key(key))
    }

    /// Inserts `val` under `key` when `id` is an object. Returns false when the
    /// handle is missing or is not an object.
    fn set_key(&mut self, id: i32, key: String, val: Value) -> bool {
        match self.pool.get_mut(&id).and_then(|v| v.as_object_mut()) {
            Some(map) => {
                map.insert(key, val);
                true
            }
            None => false,
        }
    }

    fn delete_key(&mut self, id: i32, key: &str) -> bool {
        self.pool
            .get_mut(&id)
            .and_then(|v| v.as_object_mut())
            .is_some_and(|m| m.remove(key).is_some())
    }

    /// Appends `val` to the array stored at `key` in an object handle, creating
    /// the array if the key is absent.
    fn append_array_key(&mut self, id: i32, key: &str, val: Value) -> AppendKeyOutcome {
        let Some(map) = self.pool.get_mut(&id).and_then(|v| v.as_object_mut()) else {
            return AppendKeyOutcome::NotObject;
        };
        match map.get_mut(key) {
            Some(existing) => match existing.as_array_mut() {
                Some(arr) => {
                    arr.push(val);
                    AppendKeyOutcome::Done
                }
                None => AppendKeyOutcome::KeyNotArray,
            },
            None => {
                map.insert(key.to_string(), Value::Array(vec![val]));
                AppendKeyOutcome::Done
            }
        }
    }

    fn to_text_doc(&self, id: i32, pretty: bool) -> Option<String> {
        let v = self.pool.get(&id)?;
        if pretty {
            serde_json::to_string_pretty(v).ok()
        } else {
            serde_json::to_string(v).ok()
        }
    }

    /// Length of an object handle, or `None` when missing/not an object.
    fn object_len(&self, id: i32) -> Option<usize> {
        self.pool.get(&id)?.as_object().map(serde_json::Map::len)
    }

    /// Nth key of an object handle (insertion order), or `None`.
    fn key_at_doc(&self, id: i32, index: i32) -> Option<String> {
        let map = self.pool.get(&id)?.as_object()?;
        to_index(index).and_then(|i| map.keys().nth(i)).cloned()
    }

    /// Length of the array stored at `key` inside an object handle.
    fn array_len_key(&self, id: i32, key: &str) -> Option<usize> {
        match self.pool.get(&id)?.as_object()?.get(key) {
            Some(Value::Array(a)) => Some(a.len()),
            _ => None,
        }
    }

    /// Deep-clones the element at `index` of an array handle.
    fn item_clone(&self, id: i32, index: i32) -> Option<Value> {
        let arr = self.pool.get(&id)?.as_array()?;
        to_index(index).and_then(|i| arr.get(i)).cloned()
    }

    /// Deep-clones the node reachable at `path` from handle `id`.
    fn at_clone(&self, id: i32, path: &str) -> Option<Value> {
        let root = self.pool.get(&id)?;
        get_at(root, &normalize_tokens(path)).cloned()
    }

    fn is_array_doc(&self, id: i32) -> bool {
        matches!(self.pool.get(&id), Some(Value::Array(_)))
    }

    fn is_object_doc(&self, id: i32) -> bool {
        matches!(self.pool.get(&id), Some(Value::Object(_)))
    }

    fn equals_docs(&self, id_a: i32, id_b: i32) -> bool {
        matches!(
            (self.pool.get(&id_a), self.pool.get(&id_b)),
            (Some(a), Some(b)) if a == b
        )
    }

    /// Deep-copies a handle, returning the new id. `None` if the source is gone.
    fn clone_doc(&mut self, id: i32) -> Option<i32> {
        let cloned = self.pool.get(&id)?.clone();
        Some(self.alloc_id_and_insert(cloned))
    }

    /// Empties an object/array handle in place. `Err` carries why it failed.
    fn clear_doc(&mut self, id: i32) -> Result<(), DocErr> {
        match self.pool.get_mut(&id) {
            Some(Value::Object(map)) => {
                map.clear();
                Ok(())
            }
            Some(Value::Array(arr)) => {
                arr.clear();
                Ok(())
            }
            Some(_) => Err(DocErr::WrongType),
            None => Err(DocErr::NotFound),
        }
    }

    /// Pushes `val` onto an array handle.
    fn array_append_handle(&mut self, id: i32, val: Value) -> Result<(), DocErr> {
        match self.pool.get_mut(&id) {
            Some(Value::Array(arr)) => {
                arr.push(val);
                Ok(())
            }
            Some(_) => Err(DocErr::WrongType),
            None => Err(DocErr::NotFound),
        }
    }

    /// Pushes `val` onto an array handle and maps the outcome to a native
    /// return code, logging the reason on failure. Shared by every
    /// `json_array_append*` native.
    fn append_native(&mut self, id: i32, val: Value) -> AmxResult<i32> {
        match self.array_append_handle(id, val) {
            Ok(()) => Ok(1),
            Err(DocErr::WrongType) => {
                Logger::warn(&format!("(E049) Node is not an array in json_array_append (ID {})", id));
                Ok(0)
            }
            Err(DocErr::NotFound) => {
                Logger::warn(&format!("(E047) ID {} not found in json_array_append", id));
                Ok(0)
            }
        }
    }

    /// Removes element `index` from an array handle.
    fn array_remove_handle(&mut self, id: i32, index: i32) -> Result<bool, DocErr> {
        match self.pool.get_mut(&id) {
            Some(Value::Array(arr)) => match to_index(index) {
                Some(i) if i < arr.len() => {
                    arr.remove(i);
                    Ok(true)
                }
                _ => Ok(false),
            },
            Some(_) => Err(DocErr::WrongType),
            None => Err(DocErr::NotFound),
        }
    }

    /// Deep-merges object handle `src_id` into object handle `dest_id`.
    fn merge_docs(&mut self, dest_id: i32, src_id: i32) -> Result<(), MergeErr> {
        if dest_id == src_id {
            return Ok(());
        }
        let src = match self.pool.get(&src_id) {
            Some(v @ Value::Object(_)) => v.clone(),
            Some(_) => return Err(MergeErr::SrcNotObject),
            None => return Err(MergeErr::SrcNotFound),
        };
        match self.pool.get_mut(&dest_id) {
            Some(dest @ Value::Object(_)) => {
                merge_value(dest, src);
                Ok(())
            }
            Some(_) => Err(MergeErr::DestNotObject),
            None => Err(MergeErr::DestNotFound),
        }
    }

    /// Removes the node at `path`. Returns 1 on success, 0 on any failure
    /// (logging the reason). Shared by `json_delete_at`.
    fn delete_path(&mut self, id: i32, path_str: &str) -> i32 {
        let tokens = normalize_tokens(path_str);
        let root = match self.pool.get_mut(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E040) ID {} not found in json_delete_at", id));
                return 0;
            }
        };
        if tokens.is_empty() {
            Logger::warn("(E054) Deleting the root is not allowed in json_delete_at");
            return 0;
        }
        let (parent, last) = match get_parent_mut(root, &tokens) {
            Some(t) => t,
            None => {
                Logger::warn(&format!("(E051) Parent path of '{}' not found", path_str));
                return 0;
            }
        };
        match parent {
            Value::Object(map) => i32::from(map.remove(&last).is_some()),
            Value::Array(arr) => {
                let idx: usize = match last.parse() {
                    Ok(n) => n,
                    Err(_) => {
                        Logger::warn(&format!("(E052) Invalid index '{}' in '{}'", last, path_str));
                        return 0;
                    }
                };
                if idx >= arr.len() {
                    return 0;
                }
                arr.remove(idx);
                1
            }
            _ => {
                Logger::warn(&format!("(E053) Parent node is not a container in '{}'", path_str));
                0
            }
        }
    }

    // ----------------------- PARSE / FREE -----------------------
    #[native(name = "json_parse")]
    pub fn json_parse(
        &mut self,
        _: &Amx,
        json_str: AmxString,
        mut id_out: Ref<i32>,
    ) -> AmxResult<i32> {
        let input = amx_cstr(&json_str);
        match serde_json::from_str::<Value>(&input) {
            Ok(val) => {
                let id = self.alloc_id_and_insert(val);
                *id_out = id;
                Ok(1)
            }
            Err(e) => {
                Logger::error_detail(
                    "(E001) Failed to parse JSON (see log file for details)",
                    &format!("(E001) Failed to parse JSON: {}", e),
                );
                Ok(0)
            }
        }
    }

    #[native(name = "json_free")]
    pub fn json_free(&mut self, _: &Amx, id: i32) -> AmxResult<i32> {
        Ok(self.free_doc(id) as i32)
    }

    // ----------------------- LOGGING -----------------------

    /// Sets the runtime log level (0=none, 1=error, 2=warning, 3=info, 4=all).
    #[native(name = "json_log")]
    pub fn json_log(&mut self, _: &Amx, log_level: i32) -> AmxResult<bool> {
        Logger::set_log_level(log_level);
        Ok(true)
    }

    // ----------------------- GETTERS (level 1, by key) -----------------------

    #[native(name = "json_get_int")]
    pub fn json_get_int(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        mut out: Ref<i32>,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let v = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E002) ID {} not found for json_get_int", id));
                return Ok(0);
            }
        };
        let val_opt = v
            .as_object()
            .and_then(|m| m.get(&key_str))
            .and_then(|vv| value_as_i64(vv, false));
        match val_opt {
            Some(i) => {
                *out = i as i32;
                Ok(1)
            }
            None => {
                Logger::warn(&format!(
                    "(E003) Key '{}' is invalid or its value is not an integer",
                    key_str
                ));
                Ok(0)
            }
        }
    }

    #[native(name = "json_get_float")]
    pub fn json_get_float(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        mut out: Ref<f32>,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let v = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E002) ID {} not found for json_get_float", id));
                return Ok(0);
            }
        };
        let val_opt = v
            .as_object()
            .and_then(|m| m.get(&key_str))
            .and_then(|vv| value_as_f64(vv, false));
        match val_opt {
            Some(f) => {
                *out = f as f32;
                Ok(1)
            }
            None => {
                Logger::warn(&format!(
                    "(E004) Key '{}' is invalid or its value is not a float",
                    key_str
                ));
                Ok(0)
            }
        }
    }

    #[native(name = "json_get_bool")]
    pub fn json_get_bool(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        mut out: Ref<i32>,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let v = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E002) ID {} not found for json_get_bool", id));
                return Ok(0);
            }
        };
        let b = v
            .as_object()
            .and_then(|m| m.get(&key_str))
            .and_then(|vv| vv.as_bool());
        match b {
            Some(val) => {
                *out = if val { 1 } else { 0 };
                Ok(1)
            }
            None => {
                Logger::warn(&format!(
                    "(E006) Key '{}' is invalid or its value is not a bool",
                    key_str
                ));
                Ok(0)
            }
        }
    }

    #[native(name = "json_get_string")]
    pub fn json_get_string(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        output: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E002) ID {} not found for json_get_string", id));
                return Ok(0);
            }
        };
        let text = match root.as_object().and_then(|m| m.get(&key_str)) {
            Some(v) => match value_as_text(v) {
                Some(s) => s,
                None => {
                    Logger::warn(&format!(
                        "(E005) Key '{}' is invalid or not convertible to string",
                        key_str
                    ));
                    return Ok(0);
                }
            },
            None => {
                Logger::warn(&format!("(E005) Key '{}' not found in json_get_string", key_str));
                return Ok(0);
            }
        };
        output.write_str(size, &text)?;
        Ok(1)
    }

    #[native(name = "json_is_valid")]
    pub fn json_is_valid(&mut self, _: &Amx, json_str: AmxString) -> AmxResult<i32> {
        Ok(is_valid_json(&amx_cstr(&json_str)) as i32)
    }

    #[native(name = "json_has_key")]
    pub fn json_has_key(&mut self, _: &Amx, id: i32, key: AmxString) -> AmxResult<i32> {
        Ok(self.has_key(id, &amx_cstr(&key)) as i32)
    }

    // ----------------------- CREATE / SETTERS -----------------------

    #[native(name = "json_create")]
    pub fn json_create(&mut self, _: &Amx, mut out: Ref<i32>) -> AmxResult<i32> {
        *out = self.create_object();
        Ok(1)
    }

    #[native(name = "json_set_string")]
    pub fn json_set_string(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        value: AmxString,
    ) -> AmxResult<i32> {
        Ok(self.set_key(id, amx_cstr(&key), Value::String(amx_cstr(&value))) as i32)
    }

    #[native(name = "json_set_bool")]
    pub fn json_set_bool(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        value: i32,
    ) -> AmxResult<i32> {
        Ok(self.set_key(id, amx_cstr(&key), Value::Bool(value != 0)) as i32)
    }

    #[native(name = "json_set_int")]
    pub fn json_set_int(&mut self, _: &Amx, id: i32, key: AmxString, value: i32) -> AmxResult<i32> {
        let v = Value::Number(serde_json::Number::from(value));
        Ok(self.set_key(id, amx_cstr(&key), v) as i32)
    }

    #[native(name = "json_set_float")]
    pub fn json_set_float(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        value: f32,
    ) -> AmxResult<i32> {
        let num = match serde_json::Number::from_f64(value as f64) {
            Some(n) => n,
            None => return Ok(0),
        };
        Ok(self.set_key(id, amx_cstr(&key), Value::Number(num)) as i32)
    }

    #[native(name = "json_to_string")]
    pub fn json_to_string(
        &mut self,
        _: &Amx,
        id: i32,
        output: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<i32> {
        match self.to_text_doc(id, false) {
            Some(json_str) => {
                output.write_str(size, &json_str)?;
                Ok(1)
            }
            None => Ok(0),
        }
    }

    // ----------------------- FILE I/O -----------------------

    #[native(name = "json_open_file")]
    pub fn json_open_file(
        &mut self,
        _: &Amx,
        path: AmxString,
        mut out_id: Ref<i32>,
    ) -> AmxResult<i32> {
        let raw = amx_cstr(&path);
        let path_str = resolve_scriptfiles_path(&raw);
        if path_str.is_empty() {
            Logger::error("(E010) Empty path in json_open_file");
            return Ok(0);
        }
        match read_json_file(&path_str) {
            Ok(value) => {
                *out_id = self.alloc_id_and_insert(value);
                Ok(1)
            }
            Err(FileLoadErr::Io(e)) => {
                Logger::error_detail(&format!("(E011) Failed to read '{}' (from '{}')", path_str, raw), &format!("(E011) Failed to read '{}' (from '{}'): {}", path_str, raw, e));
                Ok(0)
            }
            Err(FileLoadErr::Parse(e)) => {
                Logger::error_detail(&format!("(E012) Invalid JSON in '{}' (from '{}')", path_str, raw), &format!("(E012) Invalid JSON in '{}' (from '{}'): {}", path_str, raw, e));
                Ok(0)
            }
        }
    }

    #[native(name = "json_save_file")]
    pub fn json_save_file(&mut self, _: &Amx, id: i32, path: AmxString) -> AmxResult<i32> {
        let raw = amx_cstr(&path);
        let path_str = resolve_scriptfiles_path(&raw);
        if path_str.is_empty() {
            Logger::error("(E013) Empty path in json_save_file");
            return Ok(0);
        }
        let value = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E002) ID {} not found in json_save_file", id));
                return Ok(0);
            }
        };
        match write_json_file(&path_str, value) {
            Ok(()) => Ok(1),
            Err(FileSaveErr::Serialize(e)) => {
                Logger::error_detail(&format!("(E014) Failed to serialize JSON of ID {}", id), &format!("(E014) Failed to serialize JSON of ID {}: {}", id, e));
                Ok(0)
            }
            Err(FileSaveErr::Mkdir(e)) => {
                Logger::error_detail(&format!("(E015) Failed to create directories for '{}'", path_str), &format!("(E015) Failed to create directories for '{}': {}", path_str, e));
                Ok(0)
            }
            Err(FileSaveErr::Write(e)) => {
                Logger::error_detail(&format!("(E016) Failed to write '{}' (from '{}')", path_str, raw), &format!("(E016) Failed to write '{}' (from '{}'): {}", path_str, raw, e));
                Ok(0)
            }
        }
    }

    #[native(name = "json_create_file")]
    pub fn json_create_file(&mut self, _: &Amx, path: AmxString) -> AmxResult<i32> {
        let raw = amx_cstr(&path);
        let path_str = resolve_scriptfiles_path(&raw);
        if path_str.is_empty() {
            Logger::error("(E017) Empty path in json_create_file");
            return Ok(0);
        }
        match create_empty_json_file(&path_str) {
            Ok(_) => Ok(1),
            Err(FileCreateErr::Mkdir(e)) => {
                Logger::error_detail(&format!("(E018) Failed to create directories for '{}'", path_str), &format!("(E018) Failed to create directories for '{}': {}", path_str, e));
                Ok(0)
            }
            Err(FileCreateErr::Write(e)) => {
                Logger::error_detail(&format!("(E019) Failed to create '{}' (from '{}')", path_str, raw), &format!("(E019) Failed to create '{}' (from '{}'): {}", path_str, raw, e));
                Ok(0)
            }
        }
    }

    #[native(name = "json_reload_file")]
    pub fn json_reload_file(&mut self, _: &Amx, id: i32, path: AmxString) -> AmxResult<i32> {
        let raw = amx_cstr(&path);
        let path_str = resolve_scriptfiles_path(&raw);
        if path_str.is_empty() {
            Logger::error("(E020) Empty path in json_reload_file");
            return Ok(0);
        }
        if !self.pool.contains_key(&id) {
            Logger::warn(&format!("(E002) ID {} not found in json_reload_file", id));
            return Ok(0);
        }
        match read_json_file(&path_str) {
            Ok(value) => {
                self.pool.insert(id, value);
                Ok(1)
            }
            Err(FileLoadErr::Io(e)) => {
                Logger::error_detail(&format!("(E021) Failed to read '{}' (from '{}')", path_str, raw), &format!("(E021) Failed to read '{}' (from '{}'): {}", path_str, raw, e));
                Ok(0)
            }
            Err(FileLoadErr::Parse(e)) => {
                Logger::error_detail(&format!("(E022) Invalid JSON in '{}' (from '{}')", path_str, raw), &format!("(E022) Invalid JSON in '{}' (from '{}'): {}", path_str, raw, e));
                Ok(0)
            }
        }
    }

    // ----------------------- ARRAYS / OBJECT (level 1) -----------------------

    #[native(name = "json_append_array")]
    pub fn json_append_array(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        value_json: AmxString,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let val_str = amx_cstr(&value_json);
        let val: Value = match serde_json::from_str(&val_str) {
            Ok(v) => v,
            Err(e) => {
                Logger::error_detail(&format!("(E023) Invalid value_json for '{}'", key_str), &format!("(E023) Invalid value_json for '{}': {}", key_str, e));
                return Ok(0);
            }
        };
        match self.append_array_key(id, &key_str, val) {
            AppendKeyOutcome::Done => Ok(1),
            AppendKeyOutcome::KeyNotArray => {
                Logger::warn(&format!("(E024) '{}' is not an array in ID {}", key_str, id));
                Ok(0)
            }
            AppendKeyOutcome::NotObject => Ok(0),
        }
    }

    #[native(name = "json_delete_key")]
    pub fn json_delete_key(&mut self, _: &Amx, id: i32, key: AmxString) -> AmxResult<i32> {
        Ok(self.delete_key(id, &amx_cstr(&key)) as i32)
    }

    #[native(name = "json_exists_key")]
    pub fn json_exists_key(&mut self, _: &Amx, id: i32, key: AmxString) -> AmxResult<i32> {
        Ok(self.has_key(id, &amx_cstr(&key)) as i32)
    }

    #[native(name = "json_object_len")]
    pub fn json_object_len(&mut self, _: &Amx, id: i32, mut out_len: Ref<i32>) -> AmxResult<i32> {
        match self.object_len(id) {
            Some(len) => {
                *out_len = len as i32;
                Ok(1)
            }
            None if self.pool.contains_key(&id) => {
                Logger::warn("(E030) Root is not an object in json_object_len");
                Ok(0)
            }
            None => {
                Logger::warn(&format!("(E002) ID {} not found in json_object_len", id));
                Ok(0)
            }
        }
    }

    #[native(name = "json_object_key_at")]
    pub fn json_object_key_at(
        &mut self,
        _: &Amx,
        id: i32,
        index: i32,
        output: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<i32> {
        match self.key_at_doc(id, index) {
            Some(key) => {
                output.write_str(size, &key)?;
                Ok(1)
            }
            None if !self.is_object_doc(id) => {
                Logger::warn("(E031) Root is not an object in json_object_key_at");
                Ok(0)
            }
            None => {
                Logger::warn(&format!("(E032) Invalid index {} in json_object_key_at", index));
                Ok(0)
            }
        }
    }

    #[native(name = "json_array_len")]
    pub fn json_array_len(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        mut out_len: Ref<i32>,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        if let Some(len) = self.array_len_key(id, &key_str) {
            *out_len = len as i32;
            return Ok(1);
        }
        // Distinguish the failure for a useful log line.
        match self.pool.get(&id) {
            None => Logger::warn(&format!("(E002) ID {} not found in json_array_len", id)),
            Some(v) => match v.as_object().and_then(|m| m.get(&key_str)) {
                Some(_) => {
                    Logger::warn(&format!("(E034) '{}' is not an array in json_array_len", key_str))
                }
                None => {
                    Logger::warn(&format!("(E005) Key '{}' not found in json_array_len", key_str))
                }
            },
        }
        Ok(0)
    }

    #[native(name = "json_array_get_string")]
    #[allow(clippy::too_many_arguments)]
    pub fn json_array_get_string(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        index: i32,
        field: AmxString,
        output: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let field_str = amx_cstr(&field);
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E002) ID {} not found in json_array_get_string", id));
                return Ok(0);
            }
        };
        let arr = match root.as_object().and_then(|m| m.get(&key_str)) {
            Some(Value::Array(a)) => a,
            Some(_) => {
                Logger::warn(&format!("(E035) '{}' is not an array in json_array_get_string", key_str));
                return Ok(0);
            }
            None => {
                Logger::warn(&format!("(E005) Key '{}' not found in json_array_get_string", key_str));
                return Ok(0);
            }
        };
        let item = to_index(index).and_then(|idx| arr.get(idx));
        let field_val = match item {
            Some(Value::Object(map)) => map.get(&field_str),
            _ => {
                Logger::warn(&format!("(E037) Invalid index {} or missing object in '{}'", index, key_str));
                return Ok(0);
            }
        };
        let text = match field_val {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            _ => {
                Logger::warn(&format!("(E038) Field '{}' missing or not a string in {}[{}]", field_str, key_str, index));
                return Ok(0);
            }
        };
        output.write_str(size, &text)?;
        Ok(1)
    }

    // ======================= GENERIC PATH-BASED API =======================

    #[native(name = "json_exists_at")]
    pub fn json_exists_at(&mut self, _: &Amx, id: i32, path: AmxString) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let tokens = normalize_tokens(&path_str);
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E040) ID {} not found in json_exists_at", id));
                return Ok(0);
            }
        };
        Ok(if get_at(root, &tokens).is_some() { 1 } else { 0 })
    }

    #[native(name = "json_get_string_at")]
    pub fn json_get_string_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        output: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let tokens = normalize_tokens(&path_str);
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E040) ID {} not found in json_get_string_at", id));
                return Ok(0);
            }
        };
        let node = match get_at(root, &tokens) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E041) Path '{}' not found", path_str));
                return Ok(0);
            }
        };
        let text = match value_as_text(node) {
            Some(s) => s,
            None => {
                Logger::warn(&format!("(E042) Value at '{}' is not a string/serializable", path_str));
                return Ok(0);
            }
        };
        output.write_str(size, &text)?;
        Ok(1)
    }

    #[native(name = "json_get_int_at")]
    pub fn json_get_int_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        mut out: Ref<i32>,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let tokens = normalize_tokens(&path_str);
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E040) ID {} not found in json_get_int_at", id));
                return Ok(0);
            }
        };
        let node = match get_at(root, &tokens) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E041) Path '{}' not found", path_str));
                return Ok(0);
            }
        };
        match value_as_i64(node, true) {
            Some(i) => {
                *out = i as i32;
                Ok(1)
            }
            None => {
                Logger::warn(&format!("(E043) Value at '{}' is not an integer", path_str));
                Ok(0)
            }
        }
    }

    #[native(name = "json_get_float_at")]
    pub fn json_get_float_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        mut out: Ref<f32>,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let tokens = normalize_tokens(&path_str);
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E040) ID {} not found in json_get_float_at", id));
                return Ok(0);
            }
        };
        let node = match get_at(root, &tokens) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E041) Path '{}' not found", path_str));
                return Ok(0);
            }
        };
        match value_as_f64(node, true) {
            Some(f) => {
                *out = f as f32;
                Ok(1)
            }
            None => {
                Logger::warn(&format!("(E044) Value at '{}' is not a float", path_str));
                Ok(0)
            }
        }
    }

    #[native(name = "json_get_bool_at")]
    pub fn json_get_bool_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        mut out: Ref<i32>,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let tokens = normalize_tokens(&path_str);
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E040) ID {} not found in json_get_bool_at", id));
                return Ok(0);
            }
        };
        let node = match get_at(root, &tokens) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E041) Path '{}' not found", path_str));
                return Ok(0);
            }
        };
        match node.as_bool() {
            Some(b) => {
                *out = if b { 1 } else { 0 };
                Ok(1)
            }
            None => {
                Logger::warn(&format!("(E045) Value at '{}' is not a bool", path_str));
                Ok(0)
            }
        }
    }

    #[native(name = "json_type")]
    pub fn json_type(&mut self, _: &Amx, id: i32, mut out_type: Ref<i32>) -> AmxResult<i32> {
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E046) ID {} not found in json_type", id));
                return Ok(0);
            }
        };
        *out_type = value_type_code(root);
        Ok(1)
    }

    #[native(name = "json_len")]
    pub fn json_len(&mut self, _: &Amx, id: i32, mut out_len: Ref<i32>) -> AmxResult<i32> {
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E047) ID {} not found in json_len", id));
                return Ok(0);
            }
        };
        *out_len = value_len(root);
        Ok(1)
    }

    #[native(name = "json_key_at")]
    pub fn json_key_at(
        &mut self,
        _: &Amx,
        id: i32,
        index: i32,
        output: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<i32> {
        match self.key_at_doc(id, index) {
            Some(key) => {
                output.write_str(size, &key)?;
                Ok(1)
            }
            None if !self.is_object_doc(id) => {
                if self.pool.contains_key(&id) {
                    Logger::warn("(E048) Node is not an object in json_key_at");
                } else {
                    Logger::warn(&format!("(E047) ID {} not found in json_key_at", id));
                }
                Ok(0)
            }
            None => {
                Logger::warn(&format!("(E032) Invalid index {} in json_key_at", index));
                Ok(0)
            }
        }
    }

    #[native(name = "json_item")]
    pub fn json_item(
        &mut self,
        _: &Amx,
        id: i32,
        index: i32,
        mut out_id: Ref<i32>,
    ) -> AmxResult<i32> {
        match self.item_clone(id, index) {
            Some(node) => {
                *out_id = self.alloc_id_and_insert(node);
                Ok(1)
            }
            None if !self.is_array_doc(id) => {
                if self.pool.contains_key(&id) {
                    Logger::warn("(E049) Node is not an array in json_item");
                } else {
                    Logger::warn(&format!("(E047) ID {} not found in json_item", id));
                }
                Ok(0)
            }
            None => {
                Logger::warn(&format!("(E032) Invalid index {} in json_item", index));
                Ok(0)
            }
        }
    }

    #[native(name = "json_at")]
    pub fn json_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        mut out_id: Ref<i32>,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        match self.at_clone(id, &path_str) {
            Some(node) => {
                *out_id = self.alloc_id_and_insert(node);
                Ok(1)
            }
            None if !self.pool.contains_key(&id) => {
                Logger::warn(&format!("(E040) ID {} not found in json_at", id));
                Ok(0)
            }
            None => {
                Logger::warn(&format!("(E041) Path '{}' not found", path_str));
                Ok(0)
            }
        }
    }

    /// Writes `new_val` at `path_str` inside document `id`. Returns 1 on
    /// success, 0 on failure (logging the reason). Shared by `json_set_at` and
    /// the typed `json_set_*_at` natives.
    fn set_path_value(&mut self, id: i32, path_str: &str, new_val: Value) -> i32 {
        let tokens = normalize_tokens(path_str);
        let root = match self.pool.get_mut(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E040) ID {} not found in json_set_at", id));
                return 0;
            }
        };
        if tokens.is_empty() {
            *root = new_val;
            return 1;
        }
        let (parent, last) = match get_parent_mut(root, &tokens) {
            Some(t) => t,
            None => {
                Logger::warn(&format!("(E051) Parent path of '{}' not found", path_str));
                return 0;
            }
        };
        match parent {
            Value::Object(map) => {
                map.insert(last, new_val);
                1
            }
            Value::Array(arr) => {
                let idx: usize = match last.parse() {
                    Ok(n) => n,
                    Err(_) => {
                        Logger::warn(&format!("(E052) Invalid index '{}' in '{}'", last, path_str));
                        return 0;
                    }
                };
                if idx >= arr.len() {
                    Logger::warn(&format!("(E052) Index {} out of bounds in '{}'", idx, path_str));
                    return 0;
                }
                arr[idx] = new_val;
                1
            }
            _ => {
                Logger::warn(&format!("(E053) Parent node is not a container in '{}'", path_str));
                0
            }
        }
    }

    #[native(name = "json_set_at")]
    pub fn json_set_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        value_json: AmxString,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let val_str = amx_cstr(&value_json);
        let new_val: Value = match serde_json::from_str(&val_str) {
            Ok(v) => v,
            Err(e) => {
                Logger::error_detail(
                    "(E050) Invalid value_json in json_set_at (see log file for details)",
                    &format!("(E050) Invalid value_json in json_set_at: {}", e),
                );
                return Ok(0);
            }
        };
        Ok(self.set_path_value(id, &path_str, new_val))
    }

    #[native(name = "json_set_string_at")]
    pub fn json_set_string_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        value: AmxString,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let v = amx_cstr(&value);
        Ok(self.set_path_value(id, &path_str, Value::String(v)))
    }

    #[native(name = "json_set_int_at")]
    pub fn json_set_int_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        value: i32,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let v = Value::Number(serde_json::Number::from(value));
        Ok(self.set_path_value(id, &path_str, v))
    }

    #[native(name = "json_set_float_at")]
    pub fn json_set_float_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        value: f32,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        let num = match serde_json::Number::from_f64(value as f64) {
            Some(n) => n,
            None => {
                Logger::warn(&format!("(E055) Non-finite float in json_set_float_at at '{}'", path_str));
                return Ok(0);
            }
        };
        Ok(self.set_path_value(id, &path_str, Value::Number(num)))
    }

    #[native(name = "json_set_bool_at")]
    pub fn json_set_bool_at(
        &mut self,
        _: &Amx,
        id: i32,
        path: AmxString,
        value: i32,
    ) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        Ok(self.set_path_value(id, &path_str, Value::Bool(value != 0)))
    }

    #[native(name = "json_set_null_at")]
    pub fn json_set_null_at(&mut self, _: &Amx, id: i32, path: AmxString) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        Ok(self.set_path_value(id, &path_str, Value::Null))
    }

    #[native(name = "json_delete_at")]
    pub fn json_delete_at(&mut self, _: &Amx, id: i32, path: AmxString) -> AmxResult<i32> {
        let path_str = amx_cstr(&path);
        Ok(self.delete_path(id, &path_str))
    }

    // ----------------------- TYPED ARRAY GETTERS -----------------------

    /// Resolves `root[key][index][field]` for the array-of-objects getters.
    /// Logs (and returns `None`) on any miss. Shared by the typed
    /// `json_array_get_*` natives.
    fn array_field_value<'a>(
        &'a self,
        id: i32,
        key: &str,
        index: i32,
        field: &str,
        native: &str,
    ) -> Option<&'a Value> {
        let root = match self.pool.get(&id) {
            Some(v) => v,
            None => {
                Logger::warn(&format!("(E002) ID {} not found in {}", id, native));
                return None;
            }
        };
        let arr = match root.as_object().and_then(|m| m.get(key)) {
            Some(Value::Array(a)) => a,
            Some(_) => {
                Logger::warn(&format!("(E035) '{}' is not an array in {}", key, native));
                return None;
            }
            None => {
                Logger::warn(&format!("(E005) Key '{}' not found in {}", key, native));
                return None;
            }
        };
        match to_index(index).and_then(|i| arr.get(i)) {
            Some(Value::Object(map)) => match map.get(field) {
                Some(v) => Some(v),
                None => {
                    Logger::warn(&format!("(E038) Field '{}' missing in {}[{}]", field, key, index));
                    None
                }
            },
            _ => {
                Logger::warn(&format!("(E037) Invalid index {} or missing object in '{}'", index, key));
                None
            }
        }
    }

    #[native(name = "json_array_get_int")]
    #[allow(clippy::too_many_arguments)]
    pub fn json_array_get_int(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        index: i32,
        field: AmxString,
        mut out: Ref<i32>,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let field_str = amx_cstr(&field);
        let val = self
            .array_field_value(id, &key_str, index, &field_str, "json_array_get_int")
            .and_then(|v| match v {
                Value::Number(n) => n.as_i64(),
                Value::String(s) => s.parse::<i64>().ok(),
                Value::Bool(b) => Some(if *b { 1 } else { 0 }),
                _ => None,
            });
        match val {
            Some(i) => {
                *out = i as i32;
                Ok(1)
            }
            None => Ok(0),
        }
    }

    #[native(name = "json_array_get_float")]
    #[allow(clippy::too_many_arguments)]
    pub fn json_array_get_float(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        index: i32,
        field: AmxString,
        mut out: Ref<f32>,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let field_str = amx_cstr(&field);
        let val = self
            .array_field_value(id, &key_str, index, &field_str, "json_array_get_float")
            .and_then(|v| match v {
                Value::Number(n) => n.as_f64(),
                Value::String(s) => s.parse::<f64>().ok(),
                Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                _ => None,
            });
        match val {
            Some(f) => {
                *out = f as f32;
                Ok(1)
            }
            None => Ok(0),
        }
    }

    #[native(name = "json_array_get_bool")]
    #[allow(clippy::too_many_arguments)]
    pub fn json_array_get_bool(
        &mut self,
        _: &Amx,
        id: i32,
        key: AmxString,
        index: i32,
        field: AmxString,
        mut out: Ref<i32>,
    ) -> AmxResult<i32> {
        let key_str = amx_cstr(&key);
        let field_str = amx_cstr(&field);
        let val = self
            .array_field_value(id, &key_str, index, &field_str, "json_array_get_bool")
            .and_then(|v| v.as_bool());
        match val {
            Some(b) => {
                *out = if b { 1 } else { 0 };
                Ok(1)
            }
            None => Ok(0),
        }
    }

    // ----------------------- SERIALIZE (pretty) -----------------------

    #[native(name = "json_to_string_pretty")]
    pub fn json_to_string_pretty(
        &mut self,
        _: &Amx,
        id: i32,
        output: UnsizedBuffer,
        size: usize,
    ) -> AmxResult<i32> {
        match self.to_text_doc(id, true) {
            Some(json_str) => {
                output.write_str(size, &json_str)?;
                Ok(1)
            }
            None => Ok(0),
        }
    }

    // ----------------------- HANDLE-LEVEL OPERATIONS -----------------------

    #[native(name = "json_create_array")]
    pub fn json_create_array(&mut self, _: &Amx, mut out: Ref<i32>) -> AmxResult<i32> {
        *out = self.create_array();
        Ok(1)
    }

    /// Appends `value_json` to a handle that is itself an array.
    #[native(name = "json_array_append")]
    pub fn json_array_append(
        &mut self,
        _: &Amx,
        id: i32,
        value_json: AmxString,
    ) -> AmxResult<i32> {
        let val_str = amx_cstr(&value_json);
        let val: Value = match serde_json::from_str(&val_str) {
            Ok(v) => v,
            Err(e) => {
                Logger::error_detail(
                    "(E056) Invalid value_json in json_array_append (see log file for details)",
                    &format!("(E056) Invalid value_json in json_array_append: {}", e),
                );
                return Ok(0);
            }
        };
        self.append_native(id, val)
    }

    #[native(name = "json_array_append_string")]
    pub fn json_array_append_string(
        &mut self,
        _: &Amx,
        id: i32,
        value: AmxString,
    ) -> AmxResult<i32> {
        self.append_native(id, Value::String(amx_cstr(&value)))
    }

    #[native(name = "json_array_append_int")]
    pub fn json_array_append_int(&mut self, _: &Amx, id: i32, value: i32) -> AmxResult<i32> {
        self.append_native(id, Value::Number(serde_json::Number::from(value)))
    }

    #[native(name = "json_array_append_float")]
    pub fn json_array_append_float(&mut self, _: &Amx, id: i32, value: f32) -> AmxResult<i32> {
        match serde_json::Number::from_f64(value as f64) {
            Some(num) => self.append_native(id, Value::Number(num)),
            None => {
                Logger::warn("(E055) Non-finite float in json_array_append_float");
                Ok(0)
            }
        }
    }

    #[native(name = "json_array_append_bool")]
    pub fn json_array_append_bool(&mut self, _: &Amx, id: i32, value: i32) -> AmxResult<i32> {
        self.append_native(id, Value::Bool(value != 0))
    }

    #[native(name = "json_array_append_null")]
    pub fn json_array_append_null(&mut self, _: &Amx, id: i32) -> AmxResult<i32> {
        self.append_native(id, Value::Null)
    }

    /// Removes the element at `index` from a handle that is itself an array.
    #[native(name = "json_array_remove")]
    pub fn json_array_remove(&mut self, _: &Amx, id: i32, index: i32) -> AmxResult<i32> {
        match self.array_remove_handle(id, index) {
            Ok(true) => Ok(1),
            Ok(false) => {
                Logger::warn(&format!("(E032) Invalid index {} in json_array_remove", index));
                Ok(0)
            }
            Err(DocErr::WrongType) => {
                Logger::warn(&format!("(E049) Node is not an array in json_array_remove (ID {})", id));
                Ok(0)
            }
            Err(DocErr::NotFound) => {
                Logger::warn(&format!("(E047) ID {} not found in json_array_remove", id));
                Ok(0)
            }
        }
    }

    /// Deep-copies a handle into a new one.
    #[native(name = "json_clone")]
    pub fn json_clone(&mut self, _: &Amx, id: i32, mut out_id: Ref<i32>) -> AmxResult<i32> {
        match self.clone_doc(id) {
            Some(new_id) => {
                *out_id = new_id;
                Ok(1)
            }
            None => {
                Logger::warn(&format!("(E047) ID {} not found in json_clone", id));
                Ok(0)
            }
        }
    }

    /// Empties an object/array handle in place, keeping the same ID.
    #[native(name = "json_clear")]
    pub fn json_clear(&mut self, _: &Amx, id: i32) -> AmxResult<i32> {
        match self.clear_doc(id) {
            Ok(()) => Ok(1),
            Err(DocErr::WrongType) => {
                Logger::warn(&format!("(E057) Node is not a container in json_clear (ID {})", id));
                Ok(0)
            }
            Err(DocErr::NotFound) => {
                Logger::warn(&format!("(E047) ID {} not found in json_clear", id));
                Ok(0)
            }
        }
    }

    /// Deep-merges the `src` object into the `dest` object.
    #[native(name = "json_merge")]
    pub fn json_merge(&mut self, _: &Amx, dest_id: i32, src_id: i32) -> AmxResult<i32> {
        match self.merge_docs(dest_id, src_id) {
            Ok(()) => Ok(1),
            Err(MergeErr::SrcNotObject) => {
                Logger::warn(&format!("(E058) Source ID {} is not an object in json_merge", src_id));
                Ok(0)
            }
            Err(MergeErr::SrcNotFound) => {
                Logger::warn(&format!("(E047) Source ID {} not found in json_merge", src_id));
                Ok(0)
            }
            Err(MergeErr::DestNotObject) => {
                Logger::warn(&format!("(E058) Destination ID {} is not an object in json_merge", dest_id));
                Ok(0)
            }
            Err(MergeErr::DestNotFound) => {
                Logger::warn(&format!("(E047) Destination ID {} not found in json_merge", dest_id));
                Ok(0)
            }
        }
    }

    // ----------------------- CONVENIENCE / DIAGNOSTICS -----------------------

    #[native(name = "json_set_null")]
    pub fn json_set_null(&mut self, _: &Amx, id: i32, key: AmxString) -> AmxResult<i32> {
        Ok(self.set_key(id, amx_cstr(&key), Value::Null) as i32)
    }

    #[native(name = "json_is_array")]
    pub fn json_is_array(&mut self, _: &Amx, id: i32) -> AmxResult<i32> {
        Ok(self.is_array_doc(id) as i32)
    }

    #[native(name = "json_is_object")]
    pub fn json_is_object(&mut self, _: &Amx, id: i32) -> AmxResult<i32> {
        Ok(self.is_object_doc(id) as i32)
    }

    /// Returns the number of open handles in the pool (helps spot leaks of
    /// documents that were never freed with `json_free`).
    #[native(name = "json_count")]
    pub fn json_count(&mut self, _: &Amx) -> AmxResult<i32> {
        Ok(self.pool.len() as i32)
    }

    /// Structural comparison of two handles. Returns 1 when both exist and are
    /// deeply equal, 0 otherwise.
    #[native(name = "json_equals")]
    pub fn json_equals(&mut self, _: &Amx, id_a: i32, id_b: i32) -> AmxResult<i32> {
        Ok(self.equals_docs(id_a, id_b) as i32)
    }
}

/// Failure reason for handle operations that need an object/array/container.
#[derive(Debug, PartialEq, Eq)]
enum DocErr {
    NotFound,
    WrongType,
}

/// Failure reason for `merge_docs`, distinguishing source from destination.
#[derive(Debug, PartialEq, Eq)]
enum MergeErr {
    SrcNotFound,
    SrcNotObject,
    DestNotFound,
    DestNotObject,
}

/// Outcome of appending to an array stored under a key inside an object.
#[derive(Debug, PartialEq, Eq)]
enum AppendKeyOutcome {
    Done,
    NotObject,
    KeyNotArray,
}

/// Failure reason for reading a JSON file.
enum FileLoadErr {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

/// Failure reason for writing a JSON file.
enum FileSaveErr {
    Serialize(serde_json::Error),
    Mkdir(std::io::Error),
    Write(std::io::Error),
}

/// Failure reason for creating an empty JSON file.
enum FileCreateErr {
    Mkdir(std::io::Error),
    Write(std::io::Error),
}

/// Reads and parses a JSON document from disk.
fn read_json_file(path: &str) -> Result<Value, FileLoadErr> {
    let content = std::fs::read_to_string(path).map_err(FileLoadErr::Io)?;
    serde_json::from_str(&content).map_err(FileLoadErr::Parse)
}

/// Serializes `value` (pretty) and writes it to `path`, creating parent
/// directories as needed.
fn write_json_file(path: &str, value: &Value) -> Result<(), FileSaveErr> {
    let pretty = serde_json::to_string_pretty(value).map_err(FileSaveErr::Serialize)?;
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(FileSaveErr::Mkdir)?;
    }
    std::fs::write(path, pretty).map_err(FileSaveErr::Write)
}

/// Creates an empty `{}` JSON file. Returns `Ok(false)` if it already existed
/// (a no-op), `Ok(true)` if it was created.
fn create_empty_json_file(path: &str) -> Result<bool, FileCreateErr> {
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(FileCreateErr::Mkdir)?;
    }
    if Path::new(path).exists() {
        return Ok(false);
    }
    std::fs::write(path, "{\n}\n").map_err(FileCreateErr::Write)?;
    Ok(true)
}

/// Recursively merges `src` into `dest`. Nested objects are merged key by key;
/// any other value type overwrites the destination.
fn merge_value(dest: &mut Value, src: Value) {
    match (dest, src) {
        (Value::Object(dest_map), Value::Object(src_map)) => {
            for (k, v) in src_map {
                merge_value(dest_map.entry(k).or_insert(Value::Null), v);
            }
        }
        (dest_slot, src_val) => *dest_slot = src_val,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // Builds a Plugin without touching the SA-MP runtime. Log level is set to
    // "none" so failure-path assertions do not spill log files during tests.
    fn plugin() -> crate::Plugin {
        Logger::set_log_level(0);
        crate::Plugin {
            pool: HashMap::new(),
            next_id: 1,
        }
    }

    // ---------------- to_index ----------------

    #[test]
    fn to_index_rejects_negative_and_accepts_non_negative() {
        assert_eq!(to_index(-1), None);
        assert_eq!(to_index(i32::MIN), None);
        assert_eq!(to_index(0), Some(0));
        assert_eq!(to_index(42), Some(42));
    }

    // ---------------- resolve_scriptfiles_path ----------------

    #[test]
    fn resolve_scriptfiles_path_prefixes_relative_names() {
        assert_eq!(resolve_scriptfiles_path("data.json"), "scriptfiles/data.json");
        assert_eq!(resolve_scriptfiles_path("dir/data.json"), "scriptfiles/dir/data.json");
    }

    #[test]
    fn resolve_scriptfiles_path_keeps_explicit_paths() {
        assert_eq!(resolve_scriptfiles_path("scriptfiles/x.json"), "scriptfiles/x.json");
        assert_eq!(resolve_scriptfiles_path("./x.json"), "./x.json");
        assert_eq!(resolve_scriptfiles_path("../x.json"), "../x.json");
        assert_eq!(resolve_scriptfiles_path("/abs/x.json"), "/abs/x.json");
        assert_eq!(resolve_scriptfiles_path("C:/x.json"), "C:/x.json");
    }

    #[test]
    fn resolve_scriptfiles_path_normalizes_backslashes_and_empty() {
        assert_eq!(resolve_scriptfiles_path("dir\\x.json"), "scriptfiles/dir/x.json");
        assert_eq!(resolve_scriptfiles_path("   "), "");
        assert_eq!(resolve_scriptfiles_path(""), "");
    }

    // ---------------- token parsing ----------------

    #[test]
    fn unescape_pointer_token_decodes_escapes() {
        assert_eq!(unescape_pointer_token("a~1b"), "a/b");
        assert_eq!(unescape_pointer_token("a~0b"), "a~b");
        assert_eq!(unescape_pointer_token("a~01"), "a~1");
    }

    #[test]
    fn tokens_from_pointer_handles_root_and_segments() {
        assert!(tokens_from_pointer("").is_empty());
        assert!(tokens_from_pointer("/").is_empty());
        assert_eq!(tokens_from_pointer("/a/0/b"), vec!["a", "0", "b"]);
        assert_eq!(tokens_from_pointer("/a~1b/c"), vec!["a/b", "c"]);
    }

    #[test]
    fn tokens_from_friendly_splits_dots_and_brackets() {
        assert_eq!(tokens_from_friendly("a.b"), vec!["a", "b"]);
        assert_eq!(tokens_from_friendly("a[0].b"), vec!["a", "0", "b"]);
        assert_eq!(
            tokens_from_friendly("radios[0].genres[1]"),
            vec!["radios", "0", "genres", "1"]
        );
    }

    #[test]
    fn normalize_tokens_dispatches_on_leading_slash() {
        assert_eq!(normalize_tokens("/a/0"), vec!["a", "0"]);
        assert_eq!(normalize_tokens("a[0]"), vec!["a", "0"]);
        // leading whitespace is trimmed before dispatching
        assert_eq!(normalize_tokens("  /a"), vec!["a"]);
    }

    // ---------------- get_at / get_parent_mut ----------------

    #[test]
    fn get_at_navigates_objects_and_arrays() {
        let root = json!({ "a": { "b": [10, 20, 30] } });
        let tokens = normalize_tokens("a.b[2]");
        assert_eq!(get_at(&root, &tokens), Some(&json!(30)));
    }

    #[test]
    fn get_at_returns_none_on_miss_or_wrong_type() {
        let root = json!({ "a": [1, 2] });
        assert_eq!(get_at(&root, &normalize_tokens("a[9]")), None);
        assert_eq!(get_at(&root, &normalize_tokens("a.b")), None);
        assert_eq!(get_at(&root, &normalize_tokens("missing")), None);
    }

    #[test]
    fn get_parent_mut_returns_parent_and_last_token() {
        let mut root = json!({ "a": { "b": 1 } });
        let tokens = normalize_tokens("a.b");
        let (parent, last) = get_parent_mut(&mut root, &tokens).unwrap();
        assert_eq!(last, "b");
        assert!(parent.is_object());
    }

    #[test]
    fn get_parent_mut_root_for_empty_tokens() {
        let mut root = json!({ "a": 1 });
        let (parent, last) = get_parent_mut(&mut root, &[]).unwrap();
        assert_eq!(last, "");
        assert!(parent.is_object());
    }

    // ---------------- merge_value (json_merge) ----------------

    #[test]
    fn merge_value_deep_merges_objects() {
        let mut dest = json!({ "a": 1, "nested": { "x": 1, "y": 2 } });
        let src = json!({ "b": 2, "nested": { "y": 20, "z": 30 } });
        merge_value(&mut dest, src);
        assert_eq!(
            dest,
            json!({ "a": 1, "b": 2, "nested": { "x": 1, "y": 20, "z": 30 } })
        );
    }

    #[test]
    fn merge_value_overwrites_non_objects() {
        let mut dest = json!({ "a": { "x": 1 } });
        // a scalar replaces the previous object at "a"
        merge_value(&mut dest, json!({ "a": 5 }));
        assert_eq!(dest, json!({ "a": 5 }));
    }

    // ---------------- alloc_id / alloc_id_and_insert ----------------

    #[test]
    fn alloc_id_increments_and_is_unique() {
        let mut p = plugin();
        let a = p.alloc_id();
        let b = p.alloc_id();
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn alloc_id_and_insert_stores_value() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "k": "v" }));
        assert_eq!(p.pool.get(&id), Some(&json!({ "k": "v" })));
    }

    // ---------------- set_path_value (json_set_at + json_set_*_at) ----------------

    #[test]
    fn set_path_value_replaces_root_for_empty_path() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "old": true }));
        assert_eq!(p.set_path_value(id, "", json!([1, 2])), 1);
        assert_eq!(p.pool[&id], json!([1, 2]));
    }

    #[test]
    fn set_path_value_inserts_and_overwrites_object_key() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "a": 1 }));
        assert_eq!(p.set_path_value(id, "b", json!("new")), 1);
        assert_eq!(p.set_path_value(id, "a", json!(2)), 1);
        assert_eq!(p.pool[&id], json!({ "a": 2, "b": "new" }));
    }

    #[test]
    fn set_path_value_writes_nested_and_array_index() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "nested": { "arr": [0, 0] } }));
        assert_eq!(p.set_path_value(id, "nested.arr[1]", json!(99)), 1);
        assert_eq!(p.pool[&id], json!({ "nested": { "arr": [0, 99] } }));
    }

    #[test]
    fn set_path_value_fails_on_bad_targets() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "arr": [1] }));
        // missing id
        assert_eq!(p.set_path_value(999, "a", json!(1)), 0);
        // array index out of bounds
        assert_eq!(p.set_path_value(id, "arr[5]", json!(1)), 0);
        // non-numeric index into an array
        assert_eq!(p.set_path_value(id, "arr.x", json!(1)), 0);
        // parent path does not exist
        assert_eq!(p.set_path_value(id, "missing.child", json!(1)), 0);
    }

    // ---------------- array_field_value (json_array_get_*) ----------------

    #[test]
    fn array_field_value_returns_field_of_object_element() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({
            "users": [ { "name": "ana", "age": 30 }, { "name": "bob", "age": 25 } ]
        }));
        assert_eq!(
            p.array_field_value(id, "users", 1, "name", "t"),
            Some(&json!("bob"))
        );
        assert_eq!(
            p.array_field_value(id, "users", 0, "age", "t"),
            Some(&json!(30))
        );
    }

    #[test]
    fn array_field_value_returns_none_on_every_miss() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({
            "users": [ { "name": "ana" } ],
            "scalar": 5
        }));
        assert_eq!(p.array_field_value(999, "users", 0, "name", "t"), None); // missing id
        assert_eq!(p.array_field_value(id, "scalar", 0, "name", "t"), None); // not an array
        assert_eq!(p.array_field_value(id, "missing", 0, "name", "t"), None); // key missing
        assert_eq!(p.array_field_value(id, "users", 9, "name", "t"), None); // index oob
        assert_eq!(p.array_field_value(id, "users", -1, "name", "t"), None); // negative index
        assert_eq!(p.array_field_value(id, "users", 0, "age", "t"), None); // field missing
    }

    // ---------------- value coercion / inspection ----------------

    #[test]
    fn is_valid_json_distinguishes_valid_and_invalid() {
        assert!(is_valid_json(r#"{"a":1}"#));
        assert!(is_valid_json("[1,2,3]"));
        assert!(is_valid_json("42"));
        assert!(!is_valid_json("{bad}"));
        assert!(!is_valid_json(""));
    }

    #[test]
    fn value_type_code_maps_every_variant() {
        assert_eq!(value_type_code(&json!(null)), 0);
        assert_eq!(value_type_code(&json!(true)), 1);
        assert_eq!(value_type_code(&json!(3)), 2);
        assert_eq!(value_type_code(&json!("s")), 3);
        assert_eq!(value_type_code(&json!([])), 4);
        assert_eq!(value_type_code(&json!({})), 5);
    }

    #[test]
    fn value_len_counts_containers_only() {
        assert_eq!(value_len(&json!([1, 2, 3])), 3);
        assert_eq!(value_len(&json!({ "a": 1, "b": 2 })), 2);
        assert_eq!(value_len(&json!("hello")), 0);
        assert_eq!(value_len(&json!(7)), 0);
    }

    #[test]
    fn value_as_i64_respects_allow_bool() {
        assert_eq!(value_as_i64(&json!(5), false), Some(5));
        assert_eq!(value_as_i64(&json!("12"), false), Some(12));
        assert_eq!(value_as_i64(&json!("x"), false), None);
        assert_eq!(value_as_i64(&json!(true), false), None);
        assert_eq!(value_as_i64(&json!(true), true), Some(1));
        assert_eq!(value_as_i64(&json!(false), true), Some(0));
    }

    #[test]
    fn value_as_f64_respects_allow_bool() {
        assert_eq!(value_as_f64(&json!(1.5), false), Some(1.5));
        assert_eq!(value_as_f64(&json!("2.5"), false), Some(2.5));
        assert_eq!(value_as_f64(&json!(true), false), None);
        assert_eq!(value_as_f64(&json!(true), true), Some(1.0));
    }

    #[test]
    fn value_as_text_returns_strings_verbatim_else_serialized() {
        assert_eq!(value_as_text(&json!("hi")), Some("hi".to_string()));
        assert_eq!(value_as_text(&json!(42)), Some("42".to_string()));
        assert_eq!(value_as_text(&json!({ "a": 1 })), Some(r#"{"a":1}"#.to_string()));
    }

    // ---------------- simple pool cores ----------------

    #[test]
    fn create_object_and_array_store_empty_containers() {
        let mut p = plugin();
        let obj = p.create_object();
        let arr = p.create_array();
        assert_eq!(p.pool[&obj], json!({}));
        assert_eq!(p.pool[&arr], json!([]));
        assert_ne!(obj, arr);
    }

    #[test]
    fn free_doc_removes_only_existing() {
        let mut p = plugin();
        let id = p.create_object();
        assert!(p.free_doc(id));
        assert!(!p.free_doc(id));
        assert!(p.pool.is_empty());
    }

    #[test]
    fn has_key_checks_object_membership() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "a": 1 }));
        assert!(p.has_key(id, "a"));
        assert!(!p.has_key(id, "b"));
        assert!(!p.has_key(999, "a"));
        let arr = p.alloc_id_and_insert(json!([1]));
        assert!(!p.has_key(arr, "a"));
    }

    #[test]
    fn set_key_inserts_only_into_objects() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({}));
        assert!(p.set_key(id, "a".to_string(), json!(1)));
        assert!(p.set_key(id, "a".to_string(), json!(2))); // overwrite
        assert_eq!(p.pool[&id], json!({ "a": 2 }));
        let arr = p.alloc_id_and_insert(json!([]));
        assert!(!p.set_key(arr, "a".to_string(), json!(1)));
        assert!(!p.set_key(999, "a".to_string(), json!(1)));
    }

    #[test]
    fn delete_key_removes_present_keys() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "a": 1, "b": 2 }));
        assert!(p.delete_key(id, "a"));
        assert!(!p.delete_key(id, "a"));
        assert_eq!(p.pool[&id], json!({ "b": 2 }));
        assert!(!p.delete_key(999, "b"));
    }

    #[test]
    fn to_text_doc_compact_and_pretty() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "a": 1 }));
        assert_eq!(p.to_text_doc(id, false), Some(r#"{"a":1}"#.to_string()));
        assert_eq!(p.to_text_doc(id, true), Some("{\n  \"a\": 1\n}".to_string()));
        assert_eq!(p.to_text_doc(999, false), None);
    }

    #[test]
    fn object_len_and_key_at_doc() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "x": 1, "y": 2 }));
        assert_eq!(p.object_len(id), Some(2));
        assert_eq!(p.key_at_doc(id, 0), Some("x".to_string()));
        assert_eq!(p.key_at_doc(id, 1), Some("y".to_string()));
        assert_eq!(p.key_at_doc(id, 2), None);
        assert_eq!(p.key_at_doc(id, -1), None);
        // not an object / missing
        let arr = p.alloc_id_and_insert(json!([1]));
        assert_eq!(p.object_len(arr), None);
        assert_eq!(p.object_len(999), None);
    }

    #[test]
    fn array_len_key_only_for_array_values() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "arr": [1, 2, 3], "scalar": 5 }));
        assert_eq!(p.array_len_key(id, "arr"), Some(3));
        assert_eq!(p.array_len_key(id, "scalar"), None);
        assert_eq!(p.array_len_key(id, "missing"), None);
        assert_eq!(p.array_len_key(999, "arr"), None);
    }

    #[test]
    fn item_clone_reads_array_elements() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!([10, { "k": "v" }, 30]));
        assert_eq!(p.item_clone(id, 1), Some(json!({ "k": "v" })));
        assert_eq!(p.item_clone(id, 3), None);
        assert_eq!(p.item_clone(id, -1), None);
        let obj = p.alloc_id_and_insert(json!({}));
        assert_eq!(p.item_clone(obj, 0), None);
    }

    #[test]
    fn at_clone_reads_nested_paths() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "a": { "b": [1, 2] } }));
        assert_eq!(p.at_clone(id, "a.b[1]"), Some(json!(2)));
        assert_eq!(p.at_clone(id, "/a/b/0"), Some(json!(1)));
        assert_eq!(p.at_clone(id, "a.missing"), None);
        assert_eq!(p.at_clone(999, "a"), None);
    }

    #[test]
    fn type_predicates_and_equals() {
        let mut p = plugin();
        let obj = p.alloc_id_and_insert(json!({ "a": 1 }));
        let arr = p.alloc_id_and_insert(json!([1]));
        let obj2 = p.alloc_id_and_insert(json!({ "a": 1 }));
        assert!(p.is_object_doc(obj));
        assert!(!p.is_object_doc(arr));
        assert!(p.is_array_doc(arr));
        assert!(!p.is_array_doc(obj));
        assert!(p.equals_docs(obj, obj2));
        assert!(!p.equals_docs(obj, arr));
        assert!(!p.equals_docs(obj, 999)); // one missing
    }

    #[test]
    fn clone_doc_makes_independent_copy() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "a": 1 }));
        let copy = p.clone_doc(id).unwrap();
        assert_ne!(id, copy);
        p.set_key(copy, "a".to_string(), json!(99));
        assert_eq!(p.pool[&id], json!({ "a": 1 })); // original untouched
        assert_eq!(p.clone_doc(999), None);
    }

    #[test]
    fn clear_doc_empties_containers_only() {
        let mut p = plugin();
        let obj = p.alloc_id_and_insert(json!({ "a": 1 }));
        let arr = p.alloc_id_and_insert(json!([1, 2]));
        let scalar = p.alloc_id_and_insert(json!(5));
        assert_eq!(p.clear_doc(obj), Ok(()));
        assert_eq!(p.clear_doc(arr), Ok(()));
        assert_eq!(p.pool[&obj], json!({}));
        assert_eq!(p.pool[&arr], json!([]));
        assert_eq!(p.clear_doc(scalar), Err(DocErr::WrongType));
        assert_eq!(p.clear_doc(999), Err(DocErr::NotFound));
    }

    #[test]
    fn array_append_handle_pushes_or_errors() {
        let mut p = plugin();
        let arr = p.alloc_id_and_insert(json!([1]));
        assert_eq!(p.array_append_handle(arr, json!(2)), Ok(()));
        assert_eq!(p.pool[&arr], json!([1, 2]));
        let obj = p.alloc_id_and_insert(json!({}));
        assert_eq!(p.array_append_handle(obj, json!(1)), Err(DocErr::WrongType));
        assert_eq!(p.array_append_handle(999, json!(1)), Err(DocErr::NotFound));
    }

    #[test]
    fn array_remove_handle_removes_by_index() {
        let mut p = plugin();
        let arr = p.alloc_id_and_insert(json!([10, 20, 30]));
        assert_eq!(p.array_remove_handle(arr, 1), Ok(true));
        assert_eq!(p.pool[&arr], json!([10, 30]));
        assert_eq!(p.array_remove_handle(arr, 9), Ok(false));
        assert_eq!(p.array_remove_handle(arr, -1), Ok(false));
        let obj = p.alloc_id_and_insert(json!({}));
        assert_eq!(p.array_remove_handle(obj, 0), Err(DocErr::WrongType));
        assert_eq!(p.array_remove_handle(999, 0), Err(DocErr::NotFound));
    }

    #[test]
    fn append_array_key_creates_or_pushes() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "list": [1], "scalar": 9 }));
        assert_eq!(p.append_array_key(id, "list", json!(2)), AppendKeyOutcome::Done);
        assert_eq!(p.append_array_key(id, "new", json!("a")), AppendKeyOutcome::Done);
        assert_eq!(p.pool[&id]["list"], json!([1, 2]));
        assert_eq!(p.pool[&id]["new"], json!(["a"]));
        assert_eq!(
            p.append_array_key(id, "scalar", json!(1)),
            AppendKeyOutcome::KeyNotArray
        );
        let arr = p.alloc_id_and_insert(json!([]));
        assert_eq!(p.append_array_key(arr, "k", json!(1)), AppendKeyOutcome::NotObject);
    }

    #[test]
    fn merge_docs_reports_specific_errors() {
        let mut p = plugin();
        let dest = p.alloc_id_and_insert(json!({ "a": 1 }));
        let src = p.alloc_id_and_insert(json!({ "b": 2 }));
        assert_eq!(p.merge_docs(dest, src), Ok(()));
        assert_eq!(p.pool[&dest], json!({ "a": 1, "b": 2 }));
        // same id is a no-op success
        assert_eq!(p.merge_docs(dest, dest), Ok(()));
        let arr = p.alloc_id_and_insert(json!([1]));
        assert_eq!(p.merge_docs(dest, arr), Err(MergeErr::SrcNotObject));
        assert_eq!(p.merge_docs(dest, 999), Err(MergeErr::SrcNotFound));
        assert_eq!(p.merge_docs(arr, src), Err(MergeErr::DestNotObject));
        assert_eq!(p.merge_docs(999, src), Err(MergeErr::DestNotFound));
    }

    #[test]
    fn delete_path_removes_nested_nodes() {
        let mut p = plugin();
        let id = p.alloc_id_and_insert(json!({ "a": { "b": 1 }, "arr": [10, 20] }));
        assert_eq!(p.delete_path(id, "a.b"), 1);
        assert_eq!(p.delete_path(id, "arr[0]"), 1);
        assert_eq!(p.pool[&id], json!({ "a": {}, "arr": [20] }));
        assert_eq!(p.delete_path(id, ""), 0); // cannot delete root
        assert_eq!(p.delete_path(id, "missing.child"), 0);
        assert_eq!(p.delete_path(999, "a"), 0);
    }

    // ---------------- file helpers (real temp dir) ----------------

    fn temp_path(name: &str) -> String {
        let mut dir = std::env::temp_dir();
        let unique = format!(
            "json_samp_test_{}_{}",
            std::process::id(),
            // monotonic-ish suffix to avoid collisions between tests
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        dir.push(unique);
        dir.push(name);
        dir.to_string_lossy().into_owned()
    }

    #[test]
    fn write_then_read_json_file_round_trips() {
        let mut p = plugin();
        let path = temp_path("nested/data.json");
        let id = p.alloc_id_and_insert(json!({ "k": [1, 2], "s": "v" }));

        // write creates parent dirs and serializes pretty
        assert!(write_json_file(&path, &p.pool[&id]).is_ok());
        // read parses it back to an equal value
        let loaded = read_json_file(&path).ok();
        assert_eq!(loaded, Some(p.pool[&id].clone()));

        let _ = std::fs::remove_dir_all(Path::new(&path).parent().unwrap());
    }

    #[test]
    fn read_json_file_reports_io_then_parse_errors() {
        // Missing file -> Io error
        let missing = temp_path("does_not_exist.json");
        assert!(matches!(read_json_file(&missing), Err(FileLoadErr::Io(_))));

        // Existing but invalid JSON -> Parse error
        let bad = temp_path("bad.json");
        std::fs::create_dir_all(Path::new(&bad).parent().unwrap()).unwrap();
        std::fs::write(&bad, "{ not json").unwrap();
        assert!(matches!(read_json_file(&bad), Err(FileLoadErr::Parse(_))));

        let _ = std::fs::remove_dir_all(Path::new(&bad).parent().unwrap());
    }

    #[test]
    fn create_empty_json_file_is_idempotent() {
        let path = temp_path("create/empty.json");
        assert_eq!(create_empty_json_file(&path).ok(), Some(true)); // created
        assert_eq!(create_empty_json_file(&path).ok(), Some(false)); // already exists
        // the file holds valid (empty) JSON
        assert_eq!(read_json_file(&path).ok(), Some(json!({})));

        let _ = std::fs::remove_dir_all(Path::new(&path).parent().unwrap());
    }
}
