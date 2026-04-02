# Deslop Findings Comparison Report

**Generated:** 2026-04-03 00:19:52
**Baseline:** `/home/chinmay/ChinmayPersonalProjects/deslop-codex/verified_gopdfsuit_results.txt`
**Latest:**   `/home/chinmay/ChinmayPersonalProjects/deslop-codex/reports/corpus/gopdfsuit/latest-scan.txt`
**Stripped path prefixes:** ['/home/chinmay/ChinmayPersonalProjects/gopdfsuit/', '/home/chinmay/ChinmayPersonalProjects/gopdfsuit/']

## Summary

| Metric | Count |
| --- | --- |
| Baseline findings | 386 |
| Latest findings | 910 |
| Net change | +524 |
| Unchanged (same finding, same line) | 347 |
| Moved (same finding, line shifted) | 0 |
| Removed (finding gone in latest) | 39 |
| Added (new finding in latest) | 563 |

## Removed Findings (39 total)

### By Category

| Category | Count |
| --- | --- |
| print_debugging_leftover | 38 |
| exception_swallowed | 1 |

### By File

| File | Count |
| --- | --- |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 32 |
| sampledata/financialreport/data/generate_charts.py | 2 |
| sampledata/librarybook/data/generate_codes.py | 2 |
| sampledata/svg/generate_math_svg.py | 2 |
| sampledata/benchmarks/fpdf/bench.py | 1 |

### All Removed Findings

| File | Line | Category | Message |
| --- | --- | --- | --- |
| sampledata/benchmarks/fpdf/bench.py | 36 | exception_swallowed | function run_once swallows a broad exception handler |
| sampledata/financialreport/data/generate_charts.py | 37 | print_debugging_leftover | function generate_bar_chart leaves print-style debugging in Python code |
| sampledata/financialreport/data/generate_charts.py | 63 | print_debugging_leftover | function generate_pie_chart leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 416 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 417 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 418 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 420 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 431 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 433 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 437 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 439 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 443 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 444 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 445 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 446 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 483 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 484 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 485 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 486 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 487 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 488 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 489 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 490 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 491 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 492 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 493 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 494 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 495 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 496 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 497 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 498 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 499 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 500 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 501 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 502 | print_debugging_leftover | function run_benchmark leaves print-style debugging in Python code |
| sampledata/librarybook/data/generate_codes.py | 69 | print_debugging_leftover | function generate_barcode leaves print-style debugging in Python code |
| sampledata/librarybook/data/generate_codes.py | 36 | print_debugging_leftover | function generate_qr_code leaves print-style debugging in Python code |
| sampledata/svg/generate_math_svg.py | 79 | print_debugging_leftover | function save_math_svg leaves print-style debugging in Python code |
| sampledata/svg/generate_math_svg.py | 80 | print_debugging_leftover | function save_math_svg leaves print-style debugging in Python code |

## Added Findings (563 total)

### By Category

| Category | Count |
| --- | --- |
| slice_append_without_prealloc_known_bound | 240 |
| print_debugging_leftover | 38 |
| regexp_compile_in_hot_path | 32 |
| map_growth_without_size_hint | 28 |
| public_any_type_leak | 21 |
| bytes_buffer_without_grow_known_bound | 18 |
| filter_then_count_then_iterate | 18 |
| strings_builder_without_grow_known_bound | 16 |
| large_h_payload_built_only_for_json_response | 12 |
| option_bag_model | 10 |
| likely_n_squared_allocation | 10 |
| byte_string_conversion_in_loop | 10 |
| strconv_repeat_on_same_binding | 10 |
| repeated_string_format_invariant_template | 10 |
| repeated_json_dumps_same_object | 8 |
| stable_value_normalization_in_inner_loop | 8 |
| builder_or_buffer_recreated_per_iteration | 8 |
| formfile_open_readall_whole_upload | 7 |
| make_slice_inside_hot_loop_same_shape | 7 |
| mutable_package_global | 6 |
| make_map_inside_hot_loop_same_shape | 6 |
| append_then_trim_each_iteration | 5 |
| nested_append_without_outer_capacity | 5 |
| repeated_c_json_inside_stream_loop | 4 |
| public_bool_parameter_api | 3 |
| repeated_slice_clone_in_loop | 3 |
| defer_in_loop_resource_growth | 3 |
| likely_n_squared_string_concat | 2 |
| repeated_map_clone_in_loop | 2 |
| uuid_hash_formatting_only_for_logs | 2 |
| string_join_without_generator | 2 |
| servefile_via_readfile_then_c_data | 1 |
| file_or_template_read_per_request | 1 |
| rows_without_close | 1 |
| json_marshaled_manually_then_c_data | 1 |
| no_streaming_for_large_export_handler | 1 |
| single_impl_interface | 1 |
| exception_swallowed | 1 |
| import_time_file_io | 1 |
| no_schema_validation_on_external_data | 1 |

### By File

