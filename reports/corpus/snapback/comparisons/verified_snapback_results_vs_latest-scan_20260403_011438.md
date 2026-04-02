# Deslop Findings Comparison Report

**Generated:** 2026-04-03 01:14:38
**Baseline:** `/home/chinmay/ChinmayPersonalProjects/deslop-codex/verified_snapback_results.txt`
**Latest:**   `/home/chinmay/ChinmayPersonalProjects/deslop-codex/reports/corpus/snapback/latest-scan.txt`
**Stripped path prefixes:** ['/home/chinmay/ChinmayPersonalProjects/SnapBack/', '/home/chinmay/ChinmayPersonalProjects/SnapBack/']

## Summary

| Metric | Count |
| --- | --- |
| Baseline findings | 67 |
| Latest findings | 117 |
| Net change | +50 |
| Unchanged (same finding, same line) | 67 |
| Moved (same finding, line shifted) | 0 |
| Removed (finding gone in latest) | 0 |
| Added (new finding in latest) | 50 |

## Added Findings (50 total)

### By Category

| Category | Count |
| --- | --- |
| public_any_type_leak | 28 |
| import_time_config_load | 9 |
| repeated_dict_get_same_key_no_cache | 8 |
| repeated_string_format_invariant_template | 3 |
| nested_list_search_map_candidate | 1 |
| write_without_buffering_in_loop | 1 |

### By File

| File | Count |
| --- | --- |
| apps/api/main.py | 17 |
| services/storage/database.py | 15 |
| services/analysis/summarizer.py | 9 |
| services/exporters/export.py | 5 |
| services/analysis/detector.py | 2 |
| services/transcription/transcription_client.py | 2 |

### All Added Findings

| File | Line | Category | Message |
| --- | --- | --- | --- |
| apps/api/main.py | 48 | import_time_config_load | module initializes AUTO_DELETE_AFTER_HOURS from configuration or secrets at import time |
| apps/api/main.py | 49 | import_time_config_load | module initializes NOTION_API_KEY from configuration or secrets at import time |
| apps/api/main.py | 122 | import_time_config_load | module initializes summarizer from configuration or secrets at import time |
| apps/api/main.py | 46 | import_time_config_load | module loads configuration or secrets while being imported |
| apps/api/main.py | 48 | import_time_config_load | module loads configuration or secrets while being imported |
| apps/api/main.py | 48 | import_time_config_load | module loads configuration or secrets while being imported |
| apps/api/main.py | 49 | import_time_config_load | module loads configuration or secrets while being imported |
| apps/api/main.py | 122 | import_time_config_load | module loads configuration or secrets while being imported |
| apps/api/main.py | 122 | import_time_config_load | module loads configuration or secrets while being imported |
| apps/api/main.py | 296 | public_any_type_leak | public function complete_session exposes a very wide type contract |
| apps/api/main.py | 362 | public_any_type_leak | public function export_notion exposes a very wide type contract |
| apps/api/main.py | 220 | public_any_type_leak | public function generate_recap exposes a very wide type contract |
| apps/api/main.py | 382 | public_any_type_leak | public function generate_study_pack exposes a very wide type contract |
| apps/api/main.py | 283 | public_any_type_leak | public function get_session_transcript exposes a very wide type contract |
| apps/api/main.py | 188 | public_any_type_leak | public function ingest_audio_chunk exposes a very wide type contract |
| apps/api/main.py | 174 | public_any_type_leak | public function ingest_transcript exposes a very wide type contract |
| apps/api/main.py | 159 | public_any_type_leak | public function start_session exposes a very wide type contract |
| services/analysis/detector.py | 51 | nested_list_search_map_candidate | function detect_missed_alerts uses nested loops for lookup; consider a dict/set for O(1) access |
| services/analysis/detector.py | 47 | public_any_type_leak | public function detect_missed_alerts exposes a very wide type contract |
| services/analysis/summarizer.py | 185 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 186 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 190 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 191 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 196 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 197 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 202 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 203 | repeated_dict_get_same_key_no_cache | function generate_study_pack calls .get() with the same key multiple times; assign to a local variable |
| services/analysis/summarizer.py | 146 | public_any_type_leak | public function generate_study_pack exposes a very wide type contract |
| services/exporters/export.py | 39 | repeated_string_format_invariant_template | function build_markdown_export formats a string inside a loop; consider building the template once |
| services/exporters/export.py | 93 | repeated_string_format_invariant_template | function build_pdf_export formats a string inside a loop; consider building the template once |
| services/exporters/export.py | 15 | public_any_type_leak | public function build_markdown_export exposes a very wide type contract |
| services/exporters/export.py | 62 | public_any_type_leak | public function build_pdf_export exposes a very wide type contract |
| services/exporters/export.py | 122 | public_any_type_leak | public function export_to_notion exposes a very wide type contract |
| services/storage/database.py | 397 | public_any_type_leak | model SessionBundle exposes field recaps with a very wide type contract |
| services/storage/database.py | 395 | public_any_type_leak | model SessionBundle exposes field session with a very wide type contract |
| services/storage/database.py | 396 | public_any_type_leak | model SessionBundle exposes field transcript with a very wide type contract |
| services/storage/database.py | 146 | public_any_type_leak | public function append_transcript_chunk exposes a very wide type contract |
| services/storage/database.py | 99 | public_any_type_leak | public function create_session exposes a very wide type contract |
| services/storage/database.py | 131 | public_any_type_leak | public function end_session exposes a very wide type contract |
| services/storage/database.py | 226 | public_any_type_leak | public function get_first_chunk_after exposes a very wide type contract |
| services/storage/database.py | 209 | public_any_type_leak | public function get_last_chunk_before exposes a very wide type contract |
| services/storage/database.py | 284 | public_any_type_leak | public function get_recaps exposes a very wide type contract |
| services/storage/database.py | 121 | public_any_type_leak | public function get_session exposes a very wide type contract |
| services/storage/database.py | 173 | public_any_type_leak | public function get_transcript exposes a very wide type contract |
| services/storage/database.py | 188 | public_any_type_leak | public function get_transcript_window exposes a very wide type contract |
| services/storage/database.py | 298 | public_any_type_leak | public function hydrate_recap exposes a very wide type contract |
| services/storage/database.py | 306 | public_any_type_leak | public function save_audio_chunk exposes a very wide type contract |
| services/storage/database.py | 243 | public_any_type_leak | public function save_recap exposes a very wide type contract |
| services/transcription/transcription_client.py | 347 | write_without_buffering_in_loop | function _ensure_whisper_model calls .write() inside a loop without visible buffering |
| services/transcription/transcription_client.py | 256 | repeated_string_format_invariant_template | function _post_transcript formats a string inside a loop; consider building the template once |
