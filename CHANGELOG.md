# Changelog

All notable changes to this project are documented in this file.

Format inspired by [Keep a Changelog](https://keepachangelog.com/). Versioning follows [Semantic Versioning](https://semver.org/). The closed-source 1.x history (never publicly released) is kept under [`changelog/`](changelog/).

## [2.0.0] — 2026/06/06

First public release. The plugin was rewritten on top of [rust-samp v3.0.0](https://github.com/NullSablex/rust-samp/releases/tag/v3.0.0): a single artifact now loads on **SA-MP** and on **Open Multiplayer** (native component or legacy mode). Every native from the 1.x line is preserved, with a large API expansion and a standardized, documented packaging.

### Added

- **Universal binary.** One `.so` / `.dll` runs on SA-MP and on Open Multiplayer. open.mp auto-loads it as a native component when dropped into `components/` (no `config.json` entry required), or in legacy mode from `plugins/` when declared under `pawn.legacy_plugins`.
- **Standalone arrays.** `json_create_array` creates an array handle; `json_array_append` (raw JSON) plus typed `json_array_append_string`, `json_array_append_int`, `json_array_append_float`, `json_array_append_bool`, `json_array_append_null`; and `json_array_remove`.
- **Typed array-of-objects getters.** `json_array_get_int`, `json_array_get_float`, `json_array_get_bool` — completing the existing `json_array_get_string`.
- **Typed path setters.** `json_set_string_at`, `json_set_int_at`, `json_set_float_at`, `json_set_bool_at`, `json_set_null_at` — set deep values without building a JSON string.
- **Null setters.** `json_set_null` (by key) and `json_set_null_at` (by path).
- **Pretty serialization.** `json_to_string_pretty` (indented), alongside the compact `json_to_string`.
- **Handle-level operations.** `json_clone` (deep copy), `json_clear` (empty in place), `json_merge` (deep object merge), `json_equals` (structural compare), `json_count` (open handles, for leak hunting), `json_is_array`, `json_is_object`.
- **Logging controls.** `json_log` native with `JSON_LOG_*` levels (none/error/warning/info/all); messages go to the console and `logs/json_samp.log`, behind a startup banner.
- **Include constants.** `JSON_SAMP_VERSION` plus the `JSON_TYPE_*` and `JSON_LOG_*` enums, shipped in `json_samp.inc`.

### Changed

- **Path integer/float getters coerce booleans.** `json_get_int_at` / `json_get_float_at` now read a JSON boolean as `0/1`, matching the write side. The level-1 getters (`json_get_int` / `json_get_float`) stay strict (number or numeric string only).
- **Stable error codes.** Failures log a coded message (`E001`–`E058`) with a severity (error for hard failures, warning for recoverable misuse). File and parse errors print a concise console line and keep the full technical cause in `logs/json_samp.log`. See [docs/errors.md](docs/errors.md).
- **Include standardized.** The Pawn include is now `json_samp.inc` with `#pragma library json_samp`, generated from a template at build time so it always tracks the crate version. Update your script to `#include <json_samp>`.
- **Consistent artifact names.** Releases ship raw `json_samp.so`, `json_samp.dll` and `json_samp.inc`, identically named across Linux and Windows.
- **Documentation.** Rewritten in English as a MkDocs Material site, with per-feature guides, a full [API reference](docs/api-reference.md) and an [error-code table](docs/errors.md). Runnable Pawn snippets live under [`examples/`](examples/).
- **License.** Distributed under the **AGPL-3.0-or-later**.

### Migration from 1.x

- Replace `#include <a_json>` with `#include <json_samp>`.
- Load the plugin as `json_samp` (`plugins json_samp.so` on SA-MP; `components/` or `legacy_plugins` on open.mp).
- No existing native was removed or renamed — only additions and the include/packaging changes above.