| File | Count |
| --- | --- |
| internal/pdf/draw.go | 147 |
| internal/pdf/form/xfdf.go | 53 |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 36 |
| bindings/python/pypdfsuit/types.py | 31 |
| internal/pdf/merge.go | 25 |
| internal/pdf/generator.go | 21 |
| internal/handlers/handlers.go | 18 |
| internal/pdf/font/subset.go | 18 |
| internal/pdf/merge/split.go | 17 |
| internal/pdf/redact/secure.go | 16 |
| internal/pdf/merge/merger.go | 15 |
| internal/pdf/redact/pdf_utils.go | 15 |
| internal/handlers/redact.go | 14 |
| typstsyntax/renderer.go | 14 |
| internal/pdf/merge/annotations.go | 10 |
| internal/pdf/outline.go | 10 |
| internal/pdf/svg/svg.go | 9 |
| internal/pdf/utils.go | 8 |
| internal/pdf/font/metrics.go | 7 |
| internal/pdf/redact/search.go | 6 |
| pkg/fontutils/fontutils.go | 6 |
| internal/pdf/font/registry.go | 5 |
| sampledata/gopdflib/zerodha/main.go | 5 |
| bindings/python/pypdfsuit/redact.py | 4 |
| internal/pdf/encryption/encrypt.go | 4 |
| internal/pdf/redact/visual.go | 4 |
| sampledata/filler/compressed/generate_medical_form.py | 4 |
| internal/pdf/font/pdfa.go | 3 |
| bindings/python/tests/test_integration.py | 2 |
| internal/pdf/helpers.go | 2 |
| internal/pdf/merge/parser.go | 2 |
| internal/pdf/redact/helpers.go | 2 |
| internal/pdf/redact/ocr_adapter.go | 2 |
| internal/pdf/redact/redactor.go | 2 |
| sampledata/benchmarks/fpdf/bench.py | 2 |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 2 |
| sampledata/financialreport/data/generate_charts.py | 2 |
| sampledata/gopdflib/financial_report/main.go | 2 |
| sampledata/librarybook/data/generate_codes.py | 2 |
| sampledata/python/amazonReceipt/amazonReceipt.py | 2 |
| sampledata/python/financial_report_pypdfsuit.py | 2 |
| sampledata/svg/generate_math_svg.py | 2 |
| internal/benchmarktemplates/zerodha_retail.go | 1 |
| internal/pdf/font/ttf.go | 1 |
| internal/pdf/links.go | 1 |
| internal/pdf/metadata.go | 1 |
| internal/pdf/pagemanager.go | 1 |
| internal/pdf/signature/signature.go | 1 |
| internal/pdf/types.go | 1 |
| internal/pdf/typst_math_test.go | 1 |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 1 |
| sampledata/benchmarks/pypdfsuit/bench.py | 1 |

### All Added Findings

| File | Line | Category | Message |
| --- | --- | --- | --- |
| bindings/python/pypdfsuit/redact.py | 71 | repeated_json_dumps_same_object | function apply_redactions calls json.dumps(redactions) multiple times; cache the result in a local variable |
| bindings/python/pypdfsuit/redact.py | 71 | repeated_json_dumps_same_object | function apply_redactions calls json.dumps(redactions) multiple times; cache the result in a local variable |
| bindings/python/pypdfsuit/redact.py | 123 | repeated_json_dumps_same_object | function apply_redactions_advanced calls json.dumps(options) multiple times; cache the result in a local variable |
| bindings/python/pypdfsuit/redact.py | 123 | repeated_json_dumps_same_object | function apply_redactions_advanced calls json.dumps(options) multiple times; cache the result in a local variable |
| bindings/python/pypdfsuit/types.py | 226 | option_bag_model | model Cell encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 175 | option_bag_model | model Config encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 447 | option_bag_model | model HtmlToImageRequest encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 425 | option_bag_model | model HtmlToPDFRequest encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 109 | option_bag_model | model PDFAConfig encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 379 | option_bag_model | model PDFTemplate encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 89 | option_bag_model | model SecurityConfig encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 125 | option_bag_model | model SignatureConfig encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 262 | option_bag_model | model Table encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 340 | option_bag_model | model Title encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 104 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 120 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 143 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 155 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 170 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 192 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 206 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 221 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 243 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 257 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 272 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 294 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 308 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 329 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 350 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 371 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 391 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 420 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 442 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 464 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 476 | public_any_type_leak | public function to_dict exposes a very wide type contract |
| bindings/python/tests/test_integration.py | 55 | repeated_json_dumps_same_object | function _generate_pdf_from_dict calls json.dumps(template_dict) multiple times; cache the result in a local variable |
| bindings/python/tests/test_integration.py | 55 | repeated_json_dumps_same_object | function _generate_pdf_from_dict calls json.dumps(template_dict) multiple times; cache the result in a local variable |
| internal/benchmarktemplates/zerodha_retail.go | 43 | stable_value_normalization_in_inner_loop | function readChain normalizes a stable value inside a loop |
| internal/handlers/handlers.go | 276 | large_h_payload_built_only_for_json_response | function handleGenerateTemplatePDF builds a large dynamic map payload for JSON rendering |
| internal/handlers/handlers.go | 181 | large_h_payload_built_only_for_json_response | function handleGetTemplateData builds a large dynamic map payload for JSON rendering |
| internal/handlers/handlers.go | 213 | servefile_via_readfile_then_c_data | function handleGetTemplateData reads a file into memory and then writes it through gin.Context.Data |
| internal/handlers/handlers.go | 198 | file_or_template_read_per_request | function handleGetTemplateData reads files directly on a request path |
| internal/handlers/handlers.go | 179 | rows_without_close | function handleGetTemplateData uses rows handle filename without an observed Close() call |
| internal/handlers/handlers.go | 542 | large_h_payload_built_only_for_json_response | function handleHTMLToImage builds a large dynamic map payload for JSON rendering |
| internal/handlers/handlers.go | 484 | large_h_payload_built_only_for_json_response | function handleHTMLToPDF builds a large dynamic map payload for JSON rendering |
| internal/handlers/handlers.go | 385 | slice_append_without_prealloc_known_bound | function handleMergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/handlers/handlers.go | 360 | large_h_payload_built_only_for_json_response | function handleMergePDFs builds a large dynamic map payload for JSON rendering |
| internal/handlers/handlers.go | 376 | repeated_c_json_inside_stream_loop | function handleMergePDFs writes Gin JSON responses from inside a loop |
| internal/handlers/handlers.go | 382 | repeated_c_json_inside_stream_loop | function handleMergePDFs writes Gin JSON responses from inside a loop |
| internal/handlers/handlers.go | 230 | large_h_payload_built_only_for_json_response | function handleUploadFont builds a large dynamic map payload for JSON rendering |
| internal/handlers/handlers.go | 251 | formfile_open_readall_whole_upload | function handleUploadFont reads an uploaded form file fully into memory |
| internal/handlers/handlers.go | 405 | large_h_payload_built_only_for_json_response | function handlerSplitPDF builds a large dynamic map payload for JSON rendering |
| internal/handlers/handlers.go | 411 | formfile_open_readall_whole_upload | function handlerSplitPDF reads an uploaded form file fully into memory |
| internal/handlers/handlers.go | 453 | bytes_buffer_without_grow_known_bound | function handlerSplitPDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/handlers/handlers.go | 460 | repeated_c_json_inside_stream_loop | function handlerSplitPDF writes Gin JSON responses from inside a loop |
| internal/handlers/handlers.go | 465 | repeated_c_json_inside_stream_loop | function handlerSplitPDF writes Gin JSON responses from inside a loop |
| internal/handlers/redact.go | 188 | large_h_payload_built_only_for_json_response | function HandleRedactApply builds a large dynamic map payload for JSON rendering |
| internal/handlers/redact.go | 290 | json_marshaled_manually_then_c_data | function HandleRedactApply marshals JSON manually before writing through gin.Context.Data |
| internal/handlers/redact.go | 290 | no_streaming_for_large_export_handler | function HandleRedactApply materializes a collection into memory before writing the response |
| internal/handlers/redact.go | 265 | formfile_open_readall_whole_upload | function HandleRedactApply reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 99 | large_h_payload_built_only_for_json_response | function HandleRedactCapabilities builds a large dynamic map payload for JSON rendering |
| internal/handlers/redact.go | 110 | formfile_open_readall_whole_upload | function HandleRedactCapabilities reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 60 | large_h_payload_built_only_for_json_response | function HandleRedactPageInfo builds a large dynamic map payload for JSON rendering |
| internal/handlers/redact.go | 71 | formfile_open_readall_whole_upload | function HandleRedactPageInfo reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 297 | large_h_payload_built_only_for_json_response | function HandleRedactSearch builds a large dynamic map payload for JSON rendering |
| internal/handlers/redact.go | 330 | formfile_open_readall_whole_upload | function HandleRedactSearch reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 138 | large_h_payload_built_only_for_json_response | function HandleRedactTextPositions builds a large dynamic map payload for JSON rendering |
| internal/handlers/redact.go | 160 | formfile_open_readall_whole_upload | function HandleRedactTextPositions reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 45 | stable_value_normalization_in_inner_loop | function normalizeTextSearchQueries normalizes a stable value inside a loop |
| internal/handlers/redact.go | 23 | stable_value_normalization_in_inner_loop | function parseCommaSeparatedTerms normalizes a stable value inside a loop |
| internal/pdf/draw.go | 1082 | likely_n_squared_string_concat | function drawTable appears to concatenate strings inside a nested loop |
| internal/pdf/draw.go | 902 | append_then_trim_each_iteration | function drawTable appends and then reslices in a loop |
| internal/pdf/draw.go | 902 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 904 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 906 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 911 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 913 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 915 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 917 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 943 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 945 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 947 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 949 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 954 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 956 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 958 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 960 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 964 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 965 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 966 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 983 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 985 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 987 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 989 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 998 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 999 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1007 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1009 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1012 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1013 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1014 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1044 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1046 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1048 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1050 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1056 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1058 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1060 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1062 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1064 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1066 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1068 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1070 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1140 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1142 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1144 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1189 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1191 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1195 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1196 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1223 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1225 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1237 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1239 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1241 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1243 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1255 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1257 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1262 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1263 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1275 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1277 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1279 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1281 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1283 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1289 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1291 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1293 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1295 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1297 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1303 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1305 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1307 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1309 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1311 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1317 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1319 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1321 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1323 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1325 | slice_append_without_prealloc_known_bound | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 938 | stable_value_normalization_in_inner_loop | function drawTable normalizes a stable value inside a loop |
| internal/pdf/draw.go | 758 | filter_then_count_then_iterate | function drawTable traverses the same collection multiple times for filter, count, and process |
| internal/pdf/draw.go | 392 | append_then_trim_each_iteration | function drawTitleTable appends and then reslices in a loop |
| internal/pdf/draw.go | 392 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 394 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 396 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 401 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 403 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 405 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 407 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 474 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 476 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 478 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 480 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 485 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 487 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 489 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 491 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 495 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 496 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 497 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 513 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 515 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 517 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 519 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 528 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 529 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 537 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 539 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 542 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 543 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 544 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 566 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 568 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 570 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 596 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 598 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 610 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 612 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 614 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 616 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 627 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 629 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 637 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 638 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 649 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 651 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 653 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 655 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 657 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 663 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 665 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 667 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 669 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 671 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 677 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 679 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 681 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 683 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 685 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 691 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 693 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 695 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 697 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 699 | slice_append_without_prealloc_known_bound | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 471 | stable_value_normalization_in_inner_loop | function drawTitleTable normalizes a stable value inside a loop |
| internal/pdf/draw.go | 337 | filter_then_count_then_iterate | function drawTitleTable traverses the same collection multiple times for filter, count, and process |
| internal/pdf/encryption/encrypt.go | 99 | likely_n_squared_allocation | function computeOwnerHash appears to allocate inside a nested loop |
| internal/pdf/encryption/encrypt.go | 99 | make_slice_inside_hot_loop_same_shape | function computeOwnerHash recreates scratch slices inside a loop |
| internal/pdf/encryption/encrypt.go | 154 | likely_n_squared_allocation | function computeUserHash appears to allocate inside a nested loop |
| internal/pdf/encryption/encrypt.go | 154 | make_slice_inside_hot_loop_same_shape | function computeUserHash recreates scratch slices inside a loop |
| internal/pdf/font/metrics.go | 940 | slice_append_without_prealloc_known_bound | function GenerateToUnicodeCMap appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/metrics.go | 950 | strings_builder_without_grow_known_bound | function GenerateToUnicodeCMap uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/metrics.go | 543 | strings_builder_without_grow_known_bound | function GenerateWidthsArrayObject uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/metrics.go | 632 | slice_append_without_prealloc_known_bound | function GetAvailableFonts appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/metrics.go | 567 | strings_builder_without_grow_known_bound | function GetHelveticaFontResourceString uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/metrics.go | 796 | slice_append_without_prealloc_known_bound | function generateCIDWidths appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/metrics.go | 810 | strings_builder_without_grow_known_bound | function generateCIDWidths uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/pdfa.go | 48 | mutable_package_global | package variable LiberationFontFiles is mutated across function bodies |
| internal/pdf/font/pdfa.go | 27 | mutable_package_global | package variable LiberationFontMapping is mutated across function bodies |
| internal/pdf/font/pdfa.go | 392 | public_bool_parameter_api | public function GetMappedFontName uses boolean parameter to control behavior |
| internal/pdf/font/registry.go | 239 | make_map_inside_hot_loop_same_shape | function CloneForGeneration recreates scratch maps inside a loop |
| internal/pdf/font/registry.go | 387 | strings_builder_without_grow_known_bound | function GeneratePDFFontResources uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/registry.go | 148 | likely_n_squared_allocation | function GenerateSubsets appears to allocate inside a nested loop |
| internal/pdf/font/registry.go | 148 | make_slice_inside_hot_loop_same_shape | function GenerateSubsets recreates scratch slices inside a loop |
| internal/pdf/font/registry.go | 208 | make_map_inside_hot_loop_same_shape | function ResetUsage recreates scratch maps inside a loop |
| internal/pdf/font/subset.go | 23 | map_growth_without_size_hint | function SubsetTTF inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 42 | filter_then_count_then_iterate | function SubsetTTF traverses the same collection multiple times for filter, count, and process |
| internal/pdf/font/subset.go | 111 | likely_n_squared_allocation | function buildSubsetFont appears to allocate inside a nested loop |
| internal/pdf/font/subset.go | 163 | slice_append_without_prealloc_known_bound | function buildSubsetFont appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/subset.go | 111 | map_growth_without_size_hint | function buildSubsetFont inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 111 | make_slice_inside_hot_loop_same_shape | function buildSubsetFont recreates scratch slices inside a loop |
| internal/pdf/font/subset.go | 58 | bytes_buffer_without_grow_known_bound | function buildSubsetFont uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 680 | bytes_buffer_without_grow_known_bound | function encodeUTF16BE uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 396 | slice_append_without_prealloc_known_bound | function subsetCmap appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/subset.go | 358 | map_growth_without_size_hint | function subsetCmap inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 452 | filter_then_count_then_iterate | function subsetCmap traverses the same collection multiple times for filter, count, and process |
| internal/pdf/font/subset.go | 351 | bytes_buffer_without_grow_known_bound | function subsetCmap uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 271 | map_growth_without_size_hint | function subsetGlyfAndLoca inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 292 | make_slice_inside_hot_loop_same_shape | function subsetGlyfAndLoca recreates scratch slices inside a loop |
| internal/pdf/font/subset.go | 265 | bytes_buffer_without_grow_known_bound | function subsetGlyfAndLoca uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 332 | bytes_buffer_without_grow_known_bound | function subsetHmtx uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 600 | slice_append_without_prealloc_known_bound | function subsetName appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/subset.go | 567 | bytes_buffer_without_grow_known_bound | function subsetName uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/ttf.go | 761 | map_growth_without_size_hint | function GetUsedGlyphs inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 685 | slice_append_without_prealloc_known_bound | function DetectFormFields appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 526 | slice_append_without_prealloc_known_bound | function DetectFormFieldsAdvanced appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 497 | regexp_compile_in_hot_path | function DetectFormFieldsAdvanced compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 509 | regexp_compile_in_hot_path | function DetectFormFieldsAdvanced compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 547 | map_growth_without_size_hint | function DetectFormFieldsAdvanced inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 893 | append_then_trim_each_iteration | function FillPDFWithXFDF appends and then reslices in a loop |
| internal/pdf/form/xfdf.go | 849 | slice_append_without_prealloc_known_bound | function FillPDFWithXFDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 953 | slice_append_without_prealloc_known_bound | function FillPDFWithXFDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 891 | repeated_slice_clone_in_loop | function FillPDFWithXFDF clones slices inside a loop |
| internal/pdf/form/xfdf.go | 907 | repeated_slice_clone_in_loop | function FillPDFWithXFDF clones slices inside a loop |
| internal/pdf/form/xfdf.go | 928 | repeated_slice_clone_in_loop | function FillPDFWithXFDF clones slices inside a loop |
| internal/pdf/form/xfdf.go | 878 | regexp_compile_in_hot_path | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 886 | regexp_compile_in_hot_path | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 903 | regexp_compile_in_hot_path | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 909 | regexp_compile_in_hot_path | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 914 | regexp_compile_in_hot_path | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 924 | regexp_compile_in_hot_path | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 965 | regexp_compile_in_hot_path | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 891 | byte_string_conversion_in_loop | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 907 | byte_string_conversion_in_loop | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 928 | byte_string_conversion_in_loop | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1015 | byte_string_conversion_in_loop | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1038 | byte_string_conversion_in_loop | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1051 | byte_string_conversion_in_loop | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1058 | strconv_repeat_on_same_binding | function FillPDFWithXFDF converts the same string input with strconv multiple times |
| internal/pdf/form/xfdf.go | 874 | map_growth_without_size_hint | function FillPDFWithXFDF inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 1040 | builder_or_buffer_recreated_per_iteration | function FillPDFWithXFDF recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 1020 | builder_or_buffer_recreated_per_iteration | function FillPDFWithXFDF recreates a strings.Builder inside a loop |
| internal/pdf/form/xfdf.go | 872 | filter_then_count_then_iterate | function FillPDFWithXFDF traverses the same collection multiple times for filter, count, and process |
| internal/pdf/form/xfdf.go | 1040 | bytes_buffer_without_grow_known_bound | function FillPDFWithXFDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 1020 | strings_builder_without_grow_known_bound | function FillPDFWithXFDF uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 709 | map_growth_without_size_hint | function FillPDFWithXFDFAdvanced inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 60 | map_growth_without_size_hint | function ParseXFDF inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 659 | builder_or_buffer_recreated_per_iteration | function decompressStreams recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 659 | bytes_buffer_without_grow_known_bound | function decompressStreams uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 624 | map_growth_without_size_hint | function detectFormFieldsNaive inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 1239 | regexp_compile_in_hot_path | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1252 | regexp_compile_in_hot_path | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1254 | regexp_compile_in_hot_path | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1260 | regexp_compile_in_hot_path | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1286 | regexp_compile_in_hot_path | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1251 | map_growth_without_size_hint | function fillXFDFInObjStmBody inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 1318 | builder_or_buffer_recreated_per_iteration | function fillXFDFInObjStmBody recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 1336 | builder_or_buffer_recreated_per_iteration | function fillXFDFInObjStmBody recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 1318 | bytes_buffer_without_grow_known_bound | function fillXFDFInObjStmBody uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 1356 | strings_builder_without_grow_known_bound | function fillXFDFInObjStmBody uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 348 | regexp_compile_in_hot_path | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 360 | regexp_compile_in_hot_path | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 362 | regexp_compile_in_hot_path | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 364 | regexp_compile_in_hot_path | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 369 | regexp_compile_in_hot_path | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 402 | regexp_compile_in_hot_path | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 441 | regexp_compile_in_hot_path | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/generator.go | 667 | append_then_trim_each_iteration | function GenerateTemplatePDF appends and then reslices in a loop |
| internal/pdf/generator.go | 427 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 667 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 674 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 676 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 678 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 680 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 693 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 746 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 895 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 907 | slice_append_without_prealloc_known_bound | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 162 | map_growth_without_size_hint | function GenerateTemplatePDF inserts into a map in a loop without a visible size hint |
| internal/pdf/generator.go | 606 | stable_value_normalization_in_inner_loop | function GenerateTemplatePDF normalizes a stable value inside a loop |
| internal/pdf/generator.go | 697 | builder_or_buffer_recreated_per_iteration | function GenerateTemplatePDF recreates a strings.Builder inside a loop |
| internal/pdf/generator.go | 157 | filter_then_count_then_iterate | function GenerateTemplatePDF traverses the same collection multiple times for filter, count, and process |
| internal/pdf/generator.go | 83 | bytes_buffer_without_grow_known_bound | function GenerateTemplatePDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/generator.go | 498 | strings_builder_without_grow_known_bound | function GenerateTemplatePDF uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/generator.go | 1628 | filter_then_count_then_iterate | function collectAllStandardFontsInTemplate traverses the same collection multiple times for filter, count, and process |
| internal/pdf/generator.go | 1784 | filter_then_count_then_iterate | function collectUnregisteredMathFontNames traverses the same collection multiple times for filter, count, and process |
| internal/pdf/generator.go | 1503 | filter_then_count_then_iterate | function collectUsedStandardFonts traverses the same collection multiple times for filter, count, and process |
| internal/pdf/generator.go | 1369 | filter_then_count_then_iterate | function generateAllContentWithImages traverses the same collection multiple times for filter, count, and process |
| internal/pdf/helpers.go | 108 | regexp_compile_in_hot_path | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/helpers.go | 147 | regexp_compile_in_hot_path | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/links.go | 80 | public_bool_parameter_api | public function ParseLink uses boolean parameter to control behavior |
| internal/pdf/merge.go | 55 | likely_n_squared_allocation | function MergePDFs appears to allocate inside a nested loop |
| internal/pdf/merge.go | 69 | likely_n_squared_allocation | function MergePDFs appears to allocate inside a nested loop |
| internal/pdf/merge.go | 100 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 118 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 130 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 145 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 152 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 180 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 197 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 55 | repeated_map_clone_in_loop | function MergePDFs clones maps inside a loop |
| internal/pdf/merge.go | 209 | regexp_compile_in_hot_path | function MergePDFs compiles regular expressions inside a loop |
| internal/pdf/merge.go | 61 | map_growth_without_size_hint | function MergePDFs inserts into a map in a loop without a visible size hint |
| internal/pdf/merge.go | 55 | make_map_inside_hot_loop_same_shape | function MergePDFs recreates scratch maps inside a loop |
| internal/pdf/merge.go | 69 | make_map_inside_hot_loop_same_shape | function MergePDFs recreates scratch maps inside a loop |
| internal/pdf/merge.go | 72 | filter_then_count_then_iterate | function MergePDFs traverses the same collection multiple times for filter, count, and process |
| internal/pdf/merge.go | 16 | bytes_buffer_without_grow_known_bound | function MergePDFs uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/merge.go | 352 | nested_append_without_outer_capacity | function extractFormFieldsFromFile appends inside nested loops without visible preallocation |
| internal/pdf/merge.go | 327 | slice_append_without_prealloc_known_bound | function extractFormFieldsFromFile appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 352 | slice_append_without_prealloc_known_bound | function extractFormFieldsFromFile appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 343 | regexp_compile_in_hot_path | function extractFormFieldsFromFile compiles regular expressions inside a loop |
| internal/pdf/merge.go | 345 | regexp_compile_in_hot_path | function extractFormFieldsFromFile compiles regular expressions inside a loop |
| internal/pdf/merge.go | 347 | strconv_repeat_on_same_binding | function extractFormFieldsFromFile converts the same string input with strconv multiple times |
| internal/pdf/merge.go | 328 | map_growth_without_size_hint | function extractFormFieldsFromFile inserts into a map in a loop without a visible size hint |
| internal/pdf/merge.go | 281 | strconv_repeat_on_same_binding | function replaceRefsOutsideStreams converts the same string input with strconv multiple times |
| internal/pdf/merge.go | 253 | bytes_buffer_without_grow_known_bound | function replaceRefsOutsideStreams uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/merge/annotations.go | 71 | nested_append_without_outer_capacity | function ExtractAPDependencies appends inside nested loops without visible preallocation |
| internal/pdf/merge/annotations.go | 64 | slice_append_without_prealloc_known_bound | function ExtractAPDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 71 | slice_append_without_prealloc_known_bound | function ExtractAPDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 65 | map_growth_without_size_hint | function ExtractAPDependencies inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/annotations.go | 22 | slice_append_without_prealloc_known_bound | function ExtractAnnotationsFromPage appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 36 | slice_append_without_prealloc_known_bound | function ExtractAnnotationsFromPage appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 35 | strconv_repeat_on_same_binding | function ExtractAnnotationsFromPage converts the same string input with strconv multiple times |
| internal/pdf/merge/annotations.go | 105 | map_growth_without_size_hint | function ExtractFormFields inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/annotations.go | 310 | regexp_compile_in_hot_path | function collectDepsRecursive compiles regular expressions inside a loop |
| internal/pdf/merge/annotations.go | 185 | strconv_repeat_on_same_binding | function extractFieldsArray converts the same string input with strconv multiple times |
| internal/pdf/merge/merger.go | 38 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 77 | slice_append_without_prealloc_known_bound | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 300 | nested_append_without_outer_capacity | function collectObjectsWithDependencies appends inside nested loops without visible preallocation |
| internal/pdf/merge/merger.go | 290 | slice_append_without_prealloc_known_bound | function collectObjectsWithDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 300 | slice_append_without_prealloc_known_bound | function collectObjectsWithDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 276 | map_growth_without_size_hint | function collectObjectsWithDependencies inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/merger.go | 245 | slice_append_without_prealloc_known_bound | function extractKidsRecursive appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 251 | slice_append_without_prealloc_known_bound | function extractKidsRecursive appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 254 | slice_append_without_prealloc_known_bound | function extractKidsRecursive appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 322 | slice_append_without_prealloc_known_bound | function writeCatalog appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 341 | slice_append_without_prealloc_known_bound | function writePages appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 417 | nested_append_without_outer_capacity | function writeXRefAndTrailer appends inside nested loops without visible preallocation |
| internal/pdf/merge/merger.go | 417 | slice_append_without_prealloc_known_bound | function writeXRefAndTrailer appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 419 | slice_append_without_prealloc_known_bound | function writeXRefAndTrailer appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 420 | slice_append_without_prealloc_known_bound | function writeXRefAndTrailer appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/parser.go | 501 | map_growth_without_size_hint | function ParseObjectStream inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/parser.go | 300 | bytes_buffer_without_grow_known_bound | function ReplaceRefsOutsideStreams uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/merge/split.go | 62 | slice_append_without_prealloc_known_bound | function ParsePageSpec appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 47 | map_growth_without_size_hint | function ParsePageSpec inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/split.go | 101 | slice_append_without_prealloc_known_bound | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 110 | slice_append_without_prealloc_known_bound | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 124 | slice_append_without_prealloc_known_bound | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 137 | slice_append_without_prealloc_known_bound | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 149 | slice_append_without_prealloc_known_bound | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 125 | map_growth_without_size_hint | function SplitPDF inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/split.go | 163 | append_then_trim_each_iteration | function buildPDFFromPageObjs appends and then reslices in a loop |
| internal/pdf/merge/split.go | 163 | slice_append_without_prealloc_known_bound | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 178 | slice_append_without_prealloc_known_bound | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 193 | slice_append_without_prealloc_known_bound | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 203 | slice_append_without_prealloc_known_bound | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 236 | slice_append_without_prealloc_known_bound | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 246 | slice_append_without_prealloc_known_bound | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 162 | map_growth_without_size_hint | function buildPDFFromPageObjs inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/split.go | 183 | filter_then_count_then_iterate | function buildPDFFromPageObjs traverses the same collection multiple times for filter, count, and process |
| internal/pdf/metadata.go | 76 | strings_builder_without_grow_known_bound | function GenerateXMPMetadata uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/outline.go | 478 | uuid_hash_formatting_only_for_logs | function GetNamedDestinations formats identifiers inside a loop only for logging |
| internal/pdf/outline.go | 439 | strings_builder_without_grow_known_bound | function GetNamedDestinations uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/outline.go | 409 | strings_builder_without_grow_known_bound | function escapeTextUnicode uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/outline.go | 345 | slice_append_without_prealloc_known_bound | function generateOutlineObjects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/outline.go | 348 | slice_append_without_prealloc_known_bound | function generateOutlineObjects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/outline.go | 353 | slice_append_without_prealloc_known_bound | function generateOutlineObjects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/outline.go | 362 | uuid_hash_formatting_only_for_logs | function generateOutlineObjects formats identifiers inside a loop only for logging |
| internal/pdf/outline.go | 328 | builder_or_buffer_recreated_per_iteration | function generateOutlineObjects recreates a strings.Builder inside a loop |
| internal/pdf/outline.go | 327 | filter_then_count_then_iterate | function generateOutlineObjects traverses the same collection multiple times for filter, count, and process |
| internal/pdf/outline.go | 316 | strings_builder_without_grow_known_bound | function generateOutlineObjects uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/pagemanager.go | 42 | public_bool_parameter_api | public function NewPageManager uses boolean parameter to control behavior |
| internal/pdf/redact/helpers.go | 105 | regexp_compile_in_hot_path | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/redact/helpers.go | 137 | regexp_compile_in_hot_path | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/redact/ocr_adapter.go | 63 | slice_append_without_prealloc_known_bound | function runOCRSearch appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/ocr_adapter.go | 29 | single_impl_interface | interface OCRProvider currently has one obvious repository-local implementation |
| internal/pdf/redact/pdf_utils.go | 28 | regexp_compile_in_hot_path | function buildObjectMap compiles regular expressions inside a loop |
| internal/pdf/redact/pdf_utils.go | 38 | regexp_compile_in_hot_path | function buildObjectMap compiles regular expressions inside a loop |
| internal/pdf/redact/pdf_utils.go | 68 | map_growth_without_size_hint | function buildObjectMap inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/pdf_utils.go | 641 | bytes_buffer_without_grow_known_bound | function decodePDFLiteral uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 576 | strings_builder_without_grow_known_bound | function decodeTJArray uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 425 | byte_string_conversion_in_loop | function extractKidsRefs converts between bytes and strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 215 | slice_append_without_prealloc_known_bound | function extractPageContent appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/pdf_utils.go | 215 | byte_string_conversion_in_loop | function extractPageContent converts between bytes and strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 219 | bytes_buffer_without_grow_known_bound | function extractPageContent uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 527 | slice_append_without_prealloc_known_bound | function parseTextOperators appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/pdf_utils.go | 506 | strconv_repeat_on_same_binding | function parseTextOperators converts the same string input with strconv multiple times |
| internal/pdf/redact/pdf_utils.go | 881 | slice_append_without_prealloc_known_bound | function rebuildPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/pdf_utils.go | 839 | map_growth_without_size_hint | function rebuildPDF inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/pdf_utils.go | 824 | bytes_buffer_without_grow_known_bound | function rebuildPDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 363 | map_growth_without_size_hint | function resolveUsedXObjectRefs inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/redactor.go | 138 | likely_n_squared_allocation | function AnalyzePageCapabilities appears to allocate inside a nested loop |
| internal/pdf/redact/redactor.go | 138 | make_slice_inside_hot_loop_same_shape | function AnalyzePageCapabilities recreates scratch slices inside a loop |
| internal/pdf/redact/search.go | 69 | slice_append_without_prealloc_known_bound | function FindTextOccurrences appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 77 | slice_append_without_prealloc_known_bound | function FindTextOccurrences appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 95 | stable_value_normalization_in_inner_loop | function FindTextOccurrencesMulti normalizes a stable value inside a loop |
| internal/pdf/redact/search.go | 300 | slice_append_without_prealloc_known_bound | function findAllCombinedMatchRects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 303 | slice_append_without_prealloc_known_bound | function findAllCombinedMatchRects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 400 | slice_append_without_prealloc_known_bound | function findAllCombinedMatchRects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 312 | filter_then_count_then_iterate | function applyRectMaskToText traverses the same collection multiple times for filter, count, and process |
| internal/pdf/redact/secure.go | 54 | likely_n_squared_allocation | function applySecureContentRedactions appears to allocate inside a nested loop |
| internal/pdf/redact/secure.go | 43 | slice_append_without_prealloc_known_bound | function applySecureContentRedactions appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 50 | slice_append_without_prealloc_known_bound | function applySecureContentRedactions appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 62 | slice_append_without_prealloc_known_bound | function applySecureContentRedactions appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 27 | map_growth_without_size_hint | function applySecureContentRedactions inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/secure.go | 54 | make_map_inside_hot_loop_same_shape | function applySecureContentRedactions recreates scratch maps inside a loop |
| internal/pdf/redact/secure.go | 422 | slice_append_without_prealloc_known_bound | function buildRedactionTJArray appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 428 | slice_append_without_prealloc_known_bound | function buildRedactionTJArray appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 399 | filter_then_count_then_iterate | function buildRedactionTJArray traverses the same collection multiple times for filter, count, and process |
| internal/pdf/redact/secure.go | 375 | strings_builder_without_grow_known_bound | function buildRedactionTJArray uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/redact/secure.go | 155 | slice_append_without_prealloc_known_bound | function extractContentKeys appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 155 | byte_string_conversion_in_loop | function extractContentKeys converts between bytes and strings inside a loop |
| internal/pdf/redact/secure.go | 109 | regexp_compile_in_hot_path | function rewriteSecureStreamTree compiles regular expressions inside a loop |
| internal/pdf/redact/secure.go | 262 | stable_value_normalization_in_inner_loop | function scrubDecodedContent normalizes a stable value inside a loop |
| internal/pdf/redact/secure.go | 221 | strings_builder_without_grow_known_bound | function scrubDecodedContent uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/redact/visual.go | 32 | map_growth_without_size_hint | function ApplyRedactions inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/visual.go | 55 | builder_or_buffer_recreated_per_iteration | function ApplyRedactions recreates a strings.Builder inside a loop |
| internal/pdf/redact/visual.go | 47 | filter_then_count_then_iterate | function ApplyRedactions traverses the same collection multiple times for filter, count, and process |
| internal/pdf/redact/visual.go | 55 | strings_builder_without_grow_known_bound | function ApplyRedactions uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/signature/signature.go | 482 | slice_append_without_prealloc_known_bound | function createPKCS7SignedData appends to a slice inside a range loop without visible preallocation |
| internal/pdf/svg/svg.go | 110 | likely_n_squared_allocation | function ConvertSVGToPDFCommands appears to allocate inside a nested loop |
| internal/pdf/svg/svg.go | 110 | repeated_map_clone_in_loop | function ConvertSVGToPDFCommands clones maps inside a loop |
| internal/pdf/svg/svg.go | 98 | map_growth_without_size_hint | function ConvertSVGToPDFCommands inserts into a map in a loop without a visible size hint |
| internal/pdf/svg/svg.go | 110 | make_map_inside_hot_loop_same_shape | function ConvertSVGToPDFCommands recreates scratch maps inside a loop |
| internal/pdf/svg/svg.go | 67 | bytes_buffer_without_grow_known_bound | function ConvertSVGToPDFCommands uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/svg/svg.go | 257 | strconv_repeat_on_same_binding | function applyTransform converts the same string input with strconv multiple times |
| internal/pdf/svg/svg.go | 258 | strconv_repeat_on_same_binding | function applyTransform converts the same string input with strconv multiple times |
| internal/pdf/svg/svg.go | 475 | strconv_repeat_on_same_binding | function parsePathData converts the same string input with strconv multiple times |
| internal/pdf/svg/svg.go | 207 | map_growth_without_size_hint | function processElement inserts into a map in a loop without a visible size hint |
| internal/pdf/types.go | 9 | mutable_package_global | package variable pageSizes is mutated across function bodies |
| internal/pdf/typst_math_test.go | 290 | likely_n_squared_string_concat | function TestTypstMathStress_GenerateEquationBankPDF appears to concatenate strings inside a nested loop |
| internal/pdf/utils.go | 402 | slice_append_without_prealloc_known_bound | function WrapText appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 406 | slice_append_without_prealloc_known_bound | function WrapText appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 424 | slice_append_without_prealloc_known_bound | function WrapText appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 311 | slice_append_without_prealloc_known_bound | function formatPageKids appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 314 | slice_append_without_prealloc_known_bound | function formatPageKids appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 142 | strconv_repeat_on_same_binding | function parseProps converts the same string input with strconv multiple times |
| internal/pdf/utils.go | 470 | byte_string_conversion_in_loop | function wrapLongWord converts between bytes and strings inside a loop |
| internal/pdf/utils.go | 14 | mutable_package_global | package variable hexNibble is mutated across function bodies |
| pkg/fontutils/fontutils.go | 149 | defer_in_loop_resource_growth | function EnsureMathFonts defers cleanup inside a loop |
| pkg/fontutils/fontutils.go | 110 | slice_append_without_prealloc_known_bound | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 112 | slice_append_without_prealloc_known_bound | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 114 | slice_append_without_prealloc_known_bound | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 117 | slice_append_without_prealloc_known_bound | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 120 | slice_append_without_prealloc_known_bound | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| sampledata/benchmarks/fpdf/bench.py | 36 | exception_swallowed | function run_once suppresses a broad exception instead of surfacing or narrowing it |
| sampledata/benchmarks/fpdf/bench.py | 11 | import_time_file_io | module performs filesystem work while being imported |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 65 | slice_append_without_prealloc_known_bound | function buildRows appends to a slice inside a range loop without visible preallocation |
| sampledata/benchmarks/pypdfsuit/bench.py | 37 | repeated_string_format_invariant_template | function main formats a string inside a loop; consider building the template once |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 21 | no_schema_validation_on_external_data | function load_data parses external data without schema validation; corrupt input propagates silently |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 108 | repeated_string_format_invariant_template | function main formats a string inside a loop; consider building the template once |
| sampledata/filler/compressed/generate_medical_form.py | 200 | repeated_string_format_invariant_template | function construct_object_stream formats a string inside a loop; consider building the template once |
| sampledata/filler/compressed/generate_medical_form.py | 80 | repeated_string_format_invariant_template | function generate_pdf formats a string inside a loop; consider building the template once |
| sampledata/filler/compressed/generate_medical_form.py | 106 | string_join_without_generator | function generate_pdf passes a list comprehension to .join(); use a generator expression to avoid an intermediate list |
| sampledata/filler/compressed/generate_medical_form.py | 136 | string_join_without_generator | function generate_pdf passes a list comprehension to .join(); use a generator expression to avoid an intermediate list |
| sampledata/financialreport/data/generate_charts.py | 37 | print_debugging_leftover | function generate_bar_chart leaves print()-based debugging in production code |
| sampledata/financialreport/data/generate_charts.py | 63 | print_debugging_leftover | function generate_pie_chart leaves print()-based debugging in production code |
| sampledata/gopdflib/financial_report/main.go | 133 | slice_append_without_prealloc_known_bound | function main appends to a slice inside a range loop without visible preallocation |
| sampledata/gopdflib/financial_report/main.go | 88 | defer_in_loop_resource_growth | function main defers cleanup inside a loop |
| sampledata/gopdflib/zerodha/main.go | 750 | slice_append_without_prealloc_known_bound | function main appends to a slice inside a range loop without visible preallocation |
| sampledata/gopdflib/zerodha/main.go | 686 | defer_in_loop_resource_growth | function main defers cleanup inside a loop |
| sampledata/gopdflib/zerodha/main.go | 739 | filter_then_count_then_iterate | function main traverses the same collection multiple times for filter, count, and process |
| sampledata/gopdflib/zerodha/main.go | 68 | mutable_package_global | package variable actions is mutated across function bodies |
| sampledata/gopdflib/zerodha/main.go | 60 | mutable_package_global | package variable symbols is mutated across function bodies |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 268 | repeated_string_format_invariant_template | function build_active_trader_template formats a string inside a loop; consider building the template once |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 353 | repeated_string_format_invariant_template | function build_hft_template formats a string inside a loop; consider building the template once |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 85 | repeated_string_format_invariant_template | function generate_trades formats a string inside a loop; consider building the template once |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 54 | repeated_string_format_invariant_template | function read_chain formats a string inside a loop; consider building the template once |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 416 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 417 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 418 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 420 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 431 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 433 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 437 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 439 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 443 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 444 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 445 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 446 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 483 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 484 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 485 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 486 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 487 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 488 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 489 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 490 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 491 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 492 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 493 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 494 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 495 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 496 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 497 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 498 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 499 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 500 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 501 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 502 | print_debugging_leftover | function run_benchmark leaves print()-based debugging in production code |
| sampledata/librarybook/data/generate_codes.py | 69 | print_debugging_leftover | function generate_barcode leaves print()-based debugging in production code |
| sampledata/librarybook/data/generate_codes.py | 36 | print_debugging_leftover | function generate_qr_code leaves print()-based debugging in production code |
| sampledata/python/amazonReceipt/amazonReceipt.py | 378 | repeated_json_dumps_same_object | function generate_pdf_with_page_margin calls json.dumps(template_payload) multiple times; cache the result in a local variable |
| sampledata/python/amazonReceipt/amazonReceipt.py | 378 | repeated_json_dumps_same_object | function generate_pdf_with_page_margin calls json.dumps(template_payload) multiple times; cache the result in a local variable |
| sampledata/python/financial_report_pypdfsuit.py | 194 | repeated_string_format_invariant_template | function build_financial_report_template formats a string inside a loop; consider building the template once |
| sampledata/python/financial_report_pypdfsuit.py | 371 | repeated_string_format_invariant_template | function main formats a string inside a loop; consider building the template once |
| sampledata/svg/generate_math_svg.py | 79 | print_debugging_leftover | function save_math_svg leaves print()-based debugging in production code |
| sampledata/svg/generate_math_svg.py | 80 | print_debugging_leftover | function save_math_svg leaves print()-based debugging in production code |
| typstsyntax/renderer.go | 741 | slice_append_without_prealloc_known_bound | function layoutBigOperatorLimits appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 752 | slice_append_without_prealloc_known_bound | function layoutBigOperatorLimits appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 762 | slice_append_without_prealloc_known_bound | function layoutBigOperatorLimits appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 760 | filter_then_count_then_iterate | function layoutBigOperatorLimits traverses the same collection multiple times for filter, count, and process |
| typstsyntax/renderer.go | 846 | slice_append_without_prealloc_known_bound | function layoutCases appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 900 | slice_append_without_prealloc_known_bound | function layoutLR appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 920 | slice_append_without_prealloc_known_bound | function layoutLR appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 575 | slice_append_without_prealloc_known_bound | function layoutMatrix appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 477 | likely_n_squared_allocation | function layoutMatrixGrid appears to allocate inside a nested loop |
| typstsyntax/renderer.go | 547 | slice_append_without_prealloc_known_bound | function layoutMatrixGrid appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 477 | make_slice_inside_hot_loop_same_shape | function layoutMatrixGrid recreates scratch slices inside a loop |
| typstsyntax/renderer.go | 1081 | nested_append_without_outer_capacity | function layoutSequence appends inside nested loops without visible preallocation |
| typstsyntax/renderer.go | 1081 | slice_append_without_prealloc_known_bound | function layoutSequence appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 642 | slice_append_without_prealloc_known_bound | function layoutVector appends to a slice inside a range loop without visible preallocation |
