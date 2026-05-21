# Deslop Findings Comparison Report

**Generated:** 2026-05-21 16:10:17
**Baseline:** `/home/chinmay/ChinmayPersonalProjects/deslop/verified_gopdfsuit_results.txt`
**Latest:**   `/home/chinmay/ChinmayPersonalProjects/deslop/reports/corpus/gopdfsuit/latest-scan.txt`
**Stripped path prefixes:** ['/home/chinmay/ChinmayPersonalProjects/gopdfsuit/', '/home/chinmay/ChinmayPersonalProjects/gopdfsuit/']

## Summary

| Metric | Count |
| --- | --- |
| Baseline findings | 386 |
| Latest findings | 2643 |
| Net change | +2257 |
| Unchanged (same finding, same line) | 75 |
| Moved (same finding, line shifted) | 0 |
| Removed (finding gone in latest) | 311 |
| Added (new finding in latest) | 2568 |

## Removed Findings (311 total)

### By Category

| Category | Count |
| --- | --- |
| fmt_hot_path | 99 |
| print_debugging_leftover | 38 |
| public_api_missing_type_hints | 30 |
| string_concat_in_loop | 16 |
| weak_crypto | 16 |
| over_abstracted_wrapper | 15 |
| recursive_traversal_risk | 14 |
| allocation_churn_in_loop | 13 |
| test_without_assertion_signal | 11 |
| panic_on_error | 7 |
| redundant_return_none | 6 |
| error_wrapping_misuse | 6 |
| missing_context | 4 |
| builtin_reduction_candidate | 4 |
| broad_exception_handler | 4 |
| magic_value_branching | 3 |
| variadic_public_api | 2 |
| duplicate_validation_pipeline | 2 |
| happy_path_only_test | 2 |
| goroutine_without_coordination | 2 |
| mutex_in_loop | 2 |
| goroutine_without_shutdown_path | 2 |
| duplicate_error_handler_block | 2 |
| god_function | 2 |
| mixed_concerns_function | 2 |
| network_boundary_without_timeout | 2 |
| tight_module_coupling | 1 |
| generic_name | 1 |
| exception_swallowed | 1 |
| temporary_collection_in_loop | 1 |
| name_responsibility_mismatch | 1 |

### By File

| File | Count |
| --- | --- |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 39 |
| bindings/python/pypdfsuit/types.py | 27 |
| internal/pdf/svg/svg.go | 22 |
| internal/pdf/generator.go | 17 |
| internal/pdf/outline.go | 17 |
| internal/pdf/form/xfdf.go | 11 |
| sampledata/python/gopdf/client.py | 11 |
| test/integration_test.go | 11 |
| sampledata/filler/compressed/generate_medical_form.py | 10 |
| internal/pdf/encryption/encrypt.go | 9 |
| sampledata/gopdflib/zerodha/main.go | 9 |
| sampledata/python/main.py | 9 |
| internal/pdf/merge.go | 8 |
| internal/pdf/redact/secure.go | 8 |
| internal/pdf/redact/encryption_inhouse.go | 7 |
| internal/pdf/font/registry.go | 6 |
| sampledata/svg/generate_math_svg.py | 6 |
| bindings/python/pypdfsuit/_bindings.py | 5 |
| internal/pdf/benchmark_test.go | 5 |
| internal/pdf/redact/ocr_adapter.go | 5 |
| internal/pdf/redact/pdf_utils.go | 5 |
| sampledata/benchmarks/fpdf/bench.py | 4 |
| bindings/python/tests/conftest.py | 3 |
| bindings/python/tests/test_integration.py | 3 |
| cmd/gopdfsuit/main.go | 3 |
| internal/pdf/redact/visual.go | 3 |
| internal/pdf/typst_math_test.go | 3 |
| sampledata/financialreport/data/generate_charts.py | 3 |
| sampledata/librarybook/data/generate_codes.py | 3 |
| internal/pdf/font/subset.go | 2 |
| internal/pdf/image.go | 2 |
| internal/pdf/pdf.go | 2 |
| pkg/gopdflib/example_test.go | 2 |
| sampledata/benchmarks/gen_data.go | 2 |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 2 |
| sampledata/gopdflib/zerodha/analyze_bench.py | 2 |
| sampledata/python/amazonReceipt/amazonReceipt.py | 2 |
| sampledata/python/financial_report_pypdfsuit.py | 2 |
| bindings/python/pypdfsuit/__init__.py | 1 |
| bindings/python/pypdfsuit/html.py | 1 |
| bindings/python/pypdfsuit/redact.py | 1 |
| bindings/python/setup.py | 1 |
| internal/benchmarktemplates/runner.go | 1 |
| internal/benchmarktemplates/zerodha_retail.go | 1 |
| internal/handlers/handlers.go | 1 |
| internal/pdf/draw.go | 1 |
| internal/pdf/font/metrics.go | 1 |
| internal/pdf/font/pdfa.go | 1 |
| internal/pdf/helpers.go | 1 |
| internal/pdf/metadata.go | 1 |
| internal/pdf/redact/helpers.go | 1 |
| internal/pdf/redact/redactor.go | 1 |
| internal/pdf/signature/signature.go | 1 |
| internal/pdf/utils.go | 1 |
| sampledata/benchmarks/pypdfsuit/bench.py | 1 |
| sampledata/gopdflib/financial_report/main.go | 1 |
| sampledata/python/JsonFileExample.py | 1 |
| sampledata/samplecode/main.go | 1 |
| typstsyntax/renderer.go | 1 |

### All Removed Findings

| File | Line | Category | Message |
| --- | --- | --- | --- |
| bindings/python/pypdfsuit/__init__.py | 1 | tight_module_coupling | module depends on a large number of repository-local modules |
| bindings/python/pypdfsuit/_bindings.py | 197 | public_api_missing_type_hints | public function call_bytes_array_result omits complete type hints |
| bindings/python/pypdfsuit/_bindings.py | 197 | variadic_public_api | public function call_bytes_array_result relies on *args or **kwargs instead of a clearer interface |
| bindings/python/pypdfsuit/_bindings.py | 164 | public_api_missing_type_hints | public function call_bytes_result omits complete type hints |
| bindings/python/pypdfsuit/_bindings.py | 164 | variadic_public_api | public function call_bytes_result relies on *args or **kwargs instead of a clearer interface |
| bindings/python/pypdfsuit/_bindings.py | 156 | public_api_missing_type_hints | public function get_lib omits complete type hints |
| bindings/python/pypdfsuit/html.py | 13 | duplicate_validation_pipeline | file repeats the same validation pipeline across functions |
| bindings/python/pypdfsuit/redact.py | 8 | duplicate_validation_pipeline | file repeats the same validation pipeline across functions |
| bindings/python/pypdfsuit/types.py | 160 | over_abstracted_wrapper | class Bookmark looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 226 | over_abstracted_wrapper | class Cell looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 175 | over_abstracted_wrapper | class Config looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 148 | over_abstracted_wrapper | class CustomFontConfig looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 412 | over_abstracted_wrapper | class FontInfo looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 211 | over_abstracted_wrapper | class FormField looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 447 | over_abstracted_wrapper | class HtmlToImageRequest looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 425 | over_abstracted_wrapper | class HtmlToPDFRequest looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 197 | over_abstracted_wrapper | class Image looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 109 | over_abstracted_wrapper | class PDFAConfig looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 252 | over_abstracted_wrapper | class Row looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 89 | over_abstracted_wrapper | class SecurityConfig looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 125 | over_abstracted_wrapper | class SignatureConfig looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 322 | over_abstracted_wrapper | class TitleTable looks ceremonial enough that a function or dataclass may suffice |
| bindings/python/pypdfsuit/types.py | 14 | redundant_return_none | function _to_dict returns None explicitly where falling through would be clearer |
| bindings/python/pypdfsuit/types.py | 18 | recursive_traversal_risk | function _to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 20 | recursive_traversal_risk | function _to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 29 | recursive_traversal_risk | function _to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 275 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 393 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 394 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 397 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 399 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 401 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 403 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 405 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 407 | recursive_traversal_risk | function to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/setup.py | 5 | public_api_missing_type_hints | public function has_ext_modules omits complete type hints |
| bindings/python/tests/conftest.py | 26 | public_api_missing_type_hints | public function pytest_sessionstart omits complete type hints |
| bindings/python/tests/conftest.py | 49 | public_api_missing_type_hints | public function simple_html omits complete type hints |
| bindings/python/tests/conftest.py | 55 | public_api_missing_type_hints | public function simple_xfdf omits complete type hints |
| bindings/python/tests/test_integration.py | 49 | redundant_return_none | function _resolve_math_font returns None explicitly where falling through would be clearer |
| bindings/python/tests/test_integration.py | 165 | happy_path_only_test | test test_url_to_pdf appears to cover only success expectations |
| bindings/python/tests/test_integration.py | 185 | happy_path_only_test | test test_url_to_png appears to cover only success expectations |
| cmd/gopdfsuit/main.go | 93 | panic_on_error | function main escalates ordinary error handling through panic or fatal logging |
| cmd/gopdfsuit/main.go | 43 | goroutine_without_coordination | function main launches a goroutine without an obvious coordination signal |
| cmd/gopdfsuit/main.go | 92 | goroutine_without_coordination | function main launches a goroutine without an obvious coordination signal |
| internal/benchmarktemplates/runner.go | 77 | mutex_in_loop | function RunSingleDocumentBenchmark acquires a mutex inside a loop |
| internal/benchmarktemplates/zerodha_retail.go | 47 | string_concat_in_loop | function readChain concatenates strings inside a loop |
| internal/handlers/handlers.go | 456 | fmt_hot_path | function handlerSplitPDF formats strings with fmt inside a loop |
| internal/pdf/benchmark_test.go | 108 | missing_context | function BenchmarkTypst performs context-aware work without accepting context.Context |
| internal/pdf/benchmark_test.go | 62 | fmt_hot_path | function getGoPdfSuitTemplate formats strings with fmt inside a loop |
| internal/pdf/benchmark_test.go | 24 | panic_on_error | function loadBenchmarkData escalates ordinary error handling through panic or fatal logging |
| internal/pdf/benchmark_test.go | 28 | panic_on_error | function loadBenchmarkData escalates ordinary error handling through panic or fatal logging |
| internal/pdf/benchmark_test.go | 34 | panic_on_error | function loadBenchmarkData escalates ordinary error handling through panic or fatal logging |
| internal/pdf/draw.go | 1082 | string_concat_in_loop | function drawTable concatenates strings inside a loop |
| internal/pdf/encryption/encrypt.go | 338 | weak_crypto | function GenerateDocumentID uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 115 | weak_crypto | function computeEncryptionKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 133 | weak_crypto | function computeEncryptionKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 275 | weak_crypto | function computeObjectKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 99 | allocation_churn_in_loop | function computeOwnerHash allocates new objects inside a loop |
| internal/pdf/encryption/encrypt.go | 79 | weak_crypto | function computeOwnerHash uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 83 | weak_crypto | function computeOwnerHash uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 154 | allocation_churn_in_loop | function computeUserHash allocates new objects inside a loop |
| internal/pdf/encryption/encrypt.go | 144 | weak_crypto | function computeUserHash uses weak cryptographic primitive crypto/md5 |
| internal/pdf/font/metrics.go | 632 | string_concat_in_loop | function GetAvailableFonts concatenates strings inside a loop |
| internal/pdf/font/pdfa.go | 238 | missing_context | function downloadFonts performs context-aware work without accepting context.Context |
| internal/pdf/font/registry.go | 281 | fmt_hot_path | function AssignObjectIDs formats strings with fmt inside a loop |
| internal/pdf/font/registry.go | 239 | allocation_churn_in_loop | function CloneForGeneration allocates new objects inside a loop |
| internal/pdf/font/registry.go | 392 | fmt_hot_path | function GeneratePDFFontResources formats strings with fmt inside a loop |
| internal/pdf/font/registry.go | 394 | fmt_hot_path | function GeneratePDFFontResources formats strings with fmt inside a loop |
| internal/pdf/font/registry.go | 148 | allocation_churn_in_loop | function GenerateSubsets allocates new objects inside a loop |
| internal/pdf/font/registry.go | 208 | allocation_churn_in_loop | function ResetUsage allocates new objects inside a loop |
| internal/pdf/font/subset.go | 111 | allocation_churn_in_loop | function buildSubsetFont allocates new objects inside a loop |
| internal/pdf/font/subset.go | 292 | allocation_churn_in_loop | function subsetGlyfAndLoca allocates new objects inside a loop |
| internal/pdf/form/xfdf.go | 546 | fmt_hot_path | function DetectFormFieldsAdvanced formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 878 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 902 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 965 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 971 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1010 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1013 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1033 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1049 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1070 | fmt_hot_path | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 455 | fmt_hot_path | function parseXRefStreams formats strings with fmt inside a loop |
| internal/pdf/generator.go | 846 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 861 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 876 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1111 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1113 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1123 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1173 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1181 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1183 | fmt_hot_path | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1006 | weak_crypto | function GenerateTemplatePDF uses weak cryptographic primitive crypto/md5 |
| internal/pdf/generator.go | 1019 | weak_crypto | function GenerateTemplatePDF uses weak cryptographic primitive crypto/md5 |
| internal/pdf/generator.go | 1377 | fmt_hot_path | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1380 | fmt_hot_path | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1404 | fmt_hot_path | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1414 | fmt_hot_path | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1427 | fmt_hot_path | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1440 | fmt_hot_path | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/helpers.go | 161 | fmt_hot_path | function parseXRefStreams formats strings with fmt inside a loop |
| internal/pdf/image.go | 126 | error_wrapping_misuse | function DecodeImageData uses fmt.Errorf without %w while referencing err |
| internal/pdf/image.go | 150 | error_wrapping_misuse | function DecodeImageData uses fmt.Errorf without %w while referencing err |
| internal/pdf/merge.go | 55 | allocation_churn_in_loop | function MergePDFs allocates new objects inside a loop |
| internal/pdf/merge.go | 69 | allocation_churn_in_loop | function MergePDFs allocates new objects inside a loop |
| internal/pdf/merge.go | 167 | string_concat_in_loop | function MergePDFs concatenates strings inside a loop |
| internal/pdf/merge.go | 169 | string_concat_in_loop | function MergePDFs concatenates strings inside a loop |
| internal/pdf/merge.go | 169 | fmt_hot_path | function MergePDFs formats strings with fmt inside a loop |
| internal/pdf/merge.go | 180 | fmt_hot_path | function MergePDFs formats strings with fmt inside a loop |
| internal/pdf/merge.go | 229 | fmt_hot_path | function MergePDFs formats strings with fmt inside a loop |
| internal/pdf/merge.go | 266 | fmt_hot_path | function replaceRefsOutsideStreams formats strings with fmt inside a loop |
| internal/pdf/metadata.go | 206 | fmt_hot_path | function GenerateXMPMetadata formats strings with fmt inside a loop |
| internal/pdf/outline.go | 478 | fmt_hot_path | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 480 | fmt_hot_path | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 487 | fmt_hot_path | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 491 | fmt_hot_path | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 170 | fmt_hot_path | function allocateOutlineIDs formats strings with fmt inside a loop |
| internal/pdf/outline.go | 414 | fmt_hot_path | function escapeTextUnicode formats strings with fmt inside a loop |
| internal/pdf/outline.go | 420 | fmt_hot_path | function escapeTextUnicode formats strings with fmt inside a loop |
| internal/pdf/outline.go | 362 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 364 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 367 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 372 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 375 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 380 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 383 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 386 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 387 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 388 | fmt_hot_path | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/pdf.go | 104 | error_wrapping_misuse | function ConvertHTMLToImage uses fmt.Errorf without %w while referencing err |
| internal/pdf/pdf.go | 63 | error_wrapping_misuse | function ConvertHTMLToPDF uses fmt.Errorf without %w while referencing err |
| internal/pdf/redact/encryption_inhouse.go | 232 | weak_crypto | function deriveFileKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 243 | weak_crypto | function deriveFileKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 332 | weak_crypto | function deriveObjectKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 283 | weak_crypto | function deriveUserPasswordFromOwner uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 287 | weak_crypto | function deriveUserPasswordFromOwner uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 358 | weak_crypto | function rc4Crypt uses weak cryptographic primitive crypto/rc4 |
| internal/pdf/redact/encryption_inhouse.go | 255 | weak_crypto | function validateUserPassword uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/helpers.go | 149 | fmt_hot_path | function parseXRefStreams formats strings with fmt inside a loop |
| internal/pdf/redact/ocr_adapter.go | 112 | fmt_hot_path | function ExtractWords formats strings with fmt inside a loop |
| internal/pdf/redact/ocr_adapter.go | 114 | missing_context | function ExtractWords performs context-aware work without accepting context.Context |
| internal/pdf/redact/ocr_adapter.go | 129 | missing_context | function ExtractWords performs context-aware work without accepting context.Context |
| internal/pdf/redact/ocr_adapter.go | 116 | error_wrapping_misuse | function ExtractWords uses fmt.Errorf without %w while referencing err |
| internal/pdf/redact/ocr_adapter.go | 132 | error_wrapping_misuse | function ExtractWords uses fmt.Errorf without %w while referencing err |
| internal/pdf/redact/pdf_utils.go | 68 | fmt_hot_path | function buildObjectMap formats strings with fmt inside a loop |
| internal/pdf/redact/pdf_utils.go | 425 | string_concat_in_loop | function extractKidsRefs concatenates strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 215 | string_concat_in_loop | function extractPageContent concatenates strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 845 | fmt_hot_path | function rebuildPDF formats strings with fmt inside a loop |
| internal/pdf/redact/pdf_utils.go | 875 | fmt_hot_path | function rebuildPDF formats strings with fmt inside a loop |
| internal/pdf/redact/redactor.go | 138 | allocation_churn_in_loop | function AnalyzePageCapabilities allocates new objects inside a loop |
| internal/pdf/redact/secure.go | 54 | allocation_churn_in_loop | function applySecureContentRedactions allocates new objects inside a loop |
| internal/pdf/redact/secure.go | 43 | fmt_hot_path | function applySecureContentRedactions formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 50 | fmt_hot_path | function applySecureContentRedactions formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 378 | fmt_hot_path | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 400 | fmt_hot_path | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 451 | fmt_hot_path | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 456 | fmt_hot_path | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 155 | string_concat_in_loop | function extractContentKeys concatenates strings inside a loop |
| internal/pdf/redact/visual.go | 59 | fmt_hot_path | function ApplyRedactions formats strings with fmt inside a loop |
| internal/pdf/redact/visual.go | 65 | fmt_hot_path | function ApplyRedactions formats strings with fmt inside a loop |
| internal/pdf/redact/visual.go | 70 | fmt_hot_path | function ApplyRedactions formats strings with fmt inside a loop |
| internal/pdf/signature/signature.go | 565 | panic_on_error | function mustMarshal escalates ordinary error handling through panic or fatal logging |
| internal/pdf/svg/svg.go | 110 | allocation_churn_in_loop | function ConvertSVGToPDFCommands allocates new objects inside a loop |
| internal/pdf/svg/svg.go | 124 | fmt_hot_path | function ConvertSVGToPDFCommands formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 130 | fmt_hot_path | function ConvertSVGToPDFCommands formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 167 | fmt_hot_path | function ConvertSVGToPDFCommands formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 252 | fmt_hot_path | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 259 | fmt_hot_path | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 262 | fmt_hot_path | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 267 | fmt_hot_path | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 459 | string_concat_in_loop | function parsePathData concatenates strings inside a loop |
| internal/pdf/svg/svg.go | 477 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 486 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 493 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 502 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 508 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 513 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 519 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 524 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 539 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 555 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 579 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 604 | fmt_hot_path | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 454 | generic_name | function parsePathData uses a generic name without strong domain-specific signals |
| internal/pdf/typst_math_test.go | 290 | string_concat_in_loop | function TestTypstMathStress_GenerateEquationBankPDF concatenates strings inside a loop |
| internal/pdf/typst_math_test.go | 421 | string_concat_in_loop | function TestTypstMathStress_GenerateImageStyleShowcasePDF concatenates strings inside a loop |
| internal/pdf/typst_math_test.go | 125 | string_concat_in_loop | function TestTypstMathStress_GenerateTemplatePDFWithIntegrals concatenates strings inside a loop |
| internal/pdf/utils.go | 413 | string_concat_in_loop | function WrapText concatenates strings inside a loop |
| pkg/gopdflib/example_test.go | 131 | panic_on_error | function ExampleGeneratePDF escalates ordinary error handling through panic or fatal logging |
| pkg/gopdflib/example_test.go | 146 | panic_on_error | function ExampleMergePDFs escalates ordinary error handling through panic or fatal logging |
| sampledata/benchmarks/fpdf/bench.py | 36 | exception_swallowed | function run_once swallows a broad exception handler |
| sampledata/benchmarks/fpdf/bench.py | 24 | public_api_missing_type_hints | public function footer omits complete type hints |
| sampledata/benchmarks/fpdf/bench.py | 19 | public_api_missing_type_hints | public function header omits complete type hints |
| sampledata/benchmarks/fpdf/bench.py | 30 | public_api_missing_type_hints | public function run_once omits complete type hints |
| sampledata/benchmarks/gen_data.go | 24 | fmt_hot_path | function main formats strings with fmt inside a loop |
| sampledata/benchmarks/gen_data.go | 25 | fmt_hot_path | function main formats strings with fmt inside a loop |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 66 | fmt_hot_path | function buildRows formats strings with fmt inside a loop |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 153 | mutex_in_loop | function runDataBenchGoPDFLib acquires a mutex inside a loop |
| sampledata/benchmarks/pypdfsuit/bench.py | 21 | public_api_missing_type_hints | public function run_once omits complete type hints |
| sampledata/filler/compressed/generate_medical_form.py | 203 | string_concat_in_loop | function construct_object_stream concatenates strings inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 199 | builtin_reduction_candidate | function construct_object_stream uses a loop shape that may read better with a Python built-in |
| sampledata/filler/compressed/generate_medical_form.py | 92 | string_concat_in_loop | function generate_pdf concatenates strings inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 100 | string_concat_in_loop | function generate_pdf concatenates strings inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 70 | builtin_reduction_candidate | function generate_pdf uses a loop shape that may read better with a Python built-in |
| sampledata/filler/compressed/generate_medical_form.py | 26 | public_api_missing_type_hints | public function compress omits complete type hints |
| sampledata/filler/compressed/generate_medical_form.py | 187 | public_api_missing_type_hints | public function construct_object_stream omits complete type hints |
| sampledata/filler/compressed/generate_medical_form.py | 29 | public_api_missing_type_hints | public function generate_pdf omits complete type hints |
| sampledata/filler/compressed/generate_medical_form.py | 305 | public_api_missing_type_hints | public function generate_xfdf omits complete type hints |
| sampledata/filler/compressed/generate_medical_form.py | 229 | public_api_missing_type_hints | public function write_file omits complete type hints |
| sampledata/financialreport/data/generate_charts.py | 37 | print_debugging_leftover | function generate_bar_chart leaves print-style debugging in Python code |
| sampledata/financialreport/data/generate_charts.py | 63 | print_debugging_leftover | function generate_pie_chart leaves print-style debugging in Python code |
| sampledata/financialreport/data/generate_charts.py | 73 | public_api_missing_type_hints | public function main omits complete type hints |
| sampledata/gopdflib/financial_report/main.go | 87 | goroutine_without_shutdown_path | function main launches a looping goroutine without an obvious shutdown path |
| sampledata/gopdflib/zerodha/analyze_bench.py | 5 | public_api_missing_type_hints | public function parse_runs omits complete type hints |
| sampledata/gopdflib/zerodha/analyze_bench.py | 24 | public_api_missing_type_hints | public function print_stats omits complete type hints |
| sampledata/gopdflib/zerodha/main.go | 320 | fmt_hot_path | function buildActiveTraderTemplate formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 321 | fmt_hot_path | function buildActiveTraderTemplate formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 322 | fmt_hot_path | function buildActiveTraderTemplate formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 489 | fmt_hot_path | function buildHFTTemplate formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 493 | fmt_hot_path | function buildHFTTemplate formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 494 | fmt_hot_path | function buildHFTTemplate formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 495 | fmt_hot_path | function buildHFTTemplate formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 92 | fmt_hot_path | function generateTrades formats strings with fmt inside a loop |
| sampledata/gopdflib/zerodha/main.go | 685 | goroutine_without_shutdown_path | function main launches a looping goroutine without an obvious shutdown path |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 137 | duplicate_error_handler_block | file repeats highly similar exception-handling blocks |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 262 | builtin_reduction_candidate | function build_active_trader_template uses a loop shape that may read better with a Python built-in |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 73 | magic_value_branching | function generate_trades repeats branch-shaping literals instead of naming them explicitly |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 77 | builtin_reduction_candidate | function generate_trades uses a loop shape that may read better with a Python built-in |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 134 | temporary_collection_in_loop | function get_machine_info allocates a temporary collection inside a loop |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | god_function | function get_machine_info concentrates too much control flow and behavior |
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
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | mixed_concerns_function | function run_benchmark mixes multiple infrastructure concerns in one body |
| sampledata/librarybook/data/generate_codes.py | 69 | print_debugging_leftover | function generate_barcode leaves print-style debugging in Python code |
| sampledata/librarybook/data/generate_codes.py | 36 | print_debugging_leftover | function generate_qr_code leaves print-style debugging in Python code |
| sampledata/librarybook/data/generate_codes.py | 83 | public_api_missing_type_hints | public function main omits complete type hints |
| sampledata/python/JsonFileExample.py | 16 | public_api_missing_type_hints | public function main omits complete type hints |
| sampledata/python/amazonReceipt/amazonReceipt.py | 25 | over_abstracted_wrapper | class ReceiptItem looks ceremonial enough that a function or dataclass may suffice |
| sampledata/python/amazonReceipt/amazonReceipt.py | 118 | magic_value_branching | function sample_product_image repeats branch-shaping literals instead of naming them explicitly |
| sampledata/python/financial_report_pypdfsuit.py | 356 | broad_exception_handler | function main catches a broad exception without narrowing the failure type |
| sampledata/python/financial_report_pypdfsuit.py | 342 | public_api_missing_type_hints | public function main omits complete type hints |
| sampledata/python/gopdf/client.py | 27 | duplicate_error_handler_block | file repeats highly similar exception-handling blocks |
| sampledata/python/gopdf/client.py | 57 | broad_exception_handler | function generate_from_file catches a broad exception without narrowing the failure type |
| sampledata/python/gopdf/client.py | 59 | redundant_return_none | function generate_from_file returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 13 | network_boundary_without_timeout | function generate_pdf calls an external HTTP boundary without an obvious timeout or retry policy |
| sampledata/python/gopdf/client.py | 27 | broad_exception_handler | function generate_pdf catches a broad exception without narrowing the failure type |
| sampledata/python/gopdf/client.py | 29 | redundant_return_none | function generate_pdf returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 43 | redundant_return_none | function generate_pdf returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 47 | redundant_return_none | function generate_pdf returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 1 | name_responsibility_mismatch | module client uses a utility-style name but coordinates multiple infrastructure concerns |
| sampledata/python/gopdf/client.py | 49 | public_api_missing_type_hints | public function generate_from_file omits complete type hints |
| sampledata/python/gopdf/client.py | 13 | public_api_missing_type_hints | public function generate_pdf omits complete type hints |
| sampledata/python/main.py | 20 | recursive_traversal_risk | function fill_template uses direct recursion and may need an iterative traversal for deep inputs |
| sampledata/python/main.py | 22 | recursive_traversal_risk | function fill_template uses direct recursion and may need an iterative traversal for deep inputs |
| sampledata/python/main.py | 36 | network_boundary_without_timeout | function main calls an external HTTP boundary without an obvious timeout or retry policy |
| sampledata/python/main.py | 80 | broad_exception_handler | function main catches a broad exception without narrowing the failure type |
| sampledata/python/main.py | 36 | god_function | function main concentrates too much control flow and behavior |
| sampledata/python/main.py | 36 | mixed_concerns_function | function main mixes multiple infrastructure concerns in one body |
| sampledata/python/main.py | 36 | magic_value_branching | function main repeats branch-shaping literals instead of naming them explicitly |
| sampledata/python/main.py | 14 | public_api_missing_type_hints | public function fill_template omits complete type hints |
| sampledata/python/main.py | 36 | public_api_missing_type_hints | public function main omits complete type hints |
| sampledata/samplecode/main.go | 116 | fmt_hot_path | function main formats strings with fmt inside a loop |
| sampledata/svg/generate_math_svg.py | 79 | print_debugging_leftover | function save_math_svg leaves print-style debugging in Python code |
| sampledata/svg/generate_math_svg.py | 80 | print_debugging_leftover | function save_math_svg leaves print-style debugging in Python code |
| sampledata/svg/generate_math_svg.py | 5 | public_api_missing_type_hints | public function generate_complex_calculus omits complete type hints |
| sampledata/svg/generate_math_svg.py | 38 | public_api_missing_type_hints | public function generate_definite_integral_problem omits complete type hints |
| sampledata/svg/generate_math_svg.py | 49 | public_api_missing_type_hints | public function generate_derivative_chain_problem omits complete type hints |
| sampledata/svg/generate_math_svg.py | 59 | public_api_missing_type_hints | public function save_math_svg omits complete type hints |
| test/integration_test.go | 155 | test_without_assertion_signal | test TestFillPDF exercises code without an obvious assertion signal |
| test/integration_test.go | 79 | test_without_assertion_signal | test TestGenerateTemplatePDF exercises code without an obvious assertion signal |
| test/integration_test.go | 450 | test_without_assertion_signal | test TestGenerateTypstMathShowcasePDF exercises code without an obvious assertion signal |
| test/integration_test.go | 479 | test_without_assertion_signal | test TestGenerateTypstSamplePDF exercises code without an obvious assertion signal |
| test/integration_test.go | 253 | test_without_assertion_signal | test TestHtmlToImage exercises code without an obvious assertion signal |
| test/integration_test.go | 213 | test_without_assertion_signal | test TestHtmlToPDF exercises code without an obvious assertion signal |
| test/integration_test.go | 542 | test_without_assertion_signal | test TestIntegrationSuite exercises code without an obvious assertion signal |
| test/integration_test.go | 109 | test_without_assertion_signal | test TestMergePDFs exercises code without an obvious assertion signal |
| test/integration_test.go | 294 | test_without_assertion_signal | test TestSplitPDF exercises code without an obvious assertion signal |
| test/integration_test.go | 398 | test_without_assertion_signal | test TestSplitPDFMaxPerFile exercises code without an obvious assertion signal |
| test/integration_test.go | 346 | test_without_assertion_signal | test TestSplitPDFRange exercises code without an obvious assertion signal |
| typstsyntax/renderer.go | 477 | allocation_churn_in_loop | function layoutMatrixGrid allocates new objects inside a loop |

## Added Findings (2568 total)

### By Category

| Category | Count |
| --- | --- |
| slice_grow_without_cap_hint / performance / stable | 188 |
| slice_append_without_prealloc_known_bound / hot_path / stable | 175 |
| go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | 75 |
| fmt_hot_path / performance / stable | 68 |
| go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | 66 |
| binary_read_for_single_field / performance / stable | 48 |
| go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | 47 |
| go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | 46 |
| go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | 45 |
| go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | 44 |
| test_imports_private_production_module / discipline / stable | 39 |
| go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | 35 |
| error_detail_leaked_to_client / security / stable | 34 |
| regexp_compile_in_hot_path / hot_path / stable | 32 |
| go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | 31 |
| go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | 31 |
| go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | 29 |
| go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | 28 |
| go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | 28 |
| builder_write_string_single_byte_literal / performance / stable | 28 |
| map_growth_without_size_hint / hot_path / stable | 28 |
| sprintf_for_simple_string_format / performance / stable | 27 |
| go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | 26 |
| python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | 26 |
| python_perf_layer_caching_unbounded_dict_cache / performance / stable | 25 |
| go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | 25 |
| error_logged_and_returned / library / stable | 25 |
| go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | 23 |
| copy_of_mapping_created_only_to_read_values / performance / stable | 22 |
| go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | 21 |
| go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | 18 |
| bytes_buffer_without_grow_known_bound / hot_path / stable | 18 |
| go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | 17 |
| go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | 17 |
| go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | 17 |
| string_concat_in_loop / performance / stable | 16 |
| weak_crypto / security / stable | 16 |
| strings_builder_without_grow_known_bound / hot_path / stable | 16 |
| go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | 15 |
| go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | 15 |
| go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | 15 |
| weak_hash_for_integrity / security / stable | 15 |
| errors_new_for_static_sentinel / performance / stable | 15 |
| python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | 14 |
| go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | 13 |
| raw_db_error_exposed_to_client / architecture / stable | 13 |
| handler_switches_on_error_strings / architecture / stable | 13 |
| allocation_churn_in_loop / performance / stable | 13 |
| compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | 12 |
| log_printf_for_production / library / stable | 12 |
| go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | 12 |
| condition_tree_nests_past_two_business_decision_levels / discipline / stable | 11 |
| python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | 11 |
| python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | 11 |
| full_dataset_load / performance / stable | 11 |
| strings_replace_all_for_single_char / performance / stable | 11 |
| go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | 11 |
| same_contextmanager_pattern_copied_across_modules / maintainability / stable | 10 |
| option_bag_model / quality / stable | 10 |
| python_public_api_any_contract / packaging / stable | 10 |
| go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | 10 |
| go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | 10 |
| go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | 10 |
| filter_then_count_then_iterate / hot_path / stable | 10 |
| likely_n_squared_allocation / performance / experimental | 10 |
| byte_string_conversion_in_loop / hot_path / stable | 10 |
| strconv_repeat_on_same_binding / hot_path / stable | 10 |
| go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | 9 |
| go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | 9 |
| blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | 9 |
| python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | 9 |
| public_any_type_leak / quality / stable | 9 |
| gin_handler_mixes_html_json_and_file_responses / architecture / stable | 9 |
| gin_handler_returns_multiple_response_shapes / architecture / stable | 9 |
| three_index_slice_for_append_safety / performance / stable | 9 |
| unnecessary_map_for_set_of_ints / performance / stable | 9 |
| function_local_import_executed_in_frequent_path / hot_path / stable | 8 |
| stable_value_normalization_in_inner_loop / hot_path / stable | 8 |
| go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | 8 |
| builder_or_buffer_recreated_per_iteration / hot_path / stable | 8 |
| python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | 7 |
| public_api_missing_type_hints / maintainability / stable | 7 |
| expensive_work_starts_before_input_validation / discipline / stable | 7 |
| comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | 7 |
| panic_on_error / errors / stable | 7 |
| formfile_open_readall_whole_upload / gin / stable | 7 |
| make_slice_inside_hot_loop_same_shape / hot_path / stable | 7 |
| go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | 7 |
| python_perf_layer_algorithmic_complexity_full_sort_for_top_n / performance / stable | 6 |
| circular_import_hidden_by_function_local_import_on_hot_path / packaging / stable | 6 |
| repeated_string_literal / duplication / stable | 6 |
| function_returns_multiple_unlabeled_shape_variants / discipline / stable | 6 |
| go_perf_layer_lazy_loading_eager_metric_label_cardinality_build / performance / stable | 6 |
| mutable_package_global / idioms / stable | 6 |
| make_map_inside_hot_loop_same_shape / hot_path / stable | 6 |
| go_perf_layer_algorithmic_complexity_sort_before_linear_dedup / performance / stable | 6 |
| go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans / performance / stable | 6 |
| string_builder_write_string_vs_plus / performance / stable | 6 |
| fmt_errorf_without_wrap_verb / performance / stable | 6 |
| error_wrapping_misuse / errors / stable | 6 |
| quadratic_string_building_via_plus_equals / performance / stable | 6 |
| print_debugging_leftover / maintainability / stable | 6 |
| python_perf_layer_error_handling_cost_error_message_formatted_on_success_path / performance / stable | 5 |
| cache_object_exists_without_size_or_eviction_policy_documentation / maintainability / stable | 5 |
| eager_full_file_or_stream_read_when_incremental_iteration_suffices / performance / stable | 5 |
| repeated_file_open_for_same_resource_within_single_operation / performance / stable | 5 |
| full_response_or_export_buffered_before_incremental_consumer_use / performance / stable | 5 |
| redundant_return_none / maintainability / stable | 5 |
| recursive_traversal_risk / performance / stable | 5 |
| feature_flag_lookup_without_config_abstraction / architecture / stable | 5 |
| go_perf_layer_async_concurrency_mutex_held_during_slow_call / performance / stable | 5 |
| go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst / performance / stable | 5 |
| go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item / performance / stable | 5 |
| go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop / performance / stable | 5 |
| go_perf_layer_resource_pooling_rate_limiter_per_request / performance / stable | 5 |
| append_then_trim_each_iteration / hot_path / stable | 5 |
| go_perf_layer_caching_cache_key_built_with_fmt / performance / stable | 5 |
| go_perf_layer_logging_overhead_log_fields_built_before_level_check / performance / stable | 5 |
| go_perf_layer_resource_pooling_db_pool_created_per_repository / performance / stable | 5 |
| nested_append_without_outer_capacity / hot_path / stable | 5 |
| python_perf_layer_profiling_benchmarking_benchmark_missing_warmup / performance / stable | 5 |
| repeated_split_or_join_on_invariant_separator_inside_loop / hot_path / stable | 5 |
| invariant_computation_not_hoisted_out_of_nested_loop / hot_path_ext / stable | 4 |
| repeated_small_writes_without_buffering_or_join / performance / stable | 4 |
| public_api_returns_none_or_value_without_explicit_optional_contract / quality / stable | 4 |
| go_perf_layer_caching_cache_miss_does_duplicate_work / performance / stable | 4 |
| go_perf_layer_logging_overhead_logger_with_fields_per_request / performance / stable | 4 |
| go_perf_layer_memory_allocation_closure_capture_allocates_in_loop / performance / stable | 4 |
| go_perf_layer_network_calls_http_client_created_per_call / performance / stable | 4 |
| go_perf_layer_network_calls_tls_config_built_per_request / performance / stable | 4 |
| go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop / performance / stable | 4 |
| repeated_c_json_inside_stream_loop / gin / stable | 4 |
| go_perf_layer_data_structure_choice_slice_queue_pop_front / performance / stable | 4 |
| map_lookup_double_access / performance / stable | 4 |
| struct_field_exposed_in_json / security / stable | 4 |
| missing_context / context / stable | 4 |
| md5_sum_in_loop / performance / stable | 4 |
| go_perf_layer_resource_pooling_buffer_pool_without_max_capacity / performance / stable | 4 |
| go_perf_layer_data_structure_choice_linked_list_for_cache_iteration / performance / stable | 4 |
| copy_append_idiom_waste / performance / stable | 4 |
| go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set / performance / stable | 4 |
| repeated_string_trim_normalize / performance / stable | 4 |
| metric_name_contains_dynamic_user_or_data_values / observability / stable | 4 |
| cross_cutting_policies_embedded_in_leaf_modules_instead_of_shared_boundary / structure / stable | 4 |
| helper_returns_index_based_tuple_instead_of_named_structure / maintainability / stable | 4 |
| builtin_reduction_candidate / maintainability / stable | 4 |
| binary_or_multipart_response_missing_explicit_content_type / observability / stable | 4 |
| text_bytes_boundary_relies_on_implicit_default_encoding / boundaries / stable | 4 |
| broad_exception_handler / maintainability / stable | 4 |
| python_perf_layer_io_operations_read_entire_file_for_line_processing / performance / stable | 3 |
| python_perf_layer_io_operations_small_file_writes_without_buffer / performance / stable | 3 |
| python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path / performance / stable | 3 |
| python_perf_layer_caching_per_request_settings_cache_rebuilt / performance / stable | 3 |
| python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally / performance / stable | 3 |
| python_perf_layer_serialization_pickle_in_request_hot_path / performance / stable | 3 |
| python_perf_layer_string_handling_regex_compiled_per_call / performance / stable | 3 |
| python_perf_layer_string_handling_json_string_concat_manual_build / performance / stable | 3 |
| generic_class_used_without_type_parameter_application / discipline / stable | 3 |
| test_compares_float_with_equality_operator / discipline / stable | 3 |
| go_perf_layer_caching_per_request_config_cache_rebuild / performance / stable | 3 |
| go_perf_layer_runtime_configuration_reflection_config_loaded_per_call / performance / stable | 3 |
| waitgroup_fanout_without_errgroup_on_error_path / concurrency / stable | 3 |
| go_perf_layer_error_handling_cost_multierror_append_for_success_path / performance / stable | 3 |
| request_dto_outside_transport_package / architecture / stable | 3 |
| writer_write_byte_slice_of_string / performance / stable | 3 |
| public_bool_parameter_api / idioms / stable | 3 |
| rwmutex_without_clear_read_heavy_signal / concurrency / stable | 3 |
| repeated_slice_clone_in_loop / hot_path / stable | 3 |
| sprintf_for_simple_int_to_string / performance / stable | 3 |
| strings_hassuffix_then_trimsuffix / performance / stable | 3 |
| rand_new_per_call / performance / stable | 3 |
| rand_newsource_per_call / performance / stable | 3 |
| magic_value_branching / maintainability / stable | 3 |
| f_string_evaluated_eagerly_inside_logging_call / observability / stable | 3 |
| package_root_reexports_large_dependency_tree_by_default / packaging / stable | 2 |
| variadic_public_api / maintainability / stable | 2 |
| repeated_open_read_close_of_same_small_file_in_single_workflow / hot_path_ext / stable | 2 |
| duplicate_validation_pipeline / duplication / stable | 2 |
| python_perf_layer_network_calls_dns_or_url_parse_repeated_per_request / performance / stable | 2 |
| python_perf_layer_resource_pooling_rate_limiter_created_per_request / performance / stable | 2 |
| python_perf_layer_database_access_count_then_fetch_every_page / performance / stable | 2 |
| python_perf_layer_collection_iteration_multiple_passes_over_same_iterable / performance / stable | 2 |
| overloaded_dispatch_without_typing_overload_decorator / discipline / stable | 2 |
| recursive_walk_over_untrusted_input_lacks_depth_limit / quality / stable | 2 |
| python_perf_layer_garbage_collection_cleanup_lru_cache_on_unbounded_argument_space / performance / stable | 2 |
| goroutine_without_coordination / concurrency / stable | 2 |
| go_perf_layer_error_handling_cost_panic_recover_for_control_flow / performance / stable | 2 |
| mutex_in_loop / concurrency / stable | 2 |
| go_perf_layer_database_access_select_star_on_hot_query / performance / stable | 2 |
| same_domain_error_mapped_to_multiple_statuses / architecture / stable | 2 |
| filepath_join_with_user_path / security / stable | 2 |
| validation_logic_duplicated_across_handlers / architecture / stable | 2 |
| route_setup_scattered_without_router_package / architecture / stable | 2 |
| go_perf_layer_resource_pooling_http_transport_per_service_method / performance / stable | 2 |
| go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated / performance / stable | 2 |
| likely_n_squared_string_concat / performance / experimental | 2 |
| go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse / performance / stable | 2 |
| go_perf_layer_lazy_loading_eager_template_parse_for_unused_routes / performance / stable | 2 |
| go_perf_layer_caching_unbounded_cache_map / performance / stable | 2 |
| range_over_string_by_index / performance / stable | 2 |
| strings_hasprefix_then_trimprefix / performance / stable | 2 |
| repeated_map_clone_in_loop / hot_path / stable | 2 |
| uuid_hash_formatting_only_for_logs / hot_path / stable | 2 |
| hex_encode_to_string_in_loop / performance / stable | 2 |
| helper_or_utils_package_contains_domain_logic / architecture / stable | 2 |
| map_of_slices_prealloc / performance / stable | 2 |
| tuple_return_with_three_or_more_positional_fields_in_public_api / maintainability / stable | 2 |
| string_join_without_generator / hot_path_ext / stable | 2 |
| goroutine_without_shutdown_path / concurrency / stable | 2 |
| exception_handler_returns_default_without_any_logging / discipline / stable | 2 |
| duplicate_error_handler_block / duplication / stable | 2 |
| god_function / structure / stable | 2 |
| mixed_concerns_function / structure / stable | 2 |
| module_constant_rebound_after_public_import / boundaries / stable | 2 |
| batchable_writes_executed_one_at_a_time / performance / stable | 2 |
| repeated_path_exists_check_before_open_or_replace_in_loop / hot_path_ext / stable | 2 |
| same_sequence_scanned_multiple_times_for_related_aggregates / hot_path_ext / stable | 2 |
| logger_error_inside_except_without_exc_info / observability / stable | 2 |
| exception_log_omits_operation_identifier_or_input_summary / observability / stable | 2 |
| type_branch_and_mode_branch_compounded_in_same_function / discipline / stable | 2 |
| python_perf_layer_database_access_orm_query_inside_loop / performance / stable | 1 |
| pathlib_path_reconstructed_from_same_base_in_loop / hot_path / stable | 1 |
| partial_update_mutates_input_before_validation_succeeds / quality / stable | 1 |
| python_perf_layer_data_structure_choice_pandas_object_dtype_for_numeric_hot_columns / performance / stable | 1 |
| file_path_from_user_input_without_normalization_or_anchor_check / boundaries / stable | 1 |
| python_perf_layer_network_calls_http_session_created_per_call / performance / stable | 1 |
| python_perf_layer_garbage_collection_cleanup_file_handle_closed_by_gc / performance / stable | 1 |
| python_perf_layer_lazy_loading_feature_flag_data_loaded_before_needed / performance / stable | 1 |
| python_perf_layer_runtime_configuration_debug_mode_enabled_in_perf_sensitive_path / performance / stable | 1 |
| boolean_flag_parameter_controls_unrelated_behaviors / discipline / stable | 1 |
| temp_file_predictable_name / security / stable | 1 |
| go_perf_layer_logging_overhead_per_record_debug_log_in_batch / performance / stable | 1 |
| go_perf_layer_profiling_benchmarking_benchmark_includes_setup_in_loop / performance / stable | 1 |
| go_perf_layer_runtime_configuration_race_detector_assumed_in_benchmark_numbers / performance / stable | 1 |
| goroutine_for_sync_work / performance / stable | 1 |
| operational_command_handlers_reuse_http_services_without_adapter / architecture / stable | 1 |
| ci_missing_go_test_race / idioms / stable | 1 |
| shared_slice_append_race / security / stable | 1 |
| admin_or_debug_endpoint_registration_mixed_into_public_router_setup / architecture / stable | 1 |
| debug_endpoint_in_production / security / stable | 1 |
| go_perf_layer_profiling_benchmarking_pprof_endpoint_enabled_without_sampling_plan / performance / stable | 1 |
| servefile_via_readfile_then_c_data / gin / stable | 1 |
| file_or_template_read_per_request / gin / stable | 1 |
| gin_route_paths_repeated_as_raw_strings / architecture / stable | 1 |
| gin_route_registration_anonymous_function_overuse / architecture / stable | 1 |
| gin_handler_uses_action_param_switch_for_many_use_cases / architecture / stable | 1 |
| json_marshaled_manually_then_c_data / gin / stable | 1 |
| go_perf_layer_hot_path_optimization_reflection_on_hot_path / performance / stable | 1 |
| no_streaming_for_large_export_handler / gin / stable | 1 |
| shared_integration_test_setup_not_centralized_under_test_support / architecture / stable | 1 |
| route_registration_tests_duplicate_full_bootstrap_per_file / architecture / stable | 1 |
| gin_context_passed_beyond_request_boundary / architecture / stable | 1 |
| auth_or_tenant_extraction_duplicated_across_handlers / architecture / stable | 1 |
| missing_rate_limiting_on_auth_endpoint / security / stable | 1 |
| go_perf_layer_data_structure_choice_interface_map_for_typed_values / performance / stable | 1 |
| middleware_writes_business_response_payloads / architecture / stable | 1 |
| request_context_value_extraction_duplicated_across_handlers / architecture / stable | 1 |
| go_perf_layer_lazy_loading_lazy_once_hides_slow_first_request / performance / stable | 1 |
| http_response_body_not_drained_before_close / idioms / stable | 1 |
| timeoutless_http_default_client_or_helper_call / idioms / stable | 1 |
| go_perf_layer_runtime_configuration_gomaxprocs_set_in_library / performance / stable | 1 |
| strings_tolower_equalfold / performance / stable | 1 |
| go_perf_layer_database_access_count_query_before_paged_fetch / performance / stable | 1 |
| go_perf_layer_database_access_transaction_per_row_bulk_write / performance / stable | 1 |
| go_perf_layer_serialization_base64_roundtrip_for_binary_transport / performance / stable | 1 |
| fmt_sprintf_hex_to_string / performance / stable | 1 |
| strings_splitn_two_index_zero_cut / performance / stable | 1 |
| strconv_formatint_int64_cast_itoa / performance / stable | 1 |
| go_perf_layer_lazy_loading_eager_connect_to_all_backends / performance / stable | 1 |
| timing_attack_on_token_comparison / security / stable | 1 |
| go_perf_layer_io_operations_readall_on_known_large_file / performance / stable | 1 |
| go_perf_layer_io_operations_scanner_used_for_large_token_stream / performance / stable | 1 |
| go_perf_layer_io_operations_temporary_file_for_stream_transform / performance / stable | 1 |
| single_impl_interface / idioms / stable | 1 |
| unnecessary_slice_copy_for_readonly / performance / stable | 1 |
| go_perf_layer_framework_performance_gin_context_copied_for_sync_path / performance / stable | 1 |
| waitgroup_add_inside_loop / performance / stable | 1 |
| http_client_allocated_per_call_without_reuse / idioms / stable | 1 |
| exception_swallowed / maintainability / stable | 1 |
| import_time_file_io / quality / stable | 1 |
| full_collection_sorted_when_partial_order_or_selection_suffices / performance / stable | 1 |
| helper_name_hides_mutation_or_io_side_effect / discipline / stable | 1 |
| no_schema_validation_on_external_data / mlops / stable | 1 |
| parallel_lists_used_instead_of_record_object / maintainability / stable | 1 |
| lookup_table_derived_from_constants_rebuilt_per_invocation / hot_path_ext / stable | 1 |
| same_buffer_or_prefix_reencoded_each_iteration / hot_path / stable | 1 |
| sorted_full_collection_to_extract_top_n_elements / observability / stable | 1 |
| magic_thresholds_duplicated_across_modules / maintainability / stable | 1 |
| orchestrator_performs_low_level_tokenization_or_parsing / architecture / stable | 1 |
| fallback_branch_swallows_invariant_violation_and_returns_plausible_default / quality / stable | 1 |
| rand_newsource_with_time_now_per_call / performance / stable | 1 |
| optional_parameter_used_without_none_guard / discipline / stable | 1 |
| temporary_collection_in_loop / performance / stable | 1 |
| subprocess_or_shell_call_inside_record_processing_loop / hot_path / stable | 1 |
| concurrent_futures_executor_not_shut_down / architecture / stable | 1 |
| thread_pool_or_process_pool_created_and_destroyed_per_call / performance / stable | 1 |
| string_sentinel_values_duplicated_instead_of_constant_or_enum / maintainability / stable | 1 |
| atomic_replace_semantics_implemented_with_non_atomic_file_write / quality / stable | 1 |
| logging_basic_config_called_from_library_package / observability / stable | 1 |
| over_abstracted_wrapper / structure / stable | 1 |
| exception_raised_without_chaining_original_cause / discipline / stable | 1 |
| rate_limit_429_response_missing_retry_after_header_or_stable_body / observability / stable | 1 |
| cli_only_dependency_imported_by_library_entry_module / packaging / stable | 1 |
| public_api_surface_defined_only_by_import_side_effects / packaging / stable | 1 |
| logger_instance_created_inside_function_body / observability / stable | 1 |
| validation_only_happens_after_expensive_side_effect_has_started / quality / stable | 1 |
| network_boundary_without_timeout / maintainability / stable | 1 |
| third_party_exception_type_leaks_across_architecture_boundary / architecture / stable | 1 |
| public_api_forwards_library_specific_exception_shape / boundaries / stable | 1 |
| warning_or_error_logs_emit_unbounded_payload_text / observability / stable | 1 |
| debug_log_serializes_full_large_object_graph / observability / stable | 1 |
| default_timeout_missing_on_external_boundary_wrapper / quality / stable | 1 |
| name_responsibility_mismatch / structure / stable | 1 |
| feature_logic_embedded_in_process_entrypoint / architecture / stable | 1 |
| test_wraps_sut_in_try_except_hiding_exception_detail / discipline / stable | 1 |
| comparison_or_merge_logic_assumes_unique_keys_without_assertion / quality / stable | 1 |
| multiple_regex_passes_over_same_text_without_precompiled_plan / performance / stable | 1 |

### By File

| File | Count |
| --- | --- |
| internal/pdf/draw.go | 238 |
| internal/handlers/handlers.go | 177 |
| internal/pdf/form/xfdf.go | 138 |
| bindings/python/pypdfsuit/types.py | 115 |
| internal/pdf/generator.go | 107 |
| internal/pdf/font/ttf.go | 94 |
| internal/handlers/redact.go | 92 |
| internal/pdf/merge.go | 75 |
| internal/pdf/redact/pdf_utils.go | 75 |
| internal/pdf/redact/secure.go | 59 |
| internal/pdf/svg/svg.go | 57 |
| internal/pdf/font/subset.go | 55 |
| internal/pdf/merge/split.go | 53 |
| internal/pdf/font/metrics.go | 52 |
| typstsyntax/renderer.go | 52 |
| internal/pdf/outline.go | 51 |
| bindings/python/cgo/exports.go | 49 |
| internal/pdf/merge/merger.go | 46 |
| bindings/python/pypdfsuit/html.py | 42 |
| internal/pdf/image.go | 37 |
| bindings/python/pypdfsuit/redact.py | 36 |
| internal/pdf/redact/encryption_inhouse.go | 36 |
| internal/pdf/encryption/encrypt.go | 33 |
| internal/pdf/signature/signature.go | 33 |
| internal/pdf/redact/redactor.go | 32 |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 32 |
| internal/pdf/merge/annotations.go | 31 |
| internal/pdf/redact/search.go | 29 |
| cmd/gopdfsuit/main.go | 27 |
| internal/pdf/font/registry.go | 27 |
| sampledata/python/gopdf/client.py | 26 |
| internal/pdf/redact/ocr_adapter.go | 25 |
| bindings/python/pypdfsuit/split.py | 24 |
| internal/pdf/merge/parser.go | 23 |
| internal/pdf/utils.go | 23 |
| bindings/python/tests/test_integration.py | 21 |
| sampledata/filler/compressed/generate_medical_form.py | 21 |
| bindings/python/pypdfsuit/_bindings.py | 20 |
| sampledata/python/main.py | 20 |
| pkg/fontutils/fontutils.go | 19 |
| internal/pdf/font/pdfa.go | 18 |
| sampledata/python/amazonReceipt/amazonReceipt.py | 18 |
| internal/middleware/auth.go | 17 |
| sampledata/librarybook/data/generate_codes.py | 17 |
| bindings/python/pypdfsuit/generator.py | 16 |
| internal/pdf/pdfa.go | 16 |
| internal/benchmarktemplates/runner.go | 15 |
| bindings/python/pypdfsuit/fill.py | 14 |
| bindings/python/pypdfsuit/merge.py | 14 |
| bindings/python/tests/test_split.py | 13 |
| sampledata/financialreport/data/generate_charts.py | 12 |
| internal/pdf/redact/visual.go | 11 |
| sampledata/gopdflib/zerodha/main.go | 11 |
| internal/pdf/bookmarks.go | 10 |
| internal/pdf/metadata.go | 9 |
| sampledata/benchmarks/fpdf/bench.py | 9 |
| internal/pdf/benchmark_test.go | 8 |
| internal/pdf/helpers.go | 8 |
| internal/pdf/redact/helpers.go | 8 |
| internal/pdf/structure.go | 8 |
| typstsyntax/parser.go | 7 |
| bindings/python/tests/test_html.py | 6 |
| sampledata/gopdflib/zerodha/analyze_bench.py | 6 |
| bindings/python/tests/test_merge.py | 5 |
| internal/models/models.go | 5 |
| internal/pdf/links.go | 5 |
| internal/pdf/pagemanager.go | 5 |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 5 |
| sampledata/python/financial_report_pypdfsuit.py | 5 |
| bindings/python/tests/conftest.py | 4 |
| internal/benchmarktemplates/zerodha_retail.go | 4 |
| internal/pdf/pdf.go | 4 |
| internal/pdf/typst_math_test.go | 4 |
| sampledata/benchmarks/pypdfsuit/bench.py | 4 |
| sampledata/python/JsonFileExample.py | 4 |
| sampledata/svg/generate_math_svg.py | 4 |
| internal/pdf/merge/types.go | 3 |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 3 |
| bindings/python/tests/test_fill.py | 2 |
| internal/handlers/redact_handler_test.go | 2 |
| internal/middleware/cors.go | 2 |
| internal/models/redact.go | 2 |
| internal/pdf/font/compression.go | 2 |
| internal/pdf/types.go | 2 |
| pkg/gopdflib/example_test.go | 2 |
| sampledata/benchmarks/gen_data.go | 2 |
| sampledata/gopdflib/financial_report/main.go | 2 |
| sampledata/python/gopdf/__init__.py | 2 |
| typstsyntax/typst.go | 2 |
| bindings/python/pypdfsuit/__init__.py | 1 |
| bindings/python/setup.py | 1 |
| github/workflows/frontend-build-commit.yml | 1 |
| internal/pdf/signature/helpers.go | 1 |
| sampledata/benchmarks/gopdflib/bench.go | 1 |
| sampledata/benchmarks/gopdfsuit/bench.go | 1 |
| sampledata/gopdflib/load_from_json/main.go | 1 |
| sampledata/gopdflib/text_wrapping/main.go | 1 |
| sampledata/python/test_redact.py | 1 |

### All Added Findings

| File | Line | Category | Message |
| --- | --- | --- | --- |
| bindings/python/cgo/exports.go | 353 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ApplyRedactions matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 353 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function ApplyRedactions matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 353 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function ApplyRedactions matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 382 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ApplyRedactionsAdvanced matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 382 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function ApplyRedactionsAdvanced matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 382 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function ApplyRedactionsAdvanced matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 222 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 225 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| bindings/python/cgo/exports.go | 224 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| bindings/python/cgo/exports.go | 222 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| bindings/python/cgo/exports.go | 222 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 222 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 222 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| bindings/python/cgo/exports.go | 222 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function ConvertHTMLToImage matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| bindings/python/cgo/exports.go | 195 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 198 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| bindings/python/cgo/exports.go | 197 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| bindings/python/cgo/exports.go | 195 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| bindings/python/cgo/exports.go | 195 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 195 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 195 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| bindings/python/cgo/exports.go | 195 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function ConvertHTMLToPDF matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| bindings/python/cgo/exports.go | 300 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ExtractTextPositions matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 304 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function ExtractTextPositions matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 304 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function ExtractTextPositions matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 177 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 325 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 332 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 332 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 37 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function GeneratePDF matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| bindings/python/cgo/exports.go | 37 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GeneratePDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 37 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function GeneratePDF matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 37 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function GeneratePDF matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 252 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GetAvailableFonts matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 250 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function GetAvailableFonts matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 250 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function GetAvailableFonts matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 273 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 277 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 277 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 62 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| bindings/python/cgo/exports.go | 66 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 74 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| bindings/python/cgo/exports.go | 144 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ParsePageSpec matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 152 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function ParsePageSpec matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 152 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function ParsePageSpec matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/cgo/exports.go | 101 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| bindings/python/cgo/exports.go | 100 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| bindings/python/cgo/exports.go | 101 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| bindings/python/cgo/exports.go | 101 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| bindings/python/pypdfsuit/__init__.py | 1 | package_root_reexports_large_dependency_tree_by_default / packaging / stable | package root re-exports a large dependency tree by default |
| bindings/python/pypdfsuit/_bindings.py | 46 | invariant_computation_not_hoisted_out_of_nested_loop / hot_path_ext / stable | function _find_library appears to recompute invariant work inside nested loops |
| bindings/python/pypdfsuit/_bindings.py | 46 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function _find_library contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/_bindings.py | 48 | python_perf_layer_database_access_orm_query_inside_loop / performance / stable | function _find_library matches performance-layer rule python_perf_layer_database_access_orm_query_inside_loop |
| bindings/python/pypdfsuit/_bindings.py | 58 | python_perf_layer_error_handling_cost_error_message_formatted_on_success_path / performance / stable | function _find_library matches performance-layer rule python_perf_layer_error_handling_cost_error_message_formatted_on_success_path |
| bindings/python/pypdfsuit/_bindings.py | 51 | python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | function _find_library matches performance-layer rule python_perf_layer_serialization_json_dumps_for_equality_or_hash |
| bindings/python/pypdfsuit/_bindings.py | 46 | pathlib_path_reconstructed_from_same_base_in_loop / hot_path / stable | function _find_library reconstructs Path objects repeatedly inside a loop |
| bindings/python/pypdfsuit/_bindings.py | 86 | python_perf_layer_algorithmic_complexity_full_sort_for_top_n / performance / stable | function _load_library matches performance-layer rule python_perf_layer_algorithmic_complexity_full_sort_for_top_n |
| bindings/python/pypdfsuit/_bindings.py | 197 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function call_bytes_array_result contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/_bindings.py | 206 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function call_bytes_array_result matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/_bindings.py | 206 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function call_bytes_array_result matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/_bindings.py | 206 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function call_bytes_array_result matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/_bindings.py | 197 | partial_update_mutates_input_before_validation_succeeds / quality / stable | function call_bytes_array_result mutates update payloads before all validation has succeeded |
| bindings/python/pypdfsuit/_bindings.py | 197 | cache_object_exists_without_size_or_eviction_policy_documentation / maintainability / stable | function call_bytes_array_result uses cache-like state without a visible size or eviction policy signal |
| bindings/python/pypdfsuit/_bindings.py | 173 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function call_bytes_result matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/_bindings.py | 164 | cache_object_exists_without_size_or_eviction_policy_documentation / maintainability / stable | function call_bytes_result uses cache-like state without a visible size or eviction policy signal |
| bindings/python/pypdfsuit/_bindings.py | 197 | public_api_missing_type_hints / maintainability / stable | public function call_bytes_array_result omits complete type hints |
| bindings/python/pypdfsuit/_bindings.py | 197 | variadic_public_api / maintainability / stable | public function call_bytes_array_result relies on *args or **kwargs instead of a clearer interface |
| bindings/python/pypdfsuit/_bindings.py | 164 | public_api_missing_type_hints / maintainability / stable | public function call_bytes_result omits complete type hints |
| bindings/python/pypdfsuit/_bindings.py | 164 | variadic_public_api / maintainability / stable | public function call_bytes_result relies on *args or **kwargs instead of a clearer interface |
| bindings/python/pypdfsuit/_bindings.py | 156 | public_api_missing_type_hints / maintainability / stable | public function get_lib omits complete type hints |
| bindings/python/pypdfsuit/fill.py | 8 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function fill_pdf_with_xfdf contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/fill.py | 8 | eager_full_file_or_stream_read_when_incremental_iteration_suffices / performance / stable | function fill_pdf_with_xfdf eagerly reads full file or stream payloads where incremental iteration may suffice |
| bindings/python/pypdfsuit/fill.py | 8 | circular_import_hidden_by_function_local_import_on_hot_path / packaging / stable | function fill_pdf_with_xfdf hides an import inside the body on a live call path |
| bindings/python/pypdfsuit/fill.py | 12 | python_perf_layer_algorithmic_complexity_full_sort_for_top_n / performance / stable | function fill_pdf_with_xfdf matches performance-layer rule python_perf_layer_algorithmic_complexity_full_sort_for_top_n |
| bindings/python/pypdfsuit/fill.py | 23 | python_perf_layer_error_handling_cost_error_message_formatted_on_success_path / performance / stable | function fill_pdf_with_xfdf matches performance-layer rule python_perf_layer_error_handling_cost_error_message_formatted_on_success_path |
| bindings/python/pypdfsuit/fill.py | 28 | python_perf_layer_io_operations_read_entire_file_for_line_processing / performance / stable | function fill_pdf_with_xfdf matches performance-layer rule python_perf_layer_io_operations_read_entire_file_for_line_processing |
| bindings/python/pypdfsuit/fill.py | 28 | python_perf_layer_io_operations_small_file_writes_without_buffer / performance / stable | function fill_pdf_with_xfdf matches performance-layer rule python_perf_layer_io_operations_small_file_writes_without_buffer |
| bindings/python/pypdfsuit/fill.py | 8 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function fill_pdf_with_xfdf performs blocking I/O per item without an obvious batching strategy |
| bindings/python/pypdfsuit/fill.py | 8 | function_local_import_executed_in_frequent_path / hot_path / stable | function fill_pdf_with_xfdf performs local imports on the call path |
| bindings/python/pypdfsuit/fill.py | 8 | repeated_small_writes_without_buffering_or_join / performance / stable | function fill_pdf_with_xfdf performs repeated small writes without visible buffering |
| bindings/python/pypdfsuit/fill.py | 8 | repeated_open_read_close_of_same_small_file_in_single_workflow / hot_path_ext / stable | function fill_pdf_with_xfdf reopens and rereads files repeatedly within one workflow |
| bindings/python/pypdfsuit/fill.py | 8 | repeated_file_open_for_same_resource_within_single_operation / performance / stable | function fill_pdf_with_xfdf reopens files repeatedly within one operation |
| bindings/python/pypdfsuit/fill.py | 8 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function fill_pdf_with_xfdf repeats context-manager patterns that may want one shared helper |
| bindings/python/pypdfsuit/fill.py | 8 | expensive_work_starts_before_input_validation / discipline / stable | function fill_pdf_with_xfdf starts expensive work before validating cheap preconditions |
| bindings/python/pypdfsuit/generator.py | 12 | copy_of_mapping_created_only_to_read_values / performance / stable | function generate_pdf copies mappings only to read them |
| bindings/python/pypdfsuit/generator.py | 12 | circular_import_hidden_by_function_local_import_on_hot_path / packaging / stable | function generate_pdf hides an import inside the body on a live call path |
| bindings/python/pypdfsuit/generator.py | 37 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/generator.py | 37 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/generator.py | 17 | python_perf_layer_data_structure_choice_pandas_object_dtype_for_numeric_hot_columns / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_data_structure_choice_pandas_object_dtype_for_numeric_hot_columns |
| bindings/python/pypdfsuit/generator.py | 23 | python_perf_layer_error_handling_cost_error_message_formatted_on_success_path / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_error_handling_cost_error_message_formatted_on_success_path |
| bindings/python/pypdfsuit/generator.py | 23 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/generator.py | 20 | python_perf_layer_io_operations_small_file_writes_without_buffer / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_io_operations_small_file_writes_without_buffer |
| bindings/python/pypdfsuit/generator.py | 37 | python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_serialization_json_dumps_for_equality_or_hash |
| bindings/python/pypdfsuit/generator.py | 20 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function generate_pdf matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/generator.py | 12 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function generate_pdf performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/generator.py | 12 | function_local_import_executed_in_frequent_path / hot_path / stable | function generate_pdf performs local imports on the call path |
| bindings/python/pypdfsuit/generator.py | 12 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function generate_pdf repeats context-manager patterns that may want one shared helper |
| bindings/python/pypdfsuit/generator.py | 43 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function get_available_fonts matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/generator.py | 43 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function get_available_fonts matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/generator.py | 52 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function get_available_fonts matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/html.py | 13 | duplicate_validation_pipeline / duplication / stable | file repeats the same validation pipeline across functions |
| bindings/python/pypdfsuit/html.py | 54 | copy_of_mapping_created_only_to_read_values / performance / stable | function convert_html_to_image copies mappings only to read them |
| bindings/python/pypdfsuit/html.py | 87 | python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path |
| bindings/python/pypdfsuit/html.py | 69 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/html.py | 62 | python_perf_layer_caching_per_request_settings_cache_rebuilt / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_caching_per_request_settings_cache_rebuilt |
| bindings/python/pypdfsuit/html.py | 87 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/html.py | 87 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/html.py | 68 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/html.py | 80 | python_perf_layer_io_operations_small_file_writes_without_buffer / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_io_operations_small_file_writes_without_buffer |
| bindings/python/pypdfsuit/html.py | 62 | python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally |
| bindings/python/pypdfsuit/html.py | 56 | python_perf_layer_network_calls_dns_or_url_parse_repeated_per_request / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_network_calls_dns_or_url_parse_repeated_per_request |
| bindings/python/pypdfsuit/html.py | 62 | python_perf_layer_resource_pooling_rate_limiter_created_per_request / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_resource_pooling_rate_limiter_created_per_request |
| bindings/python/pypdfsuit/html.py | 87 | python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_serialization_json_dumps_for_equality_or_hash |
| bindings/python/pypdfsuit/html.py | 62 | python_perf_layer_serialization_pickle_in_request_hot_path / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_serialization_pickle_in_request_hot_path |
| bindings/python/pypdfsuit/html.py | 65 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/html.py | 89 | python_perf_layer_string_handling_regex_compiled_per_call / performance / stable | function convert_html_to_image matches performance-layer rule python_perf_layer_string_handling_regex_compiled_per_call |
| bindings/python/pypdfsuit/html.py | 54 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function convert_html_to_image performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/html.py | 54 | function_local_import_executed_in_frequent_path / hot_path / stable | function convert_html_to_image performs local imports on the call path |
| bindings/python/pypdfsuit/html.py | 54 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function convert_html_to_image repeats context-manager patterns that may want one shared helper |
| bindings/python/pypdfsuit/html.py | 54 | expensive_work_starts_before_input_validation / discipline / stable | function convert_html_to_image starts expensive work before validating cheap preconditions |
| bindings/python/pypdfsuit/html.py | 79 | file_path_from_user_input_without_normalization_or_anchor_check / boundaries / stable | function convert_html_to_image uses user-controlled file path without normpath/resolve check; path traversal risk |
| bindings/python/pypdfsuit/html.py | 13 | copy_of_mapping_created_only_to_read_values / performance / stable | function convert_html_to_pdf copies mappings only to read them |
| bindings/python/pypdfsuit/html.py | 13 | circular_import_hidden_by_function_local_import_on_hot_path / packaging / stable | function convert_html_to_pdf hides an import inside the body on a live call path |
| bindings/python/pypdfsuit/html.py | 15 | python_perf_layer_algorithmic_complexity_full_sort_for_top_n / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_algorithmic_complexity_full_sort_for_top_n |
| bindings/python/pypdfsuit/html.py | 49 | python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path |
| bindings/python/pypdfsuit/html.py | 27 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/html.py | 20 | python_perf_layer_caching_per_request_settings_cache_rebuilt / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_caching_per_request_settings_cache_rebuilt |
| bindings/python/pypdfsuit/html.py | 49 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/html.py | 49 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/html.py | 34 | python_perf_layer_database_access_count_then_fetch_every_page / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_database_access_count_then_fetch_every_page |
| bindings/python/pypdfsuit/html.py | 26 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/html.py | 20 | python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally |
| bindings/python/pypdfsuit/html.py | 15 | python_perf_layer_network_calls_dns_or_url_parse_repeated_per_request / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_network_calls_dns_or_url_parse_repeated_per_request |
| bindings/python/pypdfsuit/html.py | 51 | python_perf_layer_network_calls_http_session_created_per_call / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_network_calls_http_session_created_per_call |
| bindings/python/pypdfsuit/html.py | 20 | python_perf_layer_resource_pooling_rate_limiter_created_per_request / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_resource_pooling_rate_limiter_created_per_request |
| bindings/python/pypdfsuit/html.py | 49 | python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_serialization_json_dumps_for_equality_or_hash |
| bindings/python/pypdfsuit/html.py | 20 | python_perf_layer_serialization_pickle_in_request_hot_path / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_serialization_pickle_in_request_hot_path |
| bindings/python/pypdfsuit/html.py | 23 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/html.py | 49 | python_perf_layer_string_handling_json_string_concat_manual_build / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_string_handling_json_string_concat_manual_build |
| bindings/python/pypdfsuit/html.py | 51 | python_perf_layer_string_handling_regex_compiled_per_call / performance / stable | function convert_html_to_pdf matches performance-layer rule python_perf_layer_string_handling_regex_compiled_per_call |
| bindings/python/pypdfsuit/html.py | 13 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function convert_html_to_pdf performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/html.py | 13 | function_local_import_executed_in_frequent_path / hot_path / stable | function convert_html_to_pdf performs local imports on the call path |
| bindings/python/pypdfsuit/merge.py | 12 | eager_full_file_or_stream_read_when_incremental_iteration_suffices / performance / stable | function merge_pdfs eagerly reads full file or stream payloads where incremental iteration may suffice |
| bindings/python/pypdfsuit/merge.py | 12 | circular_import_hidden_by_function_local_import_on_hot_path / packaging / stable | function merge_pdfs hides an import inside the body on a live call path |
| bindings/python/pypdfsuit/merge.py | 14 | python_perf_layer_algorithmic_complexity_full_sort_for_top_n / performance / stable | function merge_pdfs matches performance-layer rule python_perf_layer_algorithmic_complexity_full_sort_for_top_n |
| bindings/python/pypdfsuit/merge.py | 14 | python_perf_layer_collection_iteration_multiple_passes_over_same_iterable / performance / stable | function merge_pdfs matches performance-layer rule python_perf_layer_collection_iteration_multiple_passes_over_same_iterable |
| bindings/python/pypdfsuit/merge.py | 17 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function merge_pdfs matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/merge.py | 23 | python_perf_layer_error_handling_cost_error_message_formatted_on_success_path / performance / stable | function merge_pdfs matches performance-layer rule python_perf_layer_error_handling_cost_error_message_formatted_on_success_path |
| bindings/python/pypdfsuit/merge.py | 14 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function merge_pdfs matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/merge.py | 17 | python_perf_layer_io_operations_read_entire_file_for_line_processing / performance / stable | function merge_pdfs matches performance-layer rule python_perf_layer_io_operations_read_entire_file_for_line_processing |
| bindings/python/pypdfsuit/merge.py | 12 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function merge_pdfs performs blocking I/O per item without an obvious batching strategy |
| bindings/python/pypdfsuit/merge.py | 12 | function_local_import_executed_in_frequent_path / hot_path / stable | function merge_pdfs performs local imports on the call path |
| bindings/python/pypdfsuit/merge.py | 12 | repeated_open_read_close_of_same_small_file_in_single_workflow / hot_path_ext / stable | function merge_pdfs reopens and rereads files repeatedly within one workflow |
| bindings/python/pypdfsuit/merge.py | 12 | repeated_file_open_for_same_resource_within_single_operation / performance / stable | function merge_pdfs reopens files repeatedly within one operation |
| bindings/python/pypdfsuit/merge.py | 12 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function merge_pdfs repeats context-manager patterns that may want one shared helper |
| bindings/python/pypdfsuit/merge.py | 12 | expensive_work_starts_before_input_validation / discipline / stable | function merge_pdfs starts expensive work before validating cheap preconditions |
| bindings/python/pypdfsuit/redact.py | 8 | repeated_string_literal / duplication / stable | file repeats the same long string literal instead of centralizing it |
| bindings/python/pypdfsuit/redact.py | 8 | duplicate_validation_pipeline / duplication / stable | file repeats the same validation pipeline across functions |
| bindings/python/pypdfsuit/redact.py | 60 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function apply_redactions matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/redact.py | 60 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function apply_redactions matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/redact.py | 71 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function apply_redactions matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/redact.py | 60 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function apply_redactions matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/redact.py | 60 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function apply_redactions matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/redact.py | 71 | python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | function apply_redactions matches performance-layer rule python_perf_layer_serialization_json_dumps_for_equality_or_hash |
| bindings/python/pypdfsuit/redact.py | 59 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function apply_redactions matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/redact.py | 54 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function apply_redactions performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/redact.py | 111 | python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_caching_cache_key_uses_json_dumps_in_hot_path |
| bindings/python/pypdfsuit/redact.py | 111 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/redact.py | 107 | python_perf_layer_caching_per_request_settings_cache_rebuilt / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_caching_per_request_settings_cache_rebuilt |
| bindings/python/pypdfsuit/redact.py | 111 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/redact.py | 123 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/redact.py | 112 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/redact.py | 112 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/redact.py | 107 | python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_logging_overhead_trace_payload_serialized_unconditionally |
| bindings/python/pypdfsuit/redact.py | 123 | python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_serialization_json_dumps_for_equality_or_hash |
| bindings/python/pypdfsuit/redact.py | 107 | python_perf_layer_serialization_pickle_in_request_hot_path / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_serialization_pickle_in_request_hot_path |
| bindings/python/pypdfsuit/redact.py | 110 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/redact.py | 115 | python_perf_layer_string_handling_json_string_concat_manual_build / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_string_handling_json_string_concat_manual_build |
| bindings/python/pypdfsuit/redact.py | 126 | python_perf_layer_string_handling_regex_compiled_per_call / performance / stable | function apply_redactions_advanced matches performance-layer rule python_perf_layer_string_handling_regex_compiled_per_call |
| bindings/python/pypdfsuit/redact.py | 105 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function apply_redactions_advanced performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/redact.py | 105 | generic_class_used_without_type_parameter_application / discipline / stable | function apply_redactions_advanced uses bare generic container annotation without type parameters |
| bindings/python/pypdfsuit/redact.py | 39 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function extract_text_positions matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/redact.py | 39 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function extract_text_positions matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/redact.py | 39 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function extract_text_positions matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/redact.py | 90 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function find_text_occurrences matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/redact.py | 90 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function find_text_occurrences matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/redact.py | 90 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function find_text_occurrences matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/redact.py | 86 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function find_text_occurrences matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/redact.py | 81 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function find_text_occurrences performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/redact.py | 16 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function get_page_info matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/redact.py | 16 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function get_page_info matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/redact.py | 16 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function get_page_info matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/split.py | 54 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function parse_page_spec matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/split.py | 54 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function parse_page_spec matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/split.py | 73 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function parse_page_spec matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/split.py | 54 | python_perf_layer_string_handling_json_string_concat_manual_build / performance / stable | function parse_page_spec matches performance-layer rule python_perf_layer_string_handling_json_string_concat_manual_build |
| bindings/python/pypdfsuit/split.py | 52 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function parse_page_spec performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/split.py | 52 | function_local_import_executed_in_frequent_path / hot_path / stable | function parse_page_spec performs local imports on the call path |
| bindings/python/pypdfsuit/split.py | 12 | full_response_or_export_buffered_before_incremental_consumer_use / performance / stable | function split_pdf buffers full output before handing it to a downstream consumer |
| bindings/python/pypdfsuit/split.py | 12 | copy_of_mapping_created_only_to_read_values / performance / stable | function split_pdf copies mappings only to read them |
| bindings/python/pypdfsuit/split.py | 12 | eager_full_file_or_stream_read_when_incremental_iteration_suffices / performance / stable | function split_pdf eagerly reads full file or stream payloads where incremental iteration may suffice |
| bindings/python/pypdfsuit/split.py | 12 | circular_import_hidden_by_function_local_import_on_hot_path / packaging / stable | function split_pdf hides an import inside the body on a live call path |
| bindings/python/pypdfsuit/split.py | 14 | python_perf_layer_algorithmic_complexity_full_sort_for_top_n / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_algorithmic_complexity_full_sort_for_top_n |
| bindings/python/pypdfsuit/split.py | 25 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/split.py | 42 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/split.py | 14 | python_perf_layer_collection_iteration_multiple_passes_over_same_iterable / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_collection_iteration_multiple_passes_over_same_iterable |
| bindings/python/pypdfsuit/split.py | 42 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/split.py | 21 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/split.py | 24 | python_perf_layer_error_handling_cost_error_message_formatted_on_success_path / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_error_handling_cost_error_message_formatted_on_success_path |
| bindings/python/pypdfsuit/split.py | 14 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/split.py | 17 | python_perf_layer_io_operations_read_entire_file_for_line_processing / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_io_operations_read_entire_file_for_line_processing |
| bindings/python/pypdfsuit/split.py | 42 | python_perf_layer_serialization_json_dumps_for_equality_or_hash / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_serialization_json_dumps_for_equality_or_hash |
| bindings/python/pypdfsuit/split.py | 17 | python_perf_layer_string_handling_bytes_decode_encode_roundtrip / performance / stable | function split_pdf matches performance-layer rule python_perf_layer_string_handling_bytes_decode_encode_roundtrip |
| bindings/python/pypdfsuit/split.py | 12 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function split_pdf performs expensive transforms before cheap reject checks |
| bindings/python/pypdfsuit/split.py | 12 | function_local_import_executed_in_frequent_path / hot_path / stable | function split_pdf performs local imports on the call path |
| bindings/python/pypdfsuit/split.py | 12 | expensive_work_starts_before_input_validation / discipline / stable | function split_pdf starts expensive work before validating cheap preconditions |
| bindings/python/pypdfsuit/types.py | 34 | repeated_string_literal / duplication / stable | file repeats the same long string literal instead of centralizing it |
| bindings/python/pypdfsuit/types.py | 37 | python_perf_layer_algorithmic_complexity_full_sort_for_top_n / performance / stable | function _python_to_json_key matches performance-layer rule python_perf_layer_algorithmic_complexity_full_sort_for_top_n |
| bindings/python/pypdfsuit/types.py | 85 | python_perf_layer_database_access_count_then_fetch_every_page / performance / stable | function _python_to_json_key matches performance-layer rule python_perf_layer_database_access_count_then_fetch_every_page |
| bindings/python/pypdfsuit/types.py | 34 | python_perf_layer_garbage_collection_cleanup_file_handle_closed_by_gc / performance / stable | function _python_to_json_key matches performance-layer rule python_perf_layer_garbage_collection_cleanup_file_handle_closed_by_gc |
| bindings/python/pypdfsuit/types.py | 34 | python_perf_layer_lazy_loading_feature_flag_data_loaded_before_needed / performance / stable | function _python_to_json_key matches performance-layer rule python_perf_layer_lazy_loading_feature_flag_data_loaded_before_needed |
| bindings/python/pypdfsuit/types.py | 34 | python_perf_layer_runtime_configuration_debug_mode_enabled_in_perf_sensitive_path / performance / stable | function _python_to_json_key matches performance-layer rule python_perf_layer_runtime_configuration_debug_mode_enabled_in_perf_sensitive_path |
| bindings/python/pypdfsuit/types.py | 11 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function _to_dict contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/types.py | 11 | copy_of_mapping_created_only_to_read_values / performance / stable | function _to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 14 | overloaded_dispatch_without_typing_overload_decorator / discipline / stable | function _to_dict dispatches on isinstance for multiple types without @typing.overload signatures |
| bindings/python/pypdfsuit/types.py | 18 | python_perf_layer_caching_cached_value_immediately_deserialized / performance / stable | function _to_dict matches performance-layer rule python_perf_layer_caching_cached_value_immediately_deserialized |
| bindings/python/pypdfsuit/types.py | 18 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function _to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 18 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function _to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 17 | python_perf_layer_data_structure_choice_list_as_fifo_queue / performance / stable | function _to_dict matches performance-layer rule python_perf_layer_data_structure_choice_list_as_fifo_queue |
| bindings/python/pypdfsuit/types.py | 17 | python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure / performance / stable | function _to_dict matches performance-layer rule python_perf_layer_error_handling_cost_multi_error_list_allocated_before_failure |
| bindings/python/pypdfsuit/types.py | 11 | recursive_walk_over_untrusted_input_lacks_depth_limit / quality / stable | function _to_dict recurses without an obvious depth limit |
| bindings/python/pypdfsuit/types.py | 14 | redundant_return_none / maintainability / stable | function _to_dict returns None explicitly where falling through would be clearer |
| bindings/python/pypdfsuit/types.py | 11 | public_api_returns_none_or_value_without_explicit_optional_contract / quality / stable | function _to_dict returns None on some paths without an explicit optional contract |
| bindings/python/pypdfsuit/types.py | 11 | function_returns_multiple_unlabeled_shape_variants / discipline / stable | function _to_dict returns multiple unlabeled shapes from the same function |
| bindings/python/pypdfsuit/types.py | 11 | boolean_flag_parameter_controls_unrelated_behaviors / discipline / stable | function _to_dict uses a boolean parameter to select materially different behaviors |
| bindings/python/pypdfsuit/types.py | 18 | recursive_traversal_risk / performance / stable | function _to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 20 | recursive_traversal_risk / performance / stable | function _to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 29 | recursive_traversal_risk / performance / stable | function _to_dict uses direct recursion and may need an iterative traversal for deep inputs |
| bindings/python/pypdfsuit/types.py | 272 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function to_dict contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/types.py | 308 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function to_dict contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/types.py | 350 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function to_dict contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/types.py | 391 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function to_dict contains a deeply nested decision tree that would benefit from decomposition |
| bindings/python/pypdfsuit/types.py | 104 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 120 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 143 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 155 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 170 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 192 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 206 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 221 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 257 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 272 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 308 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 329 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 350 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 391 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 420 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 442 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 464 | copy_of_mapping_created_only_to_read_values / performance / stable | function to_dict copies mappings only to read them |
| bindings/python/pypdfsuit/types.py | 272 | comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | function to_dict has opaque branching that suggests the code structure itself could be clearer |
| bindings/python/pypdfsuit/types.py | 308 | comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | function to_dict has opaque branching that suggests the code structure itself could be clearer |
| bindings/python/pypdfsuit/types.py | 350 | comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | function to_dict has opaque branching that suggests the code structure itself could be clearer |
| bindings/python/pypdfsuit/types.py | 391 | comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | function to_dict has opaque branching that suggests the code structure itself could be clearer |
| bindings/python/pypdfsuit/types.py | 105 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 121 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 144 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 156 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 171 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 193 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 207 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 222 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 244 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 258 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 275 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 313 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 332 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 353 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 393 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 421 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 443 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 465 | python_perf_layer_caching_unbounded_dict_cache / performance / stable | function to_dict matches performance-layer rule python_perf_layer_caching_unbounded_dict_cache |
| bindings/python/pypdfsuit/types.py | 105 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 121 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 144 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 156 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 171 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 193 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 207 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 222 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 244 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 258 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 275 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 313 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 332 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 353 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 393 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 421 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 443 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 465 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 477 | python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records / performance / stable | function to_dict matches performance-layer rule python_perf_layer_data_structure_choice_dict_of_dicts_for_fixed_records |
| bindings/python/pypdfsuit/types.py | 313 | python_perf_layer_garbage_collection_cleanup_lru_cache_on_unbounded_argument_space / performance / stable | function to_dict matches performance-layer rule python_perf_layer_garbage_collection_cleanup_lru_cache_on_unbounded_argument_space |
| bindings/python/pypdfsuit/types.py | 393 | python_perf_layer_garbage_collection_cleanup_lru_cache_on_unbounded_argument_space / performance / stable | function to_dict matches performance-layer rule python_perf_layer_garbage_collection_cleanup_lru_cache_on_unbounded_argument_space |
| bindings/python/pypdfsuit/types.py | 226 | option_bag_model / quality / stable | model Cell encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 175 | option_bag_model / quality / stable | model Config encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 447 | option_bag_model / quality / stable | model HtmlToImageRequest encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 425 | option_bag_model / quality / stable | model HtmlToPDFRequest encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 109 | option_bag_model / quality / stable | model PDFAConfig encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 379 | option_bag_model / quality / stable | model PDFTemplate encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 89 | option_bag_model / quality / stable | model SecurityConfig encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 125 | option_bag_model / quality / stable | model SignatureConfig encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 262 | option_bag_model / quality / stable | model Table encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 340 | option_bag_model / quality / stable | model Title encodes many optional or boolean switches and looks like an option bag |
| bindings/python/pypdfsuit/types.py | 243 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 257 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 272 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 294 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 308 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 329 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 350 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 371 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 391 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 476 | python_public_api_any_contract / packaging / stable | public API to_dict uses Any in its type contract |
| bindings/python/pypdfsuit/types.py | 243 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 272 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 294 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 308 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 329 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 350 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 371 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 391 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/pypdfsuit/types.py | 476 | public_any_type_leak / quality / stable | public function to_dict exposes a very wide type contract |
| bindings/python/setup.py | 5 | public_api_missing_type_hints / maintainability / stable | public function has_ext_modules omits complete type hints |
| bindings/python/tests/conftest.py | 26 | expensive_work_starts_before_input_validation / discipline / stable | function pytest_sessionstart starts expensive work before validating cheap preconditions |
| bindings/python/tests/conftest.py | 26 | public_api_missing_type_hints / maintainability / stable | public function pytest_sessionstart omits complete type hints |
| bindings/python/tests/conftest.py | 49 | public_api_missing_type_hints / maintainability / stable | public function simple_html omits complete type hints |
| bindings/python/tests/conftest.py | 55 | public_api_missing_type_hints / maintainability / stable | public function simple_xfdf omits complete type hints |
| bindings/python/tests/test_fill.py | 13 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_fill.py | 25 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_html.py | 28 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_html.py | 35 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_html.py | 49 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_html.py | 69 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_html.py | 76 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_html.py | 92 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_integration.py | 52 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function _generate_pdf_from_dict performs expensive transforms before cheap reject checks |
| bindings/python/tests/test_integration.py | 52 | generic_class_used_without_type_parameter_application / discipline / stable | function _generate_pdf_from_dict uses bare generic container annotation without type parameters |
| bindings/python/tests/test_integration.py | 59 | circular_import_hidden_by_function_local_import_on_hot_path / packaging / stable | function _has_chrome hides an import inside the body on a live call path |
| bindings/python/tests/test_integration.py | 59 | function_local_import_executed_in_frequent_path / hot_path / stable | function _has_chrome performs local imports on the call path |
| bindings/python/tests/test_integration.py | 45 | public_api_returns_none_or_value_without_explicit_optional_contract / quality / stable | function _resolve_math_font returns None on some paths without an explicit optional contract |
| bindings/python/tests/test_integration.py | 254 | test_compares_float_with_equality_operator / discipline / stable | function test_split_max_per_file compares float with == operator; use pytest.approx() or math.isclose() |
| bindings/python/tests/test_integration.py | 232 | test_compares_float_with_equality_operator / discipline / stable | function test_split_page_range compares float with == operator; use pytest.approx() or math.isclose() |
| bindings/python/tests/test_integration.py | 214 | test_compares_float_with_equality_operator / discipline / stable | function test_split_single_page compares float with == operator; use pytest.approx() or math.isclose() |
| bindings/python/tests/test_integration.py | 45 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 52 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 59 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 82 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 109 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 137 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 165 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 185 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 207 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 225 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 243 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 267 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_integration.py | 289 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.get_lib; test through public API |
| bindings/python/tests/test_merge.py | 16 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_merge.py | 32 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_merge.py | 43 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_merge.py | 52 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_merge.py | 61 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 19 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 32 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 41 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 46 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 51 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 56 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 61 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 66 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 71 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 80 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 91 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 103 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| bindings/python/tests/test_split.py | 114 | test_imports_private_production_module / discipline / stable | test file imports private production module pypdfsuit._bindings.GoPDFSuitError; test through public API |
| cmd/gopdfsuit/main.go | 22 | feature_flag_lookup_without_config_abstraction / architecture / stable | feature flag or config lookup happens inline |
| cmd/gopdfsuit/main.go | 23 | temp_file_predictable_name / security / stable | function main creates temp file with predictable name |
| cmd/gopdfsuit/main.go | 93 | panic_on_error / errors / stable | function main escalates ordinary error handling through panic or fatal logging |
| cmd/gopdfsuit/main.go | 43 | goroutine_without_coordination / concurrency / stable | function main launches a goroutine without an obvious coordination signal |
| cmd/gopdfsuit/main.go | 92 | goroutine_without_coordination / concurrency / stable | function main launches a goroutine without an obvious coordination signal |
| cmd/gopdfsuit/main.go | 100 | go_perf_layer_async_concurrency_mutex_held_during_slow_call / performance / stable | function main matches performance-layer rule go_perf_layer_async_concurrency_mutex_held_during_slow_call |
| cmd/gopdfsuit/main.go | 20 | go_perf_layer_caching_cache_miss_does_duplicate_work / performance / stable | function main matches performance-layer rule go_perf_layer_caching_cache_miss_does_duplicate_work |
| cmd/gopdfsuit/main.go | 22 | go_perf_layer_caching_per_request_config_cache_rebuild / performance / stable | function main matches performance-layer rule go_perf_layer_caching_per_request_config_cache_rebuild |
| cmd/gopdfsuit/main.go | 74 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function main matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| cmd/gopdfsuit/main.go | 56 | go_perf_layer_error_handling_cost_panic_recover_for_control_flow / performance / stable | function main matches performance-layer rule go_perf_layer_error_handling_cost_panic_recover_for_control_flow |
| cmd/gopdfsuit/main.go | 50 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function main matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| cmd/gopdfsuit/main.go | 27 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function main matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| cmd/gopdfsuit/main.go | 83 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function main matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| cmd/gopdfsuit/main.go | 22 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function main matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| cmd/gopdfsuit/main.go | 25 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function main matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| cmd/gopdfsuit/main.go | 25 | go_perf_layer_logging_overhead_logger_with_fields_per_request / performance / stable | function main matches performance-layer rule go_perf_layer_logging_overhead_logger_with_fields_per_request |
| cmd/gopdfsuit/main.go | 25 | go_perf_layer_logging_overhead_per_record_debug_log_in_batch / performance / stable | function main matches performance-layer rule go_perf_layer_logging_overhead_per_record_debug_log_in_batch |
| cmd/gopdfsuit/main.go | 50 | go_perf_layer_memory_allocation_closure_capture_allocates_in_loop / performance / stable | function main matches performance-layer rule go_perf_layer_memory_allocation_closure_capture_allocates_in_loop |
| cmd/gopdfsuit/main.go | 58 | go_perf_layer_network_calls_http_client_created_per_call / performance / stable | function main matches performance-layer rule go_perf_layer_network_calls_http_client_created_per_call |
| cmd/gopdfsuit/main.go | 22 | go_perf_layer_network_calls_tls_config_built_per_request / performance / stable | function main matches performance-layer rule go_perf_layer_network_calls_tls_config_built_per_request |
| cmd/gopdfsuit/main.go | 20 | go_perf_layer_profiling_benchmarking_benchmark_includes_setup_in_loop / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_includes_setup_in_loop |
| cmd/gopdfsuit/main.go | 20 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| cmd/gopdfsuit/main.go | 20 | go_perf_layer_runtime_configuration_race_detector_assumed_in_benchmark_numbers / performance / stable | function main matches performance-layer rule go_perf_layer_runtime_configuration_race_detector_assumed_in_benchmark_numbers |
| cmd/gopdfsuit/main.go | 22 | go_perf_layer_runtime_configuration_reflection_config_loaded_per_call / performance / stable | function main matches performance-layer rule go_perf_layer_runtime_configuration_reflection_config_loaded_per_call |
| cmd/gopdfsuit/main.go | 20 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function main matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| cmd/gopdfsuit/main.go | 92 | goroutine_for_sync_work / performance / stable | function main spawns goroutine for immediately-awaited work |
| cmd/gopdfsuit/main.go | 20 | operational_command_handlers_reuse_http_services_without_adapter / architecture / stable | operational command path appears to depend on HTTP-oriented service contracts |
| github/workflows/frontend-build-commit.yml | 1 | ci_missing_go_test_race / idioms / stable | repo CI or build automation does not visibly run `go test -race` |
| internal/benchmarktemplates/runner.go | 116 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function Fail matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| internal/benchmarktemplates/runner.go | 77 | mutex_in_loop / concurrency / stable | function RunSingleDocumentBenchmark acquires a mutex inside a loop |
| internal/benchmarktemplates/runner.go | 78 | shared_slice_append_race / security / stable | function RunSingleDocumentBenchmark appends to a shared slice while using goroutines |
| internal/benchmarktemplates/runner.go | 62 | waitgroup_fanout_without_errgroup_on_error_path / concurrency / stable | function RunSingleDocumentBenchmark fans out work with WaitGroup while errors still need coordinated cancellation |
| internal/benchmarktemplates/runner.go | 60 | go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst |
| internal/benchmarktemplates/runner.go | 60 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/benchmarktemplates/runner.go | 67 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/benchmarktemplates/runner.go | 78 | go_perf_layer_error_handling_cost_multierror_append_for_success_path / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_error_handling_cost_multierror_append_for_success_path |
| internal/benchmarktemplates/runner.go | 60 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/benchmarktemplates/runner.go | 60 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/benchmarktemplates/runner.go | 71 | go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop |
| internal/benchmarktemplates/runner.go | 66 | go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item |
| internal/benchmarktemplates/runner.go | 50 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function RunSingleDocumentBenchmark matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| internal/benchmarktemplates/runner.go | 28 | go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst / performance / stable | function monitorMemory matches performance-layer rule go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst |
| internal/benchmarktemplates/runner.go | 28 | go_perf_layer_database_access_select_star_on_hot_query / performance / stable | function monitorMemory matches performance-layer rule go_perf_layer_database_access_select_star_on_hot_query |
| internal/benchmarktemplates/zerodha_retail.go | 234 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function BenchmarkHeader matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| internal/benchmarktemplates/zerodha_retail.go | 47 | string_concat_in_loop / performance / stable | function readChain concatenates strings inside a loop |
| internal/benchmarktemplates/zerodha_retail.go | 43 | stable_value_normalization_in_inner_loop / hot_path / stable | function readChain normalizes a stable value inside a loop |
| internal/benchmarktemplates/zerodha_retail.go | 27 | full_dataset_load / performance / stable | function readText loads an entire payload into memory |
| internal/handlers/handlers.go | 162 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 178 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 271 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 298 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 356 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 401 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 478 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 536 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/handlers.go | 162 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 178 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 271 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 298 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 356 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 401 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 478 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 536 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/handlers.go | 127 | admin_or_debug_endpoint_registration_mixed_into_public_router_setup / architecture / stable | admin or debug routes are registered in the same public router setup |
| internal/handlers/handlers.go | 162 | same_domain_error_mapped_to_multiple_statuses / architecture / stable | domain error Error is translated to multiple statuses |
| internal/handlers/handlers.go | 34 | feature_flag_lookup_without_config_abstraction / architecture / stable | feature flag or config lookup happens inline |
| internal/handlers/handlers.go | 127 | debug_endpoint_in_production / security / stable | function RegisterRoutes exposes debug/pprof endpoint |
| internal/handlers/handlers.go | 87 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/handlers/handlers.go | 138 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/handlers/handlers.go | 138 | go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop |
| internal/handlers/handlers.go | 87 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/handlers/handlers.go | 102 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/handlers/handlers.go | 109 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/handlers/handlers.go | 112 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/handlers/handlers.go | 87 | go_perf_layer_network_calls_tls_config_built_per_request / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_network_calls_tls_config_built_per_request |
| internal/handlers/handlers.go | 127 | go_perf_layer_profiling_benchmarking_pprof_endpoint_enabled_without_sampling_plan / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_profiling_benchmarking_pprof_endpoint_enabled_without_sampling_plan |
| internal/handlers/handlers.go | 106 | go_perf_layer_resource_pooling_rate_limiter_per_request / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_resource_pooling_rate_limiter_per_request |
| internal/handlers/handlers.go | 87 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/handlers/handlers.go | 132 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 87 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function RegisterRoutes matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 345 | error_detail_leaked_to_client / security / stable | function handleFillPDF leaks error details to client |
| internal/handlers/handlers.go | 300 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function handleFillPDF matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/handlers.go | 306 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function handleFillPDF matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/handlers/handlers.go | 303 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function handleFillPDF matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/handlers/handlers.go | 300 | go_perf_layer_logging_overhead_logger_with_fields_per_request / performance / stable | function handleFillPDF matches performance-layer rule go_perf_layer_logging_overhead_logger_with_fields_per_request |
| internal/handlers/handlers.go | 339 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleFillPDF matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 308 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function handleFillPDF matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/handlers.go | 300 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handleFillPDF matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 276 | error_detail_leaked_to_client / security / stable | function handleGenerateTemplatePDF leaks error details to client |
| internal/handlers/handlers.go | 281 | error_detail_leaked_to_client / security / stable | function handleGenerateTemplatePDF leaks error details to client |
| internal/handlers/handlers.go | 287 | error_detail_leaked_to_client / security / stable | function handleGenerateTemplatePDF leaks error details to client |
| internal/handlers/handlers.go | 276 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function handleGenerateTemplatePDF matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| internal/handlers/handlers.go | 276 | go_perf_layer_resource_pooling_rate_limiter_per_request / performance / stable | function handleGenerateTemplatePDF matches performance-layer rule go_perf_layer_resource_pooling_rate_limiter_per_request |
| internal/handlers/handlers.go | 276 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleGenerateTemplatePDF matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 276 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function handleGenerateTemplatePDF matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/handlers.go | 276 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function handleGenerateTemplatePDF matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/handlers/handlers.go | 276 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handleGenerateTemplatePDF matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 221 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleGetFonts matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 195 | filepath_join_with_user_path / security / stable | function handleGetTemplateData joins filepath without traversal check |
| internal/handlers/handlers.go | 207 | error_detail_leaked_to_client / security / stable | function handleGetTemplateData leaks error details to client |
| internal/handlers/handlers.go | 198 | full_dataset_load / performance / stable | function handleGetTemplateData loads an entire payload into memory |
| internal/handlers/handlers.go | 181 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function handleGetTemplateData matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| internal/handlers/handlers.go | 181 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleGetTemplateData matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 180 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function handleGetTemplateData matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/handlers.go | 181 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function handleGetTemplateData matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/handlers/handlers.go | 181 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handleGetTemplateData matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 213 | servefile_via_readfile_then_c_data / gin / stable | function handleGetTemplateData reads a file into memory and then writes it through gin.Context.Data |
| internal/handlers/handlers.go | 198 | file_or_template_read_per_request / gin / stable | function handleGetTemplateData reads files directly on a request path |
| internal/handlers/handlers.go | 542 | error_detail_leaked_to_client / security / stable | function handleHTMLToImage leaks error details to client |
| internal/handlers/handlers.go | 548 | error_detail_leaked_to_client / security / stable | function handleHTMLToImage leaks error details to client |
| internal/handlers/handlers.go | 570 | error_detail_leaked_to_client / security / stable | function handleHTMLToImage leaks error details to client |
| internal/handlers/handlers.go | 542 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/handlers/handlers.go | 542 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/handlers/handlers.go | 536 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/handlers/handlers.go | 537 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| internal/handlers/handlers.go | 537 | go_perf_layer_logging_overhead_logger_with_fields_per_request / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_logging_overhead_logger_with_fields_per_request |
| internal/handlers/handlers.go | 542 | go_perf_layer_network_calls_http_client_created_per_call / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_network_calls_http_client_created_per_call |
| internal/handlers/handlers.go | 542 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 542 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/handlers.go | 542 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/handlers/handlers.go | 539 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handleHTMLToImage matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 537 | log_printf_for_production / library / stable | function handleHTMLToImage uses stdlib log in handler |
| internal/handlers/handlers.go | 547 | log_printf_for_production / library / stable | function handleHTMLToImage uses stdlib log in handler |
| internal/handlers/handlers.go | 552 | log_printf_for_production / library / stable | function handleHTMLToImage uses stdlib log in handler |
| internal/handlers/handlers.go | 565 | log_printf_for_production / library / stable | function handleHTMLToImage uses stdlib log in handler |
| internal/handlers/handlers.go | 569 | log_printf_for_production / library / stable | function handleHTMLToImage uses stdlib log in handler |
| internal/handlers/handlers.go | 574 | log_printf_for_production / library / stable | function handleHTMLToImage uses stdlib log in handler |
| internal/handlers/handlers.go | 484 | error_detail_leaked_to_client / security / stable | function handleHTMLToPDF leaks error details to client |
| internal/handlers/handlers.go | 490 | error_detail_leaked_to_client / security / stable | function handleHTMLToPDF leaks error details to client |
| internal/handlers/handlers.go | 524 | error_detail_leaked_to_client / security / stable | function handleHTMLToPDF leaks error details to client |
| internal/handlers/handlers.go | 484 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/handlers/handlers.go | 478 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/handlers/handlers.go | 479 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| internal/handlers/handlers.go | 479 | go_perf_layer_logging_overhead_logger_with_fields_per_request / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_logging_overhead_logger_with_fields_per_request |
| internal/handlers/handlers.go | 484 | go_perf_layer_network_calls_http_client_created_per_call / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_network_calls_http_client_created_per_call |
| internal/handlers/handlers.go | 484 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 484 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/handlers.go | 484 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/handlers/handlers.go | 481 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handleHTMLToPDF matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 479 | log_printf_for_production / library / stable | function handleHTMLToPDF uses stdlib log in handler |
| internal/handlers/handlers.go | 489 | log_printf_for_production / library / stable | function handleHTMLToPDF uses stdlib log in handler |
| internal/handlers/handlers.go | 494 | log_printf_for_production / library / stable | function handleHTMLToPDF uses stdlib log in handler |
| internal/handlers/handlers.go | 519 | log_printf_for_production / library / stable | function handleHTMLToPDF uses stdlib log in handler |
| internal/handlers/handlers.go | 523 | log_printf_for_production / library / stable | function handleHTMLToPDF uses stdlib log in handler |
| internal/handlers/handlers.go | 528 | log_printf_for_production / library / stable | function handleHTMLToPDF uses stdlib log in handler |
| internal/handlers/handlers.go | 385 | slice_append_without_prealloc_known_bound / hot_path / stable | function handleMergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/handlers/handlers.go | 385 | slice_grow_without_cap_hint / performance / stable | function handleMergePDFs appends to pdfBytesList in a loop without a capacity hint |
| internal/handlers/handlers.go | 360 | error_detail_leaked_to_client / security / stable | function handleMergePDFs leaks error details to client |
| internal/handlers/handlers.go | 376 | error_detail_leaked_to_client / security / stable | function handleMergePDFs leaks error details to client |
| internal/handlers/handlers.go | 382 | error_detail_leaked_to_client / security / stable | function handleMergePDFs leaks error details to client |
| internal/handlers/handlers.go | 390 | error_detail_leaked_to_client / security / stable | function handleMergePDFs leaks error details to client |
| internal/handlers/handlers.go | 379 | full_dataset_load / performance / stable | function handleMergePDFs loads an entire payload into memory |
| internal/handlers/handlers.go | 360 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/handlers.go | 365 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/handlers.go | 356 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/handlers/handlers.go | 360 | go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop |
| internal/handlers/handlers.go | 370 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/handlers/handlers.go | 356 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/handlers/handlers.go | 385 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/handlers/handlers.go | 360 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 360 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/handlers.go | 360 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handleMergePDFs matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 376 | repeated_c_json_inside_stream_loop / gin / stable | function handleMergePDFs writes Gin JSON responses from inside a loop |
| internal/handlers/handlers.go | 382 | repeated_c_json_inside_stream_loop / gin / stable | function handleMergePDFs writes Gin JSON responses from inside a loop |
| internal/handlers/handlers.go | 164 | filepath_join_with_user_path / security / stable | function handleSPA joins filepath without traversal check |
| internal/handlers/handlers.go | 162 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function handleSPA matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/handlers/handlers.go | 168 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleSPA matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 244 | error_detail_leaked_to_client / security / stable | function handleUploadFont leaks error details to client |
| internal/handlers/handlers.go | 253 | error_detail_leaked_to_client / security / stable | function handleUploadFont leaks error details to client |
| internal/handlers/handlers.go | 261 | error_detail_leaked_to_client / security / stable | function handleUploadFont leaks error details to client |
| internal/handlers/handlers.go | 251 | full_dataset_load / performance / stable | function handleUploadFont loads an entire payload into memory |
| internal/handlers/handlers.go | 230 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function handleUploadFont matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/handlers.go | 235 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function handleUploadFont matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/handlers.go | 235 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function handleUploadFont matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/handlers/handlers.go | 230 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handleUploadFont matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 230 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handleUploadFont matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 235 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function handleUploadFont matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/handlers/handlers.go | 251 | formfile_open_readall_whole_upload / gin / stable | function handleUploadFont reads an uploaded form file fully into memory |
| internal/handlers/handlers.go | 456 | fmt_hot_path / performance / stable | function handlerSplitPDF formats strings with fmt inside a loop |
| internal/handlers/handlers.go | 405 | error_detail_leaked_to_client / security / stable | function handlerSplitPDF leaks error details to client |
| internal/handlers/handlers.go | 413 | error_detail_leaked_to_client / security / stable | function handlerSplitPDF leaks error details to client |
| internal/handlers/handlers.go | 429 | error_detail_leaked_to_client / security / stable | function handlerSplitPDF leaks error details to client |
| internal/handlers/handlers.go | 440 | error_detail_leaked_to_client / security / stable | function handlerSplitPDF leaks error details to client |
| internal/handlers/handlers.go | 460 | error_detail_leaked_to_client / security / stable | function handlerSplitPDF leaks error details to client |
| internal/handlers/handlers.go | 465 | error_detail_leaked_to_client / security / stable | function handlerSplitPDF leaks error details to client |
| internal/handlers/handlers.go | 411 | full_dataset_load / performance / stable | function handlerSplitPDF loads an entire payload into memory |
| internal/handlers/handlers.go | 403 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/handlers.go | 411 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/handlers.go | 405 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/handlers/handlers.go | 454 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/handlers/handlers.go | 411 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/handlers/handlers.go | 411 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/handlers/handlers.go | 427 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/handlers/handlers.go | 401 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/handlers/handlers.go | 411 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/handlers/handlers.go | 403 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/handlers/handlers.go | 405 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/handlers.go | 405 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/handlers.go | 403 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function handlerSplitPDF matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/handlers.go | 411 | formfile_open_readall_whole_upload / gin / stable | function handlerSplitPDF reads an uploaded form file fully into memory |
| internal/handlers/handlers.go | 453 | bytes_buffer_without_grow_known_bound / hot_path / stable | function handlerSplitPDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/handlers/handlers.go | 460 | repeated_c_json_inside_stream_loop / gin / stable | function handlerSplitPDF writes Gin JSON responses from inside a loop |
| internal/handlers/handlers.go | 465 | repeated_c_json_inside_stream_loop / gin / stable | function handlerSplitPDF writes Gin JSON responses from inside a loop |
| internal/handlers/handlers.go | 207 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 244 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 276 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 345 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 360 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 405 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 484 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 542 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/handlers.go | 207 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 244 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 276 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 345 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 360 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 405 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 484 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 542 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/handlers.go | 180 | validation_logic_duplicated_across_handlers / architecture / stable | manual validation checks are duplicated across handlers |
| internal/handlers/handlers.go | 111 | gin_route_paths_repeated_as_raw_strings / architecture / stable | route path /fonts is repeated as a raw string |
| internal/handlers/handlers.go | 87 | route_setup_scattered_without_router_package / architecture / stable | route registration is spread into a non-router transport file |
| internal/handlers/handlers.go | 98 | route_setup_scattered_without_router_package / architecture / stable | route registration lives outside a router-oriented package |
| internal/handlers/handlers.go | 102 | gin_route_registration_anonymous_function_overuse / architecture / stable | route registration uses several inline anonymous Gin handlers |
| internal/handlers/redact.go | 235 | gin_handler_uses_action_param_switch_for_many_use_cases / architecture / stable | Gin handler branches on action or mode parameters for multiple workflows |
| internal/handlers/redact.go | 185 | gin_handler_mixes_html_json_and_file_responses / architecture / stable | Gin handler mixes HTML, JSON, or file-style responses |
| internal/handlers/redact.go | 185 | gin_handler_returns_multiple_response_shapes / architecture / stable | Gin handler mixes several response rendering styles |
| internal/handlers/redact.go | 57 | same_domain_error_mapped_to_multiple_statuses / architecture / stable | domain error Error is translated to multiple statuses |
| internal/handlers/redact.go | 277 | error_detail_leaked_to_client / security / stable | function HandleRedactApply leaks error details to client |
| internal/handlers/redact.go | 282 | error_detail_leaked_to_client / security / stable | function HandleRedactApply leaks error details to client |
| internal/handlers/redact.go | 265 | full_dataset_load / performance / stable | function HandleRedactApply loads an entire payload into memory |
| internal/handlers/redact.go | 290 | json_marshaled_manually_then_c_data / gin / stable | function HandleRedactApply marshals JSON manually before writing through gin.Context.Data |
| internal/handlers/redact.go | 188 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/redact.go | 187 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/handlers/redact.go | 233 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/redact.go | 198 | go_perf_layer_data_structure_choice_slice_queue_pop_front / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_data_structure_choice_slice_queue_pop_front |
| internal/handlers/redact.go | 193 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/handlers/redact.go | 250 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/handlers/redact.go | 198 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/handlers/redact.go | 198 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/handlers/redact.go | 263 | go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop |
| internal/handlers/redact.go | 246 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/handlers/redact.go | 185 | go_perf_layer_hot_path_optimization_reflection_on_hot_path / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_hot_path_optimization_reflection_on_hot_path |
| internal/handlers/redact.go | 224 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/handlers/redact.go | 188 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| internal/handlers/redact.go | 250 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/handlers/redact.go | 188 | go_perf_layer_network_calls_tls_config_built_per_request / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_network_calls_tls_config_built_per_request |
| internal/handlers/redact.go | 188 | go_perf_layer_resource_pooling_rate_limiter_per_request / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_resource_pooling_rate_limiter_per_request |
| internal/handlers/redact.go | 188 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/redact.go | 188 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/redact.go | 188 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/handlers/redact.go | 188 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function HandleRedactApply matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/redact.go | 290 | no_streaming_for_large_export_handler / gin / stable | function HandleRedactApply materializes a collection into memory before writing the response |
| internal/handlers/redact.go | 265 | formfile_open_readall_whole_upload / gin / stable | function HandleRedactApply reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 122 | error_detail_leaked_to_client / security / stable | function HandleRedactCapabilities leaks error details to client |
| internal/handlers/redact.go | 127 | error_detail_leaked_to_client / security / stable | function HandleRedactCapabilities leaks error details to client |
| internal/handlers/redact.go | 110 | full_dataset_load / performance / stable | function HandleRedactCapabilities loads an entire payload into memory |
| internal/handlers/redact.go | 99 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function HandleRedactCapabilities matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/redact.go | 110 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function HandleRedactCapabilities matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/redact.go | 99 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function HandleRedactCapabilities matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/redact.go | 99 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function HandleRedactCapabilities matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/redact.go | 99 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function HandleRedactCapabilities matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/redact.go | 110 | formfile_open_readall_whole_upload / gin / stable | function HandleRedactCapabilities reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 83 | error_detail_leaked_to_client / security / stable | function HandleRedactPageInfo leaks error details to client |
| internal/handlers/redact.go | 88 | error_detail_leaked_to_client / security / stable | function HandleRedactPageInfo leaks error details to client |
| internal/handlers/redact.go | 71 | full_dataset_load / performance / stable | function HandleRedactPageInfo loads an entire payload into memory |
| internal/handlers/redact.go | 60 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function HandleRedactPageInfo matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/redact.go | 71 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function HandleRedactPageInfo matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/redact.go | 60 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function HandleRedactPageInfo matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/redact.go | 60 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function HandleRedactPageInfo matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/redact.go | 60 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function HandleRedactPageInfo matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/redact.go | 71 | formfile_open_readall_whole_upload / gin / stable | function HandleRedactPageInfo reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 342 | error_detail_leaked_to_client / security / stable | function HandleRedactSearch leaks error details to client |
| internal/handlers/redact.go | 347 | error_detail_leaked_to_client / security / stable | function HandleRedactSearch leaks error details to client |
| internal/handlers/redact.go | 330 | full_dataset_load / performance / stable | function HandleRedactSearch loads an entire payload into memory |
| internal/handlers/redact.go | 297 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/redact.go | 309 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/redact.go | 301 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/handlers/redact.go | 301 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/handlers/redact.go | 304 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/handlers/redact.go | 312 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/handlers/redact.go | 297 | go_perf_layer_logging_overhead_log_payload_serialized_before_sampling / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_logging_overhead_log_payload_serialized_before_sampling |
| internal/handlers/redact.go | 297 | go_perf_layer_resource_pooling_rate_limiter_per_request / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_resource_pooling_rate_limiter_per_request |
| internal/handlers/redact.go | 297 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/redact.go | 297 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/redact.go | 297 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/handlers/redact.go | 297 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function HandleRedactSearch matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/redact.go | 330 | formfile_open_readall_whole_upload / gin / stable | function HandleRedactSearch reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 172 | error_detail_leaked_to_client / security / stable | function HandleRedactTextPositions leaks error details to client |
| internal/handlers/redact.go | 177 | error_detail_leaked_to_client / security / stable | function HandleRedactTextPositions leaks error details to client |
| internal/handlers/redact.go | 160 | full_dataset_load / performance / stable | function HandleRedactTextPositions loads an entire payload into memory |
| internal/handlers/redact.go | 138 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function HandleRedactTextPositions matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/handlers/redact.go | 160 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function HandleRedactTextPositions matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/handlers/redact.go | 138 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function HandleRedactTextPositions matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/handlers/redact.go | 138 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function HandleRedactTextPositions matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/handlers/redact.go | 138 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function HandleRedactTextPositions matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/handlers/redact.go | 160 | formfile_open_readall_whole_upload / gin / stable | function HandleRedactTextPositions reads an uploaded form file fully into memory |
| internal/handlers/redact.go | 41 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function normalizeTextSearchQueries matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/handlers/redact.go | 38 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function normalizeTextSearchQueries matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/handlers/redact.go | 45 | stable_value_normalization_in_inner_loop / hot_path / stable | function normalizeTextSearchQueries normalizes a stable value inside a loop |
| internal/handlers/redact.go | 46 | map_lookup_double_access / performance / stable | function normalizeTextSearchQueries performs double map lookup for same key |
| internal/handlers/redact.go | 20 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function parseCommaSeparatedTerms matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/handlers/redact.go | 16 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function parseCommaSeparatedTerms matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/handlers/redact.go | 23 | stable_value_normalization_in_inner_loop / hot_path / stable | function parseCommaSeparatedTerms normalizes a stable value inside a loop |
| internal/handlers/redact.go | 28 | map_lookup_double_access / performance / stable | function parseCommaSeparatedTerms performs double map lookup for same key |
| internal/handlers/redact.go | 83 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/redact.go | 122 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/redact.go | 172 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/redact.go | 277 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/redact.go | 342 | raw_db_error_exposed_to_client / architecture / stable | handler appears to serialize raw error text to clients |
| internal/handlers/redact.go | 83 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/redact.go | 122 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/redact.go | 172 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/redact.go | 277 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/redact.go | 342 | handler_switches_on_error_strings / architecture / stable | handler branches on err.Error() text |
| internal/handlers/redact.go | 76 | validation_logic_duplicated_across_handlers / architecture / stable | manual validation checks are duplicated across handlers |
| internal/handlers/redact_handler_test.go | 16 | shared_integration_test_setup_not_centralized_under_test_support / architecture / stable | integration-like test setup is repeated in the same file |
| internal/handlers/redact_handler_test.go | 16 | route_registration_tests_duplicate_full_bootstrap_per_file / architecture / stable | multiple transport tests rebuild full route bootstrap inline |
| internal/middleware/auth.go | 174 | gin_context_passed_beyond_request_boundary / architecture / stable | Gin handler passes *gin.Context into deeper layers or goroutines |
| internal/middleware/auth.go | 143 | auth_or_tenant_extraction_duplicated_across_handlers / architecture / stable | auth or tenant extraction is duplicated across handlers |
| internal/middleware/auth.go | 172 | missing_rate_limiting_on_auth_endpoint / security / stable | auth-style handler LogAuthInfo has no visible rate limiting guard |
| internal/middleware/auth.go | 63 | feature_flag_lookup_without_config_abstraction / architecture / stable | feature flag or config lookup happens inline |
| internal/middleware/auth.go | 119 | feature_flag_lookup_without_config_abstraction / architecture / stable | feature flag or config lookup happens inline |
| internal/middleware/auth.go | 153 | go_perf_layer_data_structure_choice_interface_map_for_typed_values / performance / stable | function GetUserInfo matches performance-layer rule go_perf_layer_data_structure_choice_interface_map_for_typed_values |
| internal/middleware/auth.go | 50 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/middleware/auth.go | 50 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/middleware/auth.go | 63 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/middleware/auth.go | 34 | go_perf_layer_network_calls_tls_config_built_per_request / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_network_calls_tls_config_built_per_request |
| internal/middleware/auth.go | 34 | go_perf_layer_resource_pooling_http_transport_per_service_method / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_resource_pooling_http_transport_per_service_method |
| internal/middleware/auth.go | 42 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/middleware/auth.go | 34 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/middleware/auth.go | 34 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function GoogleAuthMiddleware matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/middleware/auth.go | 119 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function OptionalAuthMiddleware matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/middleware/auth.go | 42 | middleware_writes_business_response_payloads / architecture / stable | middleware writes response payloads directly |
| internal/middleware/auth.go | 143 | request_context_value_extraction_duplicated_across_handlers / architecture / stable | request context value extraction is duplicated across handlers |
| internal/middleware/cors.go | 17 | go_perf_layer_resource_pooling_http_transport_per_service_method / performance / stable | function CORSMiddleware matches performance-layer rule go_perf_layer_resource_pooling_http_transport_per_service_method |
| internal/middleware/cors.go | 17 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function CORSMiddleware matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/models/models.go | 259 | request_dto_outside_transport_package / architecture / stable | request DTO HTMLToImageRequest lives outside a transport-oriented package |
| internal/models/models.go | 242 | request_dto_outside_transport_package / architecture / stable | request DTO HTMLToPDFRequest lives outside a transport-oriented package |
| internal/models/models.go | 62 | struct_field_exposed_in_json / security / stable | struct SecurityConfig exposes sensitive field OwnerPassword to JSON serialization |
| internal/models/models.go | 61 | struct_field_exposed_in_json / security / stable | struct SecurityConfig exposes sensitive field UserPassword to JSON serialization |
| internal/models/models.go | 89 | struct_field_exposed_in_json / security / stable | struct SignatureConfig exposes sensitive field PrivateKeyPEM to JSON serialization |
| internal/models/redact.go | 35 | request_dto_outside_transport_package / architecture / stable | request DTO RedactionTextQuery lives outside a transport-oriented package |
| internal/models/redact.go | 44 | struct_field_exposed_in_json / security / stable | struct ApplyRedactionOptions exposes sensitive field Password to JSON serialization |
| internal/pdf/benchmark_test.go | 80 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function BenchmarkGoPdfSuit matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| internal/pdf/benchmark_test.go | 98 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function BenchmarkTypst matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| internal/pdf/benchmark_test.go | 108 | missing_context / context / stable | function BenchmarkTypst performs context-aware work without accepting context.Context |
| internal/pdf/benchmark_test.go | 24 | panic_on_error / errors / stable | function loadBenchmarkData escalates ordinary error handling through panic or fatal logging |
| internal/pdf/benchmark_test.go | 28 | panic_on_error / errors / stable | function loadBenchmarkData escalates ordinary error handling through panic or fatal logging |
| internal/pdf/benchmark_test.go | 34 | panic_on_error / errors / stable | function loadBenchmarkData escalates ordinary error handling through panic or fatal logging |
| internal/pdf/benchmark_test.go | 33 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function loadBenchmarkData matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| internal/pdf/benchmark_test.go | 33 | go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated / performance / stable | function loadBenchmarkData matches performance-layer rule go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated |
| internal/pdf/bookmarks.go | 14 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function GenerateBookmarks matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/bookmarks.go | 23 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function GenerateBookmarks matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/bookmarks.go | 29 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function GenerateBookmarks matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/bookmarks.go | 23 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function GenerateBookmarks matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/bookmarks.go | 50 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateBookmarks matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/bookmarks.go | 70 | slice_grow_without_cap_hint / performance / stable | function generateBookmarkItems appends to itemIDs in a loop without a capacity hint |
| internal/pdf/bookmarks.go | 57 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function generateBookmarkItems matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/bookmarks.go | 57 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function generateBookmarkItems matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/bookmarks.go | 57 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function generateBookmarkItems matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/bookmarks.go | 64 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function generateBookmarkItems matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/draw.go | 1381 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function drawFooter matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/draw.go | 1469 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function drawImage matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/draw.go | 1499 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function drawImage matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/draw.go | 1608 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function drawImageWithXObjectInternal matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/draw.go | 1579 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function drawImageWithXObjectInternal matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/draw.go | 132 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function drawPageBorder matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/draw.go | 1422 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function drawPageNumber matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/draw.go | 1435 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function drawPageNumber matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/draw.go | 1082 | likely_n_squared_string_concat / performance / experimental | function drawTable appears to concatenate strings inside a nested loop |
| internal/pdf/draw.go | 902 | append_then_trim_each_iteration / hot_path / stable | function drawTable appends and then reslices in a loop |
| internal/pdf/draw.go | 902 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 904 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 906 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 943 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 945 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 947 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 949 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 983 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 985 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 987 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 989 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 998 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 999 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1044 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1046 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1048 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1050 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1140 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1142 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1144 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1189 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1191 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1237 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1239 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1241 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1243 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1275 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1277 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1279 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1281 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1283 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1289 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1291 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1293 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1295 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1297 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1303 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1305 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1307 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1309 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1311 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1317 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1319 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1321 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1323 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 1325 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 901 | three_index_slice_for_append_safety / performance / stable | function drawTable appends to a subslice without capping its capacity |
| internal/pdf/draw.go | 1188 | three_index_slice_for_append_safety / performance / stable | function drawTable appends to a subslice without capping its capacity |
| internal/pdf/draw.go | 1195 | three_index_slice_for_append_safety / performance / stable | function drawTable appends to a subslice without capping its capacity |
| internal/pdf/draw.go | 1222 | three_index_slice_for_append_safety / performance / stable | function drawTable appends to a subslice without capping its capacity |
| internal/pdf/draw.go | 1236 | three_index_slice_for_append_safety / performance / stable | function drawTable appends to a subslice without capping its capacity |
| internal/pdf/draw.go | 1275 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1277 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1279 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1281 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1283 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1289 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1291 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1293 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1295 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1297 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1303 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1305 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1307 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1309 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1311 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1317 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1319 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1321 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1323 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1325 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1044 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to checkboxBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1046 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to checkboxBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1048 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to checkboxBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1050 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to checkboxBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1140 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to colorBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1142 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to colorBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1144 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to colorBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 998 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to imgNameBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 999 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to imgNameBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 983 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 985 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 987 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 989 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 943 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to xobjBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 945 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to xobjBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 947 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to xobjBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 949 | slice_grow_without_cap_hint / performance / stable | function drawTable appends to xobjBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 1082 | string_concat_in_loop / performance / stable | function drawTable concatenates strings inside a loop |
| internal/pdf/draw.go | 745 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function drawTable matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/draw.go | 735 | go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst / performance / stable | function drawTable matches performance-layer rule go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst |
| internal/pdf/draw.go | 926 | go_perf_layer_caching_cache_key_built_with_fmt / performance / stable | function drawTable matches performance-layer rule go_perf_layer_caching_cache_key_built_with_fmt |
| internal/pdf/draw.go | 773 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function drawTable matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/draw.go | 856 | go_perf_layer_caching_per_request_config_cache_rebuild / performance / stable | function drawTable matches performance-layer rule go_perf_layer_caching_per_request_config_cache_rebuild |
| internal/pdf/draw.go | 745 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function drawTable matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/draw.go | 773 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function drawTable matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/draw.go | 744 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function drawTable matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/draw.go | 773 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function drawTable matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/draw.go | 744 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function drawTable matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/draw.go | 938 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function drawTable matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/draw.go | 744 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function drawTable matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/pdf/draw.go | 806 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function drawTable matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/draw.go | 735 | go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item / performance / stable | function drawTable matches performance-layer rule go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item |
| internal/pdf/draw.go | 735 | go_perf_layer_lazy_loading_lazy_once_hides_slow_first_request / performance / stable | function drawTable matches performance-layer rule go_perf_layer_lazy_loading_lazy_once_hides_slow_first_request |
| internal/pdf/draw.go | 853 | go_perf_layer_logging_overhead_log_fields_built_before_level_check / performance / stable | function drawTable matches performance-layer rule go_perf_layer_logging_overhead_log_fields_built_before_level_check |
| internal/pdf/draw.go | 744 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function drawTable matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/draw.go | 856 | go_perf_layer_runtime_configuration_reflection_config_loaded_per_call / performance / stable | function drawTable matches performance-layer rule go_perf_layer_runtime_configuration_reflection_config_loaded_per_call |
| internal/pdf/draw.go | 907 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function drawTable matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/draw.go | 938 | stable_value_normalization_in_inner_loop / hot_path / stable | function drawTable normalizes a stable value inside a loop |
| internal/pdf/draw.go | 758 | filter_then_count_then_iterate / hot_path / stable | function drawTable traverses the same collection multiple times for filter, count, and process |
| internal/pdf/draw.go | 938 | strings_replace_all_for_single_char / performance / stable | function drawTable uses strings.ReplaceAll for single character replacement |
| internal/pdf/draw.go | 200 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function drawTitle matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/draw.go | 208 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function drawTitle matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/draw.go | 207 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function drawTitle matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/draw.go | 220 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function drawTitle matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/draw.go | 392 | append_then_trim_each_iteration / hot_path / stable | function drawTitleTable appends and then reslices in a loop |
| internal/pdf/draw.go | 392 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 394 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 396 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 474 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 476 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 478 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 480 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 513 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 515 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 517 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 519 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 528 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 529 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 566 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 568 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 570 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 596 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 598 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 610 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 612 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 614 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 616 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 649 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 651 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 653 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 655 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 657 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 663 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 665 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 667 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 669 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 671 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 677 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 679 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 681 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 683 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 685 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 691 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 693 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 695 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 697 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 699 | slice_append_without_prealloc_known_bound / hot_path / stable | function drawTitleTable appends to a slice inside a range loop without visible preallocation |
| internal/pdf/draw.go | 392 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to bgBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 394 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to bgBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 396 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to bgBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 649 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 651 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 653 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 655 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 657 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 663 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 665 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 667 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 669 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 671 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 677 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 679 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 681 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 683 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 685 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 691 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 693 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 695 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 697 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 699 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to borderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 566 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to colorBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 568 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to colorBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 570 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to colorBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 474 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to imgBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 476 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to imgBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 478 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to imgBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 480 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to imgBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 528 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to imgNameBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 529 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to imgNameBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 513 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 515 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 517 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 519 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to placeholderBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 596 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to textPosBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 598 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to textPosBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 610 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to underlineBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 612 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to underlineBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 614 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to underlineBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 616 | slice_grow_without_cap_hint / performance / stable | function drawTitleTable appends to underlineBuf in a loop without a capacity hint |
| internal/pdf/draw.go | 324 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/draw.go | 324 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/draw.go | 364 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/draw.go | 323 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/draw.go | 388 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/draw.go | 318 | go_perf_layer_logging_overhead_log_fields_built_before_level_check / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_logging_overhead_log_fields_built_before_level_check |
| internal/pdf/draw.go | 323 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/draw.go | 323 | go_perf_layer_memory_allocation_closure_capture_allocates_in_loop / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_memory_allocation_closure_capture_allocates_in_loop |
| internal/pdf/draw.go | 397 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function drawTitleTable matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/draw.go | 471 | stable_value_normalization_in_inner_loop / hot_path / stable | function drawTitleTable normalizes a stable value inside a loop |
| internal/pdf/draw.go | 337 | filter_then_count_then_iterate / hot_path / stable | function drawTitleTable traverses the same collection multiple times for filter, count, and process |
| internal/pdf/draw.go | 471 | strings_replace_all_for_single_char / performance / stable | function drawTitleTable uses strings.ReplaceAll for single character replacement |
| internal/pdf/draw.go | 50 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function drawWatermark matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/draw.go | 85 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function drawWatermark matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/draw.go | 1642 | go_perf_layer_logging_overhead_log_fields_built_before_level_check / performance / stable | function drawWidget matches performance-layer rule go_perf_layer_logging_overhead_log_fields_built_before_level_check |
| internal/pdf/draw.go | 1626 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1650 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1654 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1670 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1674 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1677 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1696 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1704 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1712 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1718 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1752 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1758 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/draw.go | 1769 | sprintf_for_simple_string_format / performance / stable | function drawWidget uses fmt.Sprintf with only %s verbs |
| internal/pdf/encryption/encrypt.go | 353 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function FormatDocumentID matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/encryption/encrypt.go | 353 | sprintf_for_simple_string_format / performance / stable | function FormatDocumentID uses fmt.Sprintf with only %s verbs |
| internal/pdf/encryption/encrypt.go | 338 | weak_hash_for_integrity / security / stable | function GenerateDocumentID relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/encryption/encrypt.go | 338 | weak_crypto / security / stable | function GenerateDocumentID uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 115 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function computeEncryptionKey matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/encryption/encrypt.go | 115 | weak_hash_for_integrity / security / stable | function computeEncryptionKey relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/encryption/encrypt.go | 133 | weak_hash_for_integrity / security / stable | function computeEncryptionKey relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/encryption/encrypt.go | 133 | md5_sum_in_loop / performance / stable | function computeEncryptionKey uses `md5.Sum(...)` inside loops. |
| internal/pdf/encryption/encrypt.go | 115 | weak_crypto / security / stable | function computeEncryptionKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 133 | weak_crypto / security / stable | function computeEncryptionKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 275 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function computeObjectKey matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/encryption/encrypt.go | 276 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function computeObjectKey matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/encryption/encrypt.go | 275 | weak_hash_for_integrity / security / stable | function computeObjectKey relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/encryption/encrypt.go | 288 | writer_write_byte_slice_of_string / performance / stable | function computeObjectKey uses `writer.Write([]byte(s))` instead of `io.WriteString(writer, s)`. |
| internal/pdf/encryption/encrypt.go | 275 | weak_crypto / security / stable | function computeObjectKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 99 | allocation_churn_in_loop / performance / stable | function computeOwnerHash allocates new objects inside a loop |
| internal/pdf/encryption/encrypt.go | 99 | likely_n_squared_allocation / performance / experimental | function computeOwnerHash appears to allocate inside a nested loop |
| internal/pdf/encryption/encrypt.go | 94 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function computeOwnerHash matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/encryption/encrypt.go | 94 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function computeOwnerHash matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/encryption/encrypt.go | 99 | make_slice_inside_hot_loop_same_shape / hot_path / stable | function computeOwnerHash recreates scratch slices inside a loop |
| internal/pdf/encryption/encrypt.go | 79 | weak_hash_for_integrity / security / stable | function computeOwnerHash relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/encryption/encrypt.go | 83 | weak_hash_for_integrity / security / stable | function computeOwnerHash relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/encryption/encrypt.go | 83 | md5_sum_in_loop / performance / stable | function computeOwnerHash uses `md5.Sum(...)` inside loops. |
| internal/pdf/encryption/encrypt.go | 79 | weak_crypto / security / stable | function computeOwnerHash uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 83 | weak_crypto / security / stable | function computeOwnerHash uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 154 | allocation_churn_in_loop / performance / stable | function computeUserHash allocates new objects inside a loop |
| internal/pdf/encryption/encrypt.go | 154 | likely_n_squared_allocation / performance / experimental | function computeUserHash appears to allocate inside a nested loop |
| internal/pdf/encryption/encrypt.go | 154 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function computeUserHash matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/encryption/encrypt.go | 144 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function computeUserHash matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/encryption/encrypt.go | 154 | make_slice_inside_hot_loop_same_shape / hot_path / stable | function computeUserHash recreates scratch slices inside a loop |
| internal/pdf/encryption/encrypt.go | 144 | weak_hash_for_integrity / security / stable | function computeUserHash relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/encryption/encrypt.go | 144 | weak_crypto / security / stable | function computeUserHash uses weak cryptographic primitive crypto/md5 |
| internal/pdf/encryption/encrypt.go | 62 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function padPassword matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/compression.go | 42 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function GetCompressBuffer matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/font/compression.go | 42 | go_perf_layer_resource_pooling_buffer_pool_without_max_capacity / performance / stable | function GetCompressBuffer matches performance-layer rule go_perf_layer_resource_pooling_buffer_pool_without_max_capacity |
| internal/pdf/font/metrics.go | 1038 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function EncodeTextForCustomFont matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/font/metrics.go | 916 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateCIDToGIDMap matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/metrics.go | 899 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function GenerateCIDToGIDMap matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/font/metrics.go | 857 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function GenerateCIDToGIDMap matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/metrics.go | 899 | go_perf_layer_resource_pooling_db_pool_created_per_repository / performance / stable | function GenerateCIDToGIDMap matches performance-layer rule go_perf_layer_resource_pooling_db_pool_created_per_repository |
| internal/pdf/font/metrics.go | 899 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateCIDToGIDMap matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/font/metrics.go | 504 | go_perf_layer_lazy_loading_eager_metric_label_cardinality_build / performance / stable | function GenerateFontDescriptorObject matches performance-layer rule go_perf_layer_lazy_loading_eager_metric_label_cardinality_build |
| internal/pdf/font/metrics.go | 517 | builder_write_string_single_byte_literal / performance / stable | function GenerateFontDescriptorObject uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 519 | builder_write_string_single_byte_literal / performance / stable | function GenerateFontDescriptorObject uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 521 | builder_write_string_single_byte_literal / performance / stable | function GenerateFontDescriptorObject uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 483 | go_perf_layer_lazy_loading_eager_metric_label_cardinality_build / performance / stable | function GenerateFontObject matches performance-layer rule go_perf_layer_lazy_loading_eager_metric_label_cardinality_build |
| internal/pdf/font/metrics.go | 940 | slice_append_without_prealloc_known_bound / hot_path / stable | function GenerateToUnicodeCMap appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/metrics.go | 940 | slice_grow_without_cap_hint / performance / stable | function GenerateToUnicodeCMap appends to mappings in a loop without a capacity hint |
| internal/pdf/font/metrics.go | 962 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/font/metrics.go | 937 | go_perf_layer_data_structure_choice_slice_queue_pop_front / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_data_structure_choice_slice_queue_pop_front |
| internal/pdf/font/metrics.go | 950 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/metrics.go | 940 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/font/metrics.go | 1003 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/metrics.go | 937 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/font/metrics.go | 1003 | go_perf_layer_resource_pooling_buffer_pool_without_max_capacity / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_resource_pooling_buffer_pool_without_max_capacity |
| internal/pdf/font/metrics.go | 1003 | go_perf_layer_resource_pooling_db_pool_created_per_repository / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_resource_pooling_db_pool_created_per_repository |
| internal/pdf/font/metrics.go | 1003 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateToUnicodeCMap matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/font/metrics.go | 1005 | writer_write_byte_slice_of_string / performance / stable | function GenerateToUnicodeCMap uses `writer.Write([]byte(s))` instead of `io.WriteString(writer, s)`. |
| internal/pdf/font/metrics.go | 950 | strings_builder_without_grow_known_bound / hot_path / stable | function GenerateToUnicodeCMap uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/metrics.go | 648 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateTrueTypeFontObjects matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/metrics.go | 657 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function GenerateTrueTypeFontObjects matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/font/metrics.go | 648 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function GenerateTrueTypeFontObjects matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/metrics.go | 657 | go_perf_layer_resource_pooling_buffer_pool_without_max_capacity / performance / stable | function GenerateTrueTypeFontObjects matches performance-layer rule go_perf_layer_resource_pooling_buffer_pool_without_max_capacity |
| internal/pdf/font/metrics.go | 657 | go_perf_layer_resource_pooling_db_pool_created_per_repository / performance / stable | function GenerateTrueTypeFontObjects matches performance-layer rule go_perf_layer_resource_pooling_db_pool_created_per_repository |
| internal/pdf/font/metrics.go | 657 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateTrueTypeFontObjects matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/font/metrics.go | 541 | go_perf_layer_lazy_loading_eager_metric_label_cardinality_build / performance / stable | function GenerateWidthsArrayObject matches performance-layer rule go_perf_layer_lazy_loading_eager_metric_label_cardinality_build |
| internal/pdf/font/metrics.go | 550 | builder_write_string_single_byte_literal / performance / stable | function GenerateWidthsArrayObject uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 543 | strings_builder_without_grow_known_bound / hot_path / stable | function GenerateWidthsArrayObject uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/metrics.go | 632 | slice_append_without_prealloc_known_bound / hot_path / stable | function GetAvailableFonts appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/metrics.go | 632 | slice_grow_without_cap_hint / performance / stable | function GetAvailableFonts appends to fonts in a loop without a capacity hint |
| internal/pdf/font/metrics.go | 632 | string_concat_in_loop / performance / stable | function GetAvailableFonts concatenates strings inside a loop |
| internal/pdf/font/metrics.go | 632 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function GetAvailableFonts matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/font/metrics.go | 564 | go_perf_layer_lazy_loading_eager_metric_label_cardinality_build / performance / stable | function GetHelveticaFontResourceString matches performance-layer rule go_perf_layer_lazy_loading_eager_metric_label_cardinality_build |
| internal/pdf/font/metrics.go | 568 | builder_write_string_single_byte_literal / performance / stable | function GetHelveticaFontResourceString uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 571 | builder_write_string_single_byte_literal / performance / stable | function GetHelveticaFontResourceString uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 575 | builder_write_string_single_byte_literal / performance / stable | function GetHelveticaFontResourceString uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 567 | strings_builder_without_grow_known_bound / hot_path / stable | function GetHelveticaFontResourceString uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/metrics.go | 796 | slice_append_without_prealloc_known_bound / hot_path / stable | function generateCIDWidths appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/metrics.go | 796 | slice_grow_without_cap_hint / performance / stable | function generateCIDWidths appends to widths in a loop without a capacity hint |
| internal/pdf/font/metrics.go | 788 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function generateCIDWidths matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/font/metrics.go | 796 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function generateCIDWidths matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/font/metrics.go | 811 | builder_write_string_single_byte_literal / performance / stable | function generateCIDWidths uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 825 | builder_write_string_single_byte_literal / performance / stable | function generateCIDWidths uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 830 | builder_write_string_single_byte_literal / performance / stable | function generateCIDWidths uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 834 | builder_write_string_single_byte_literal / performance / stable | function generateCIDWidths uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 837 | builder_write_string_single_byte_literal / performance / stable | function generateCIDWidths uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/font/metrics.go | 810 | strings_builder_without_grow_known_bound / hot_path / stable | function generateCIDWidths uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/pdfa.go | 149 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function EnsureFontsAvailable matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/pdf/font/pdfa.go | 413 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function GetLiberationFontPostScriptName matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/font/pdfa.go | 238 | http_response_body_not_drained_before_close / idioms / stable | function downloadFonts closes HTTP response resp without draining or consuming the body |
| internal/pdf/font/pdfa.go | 308 | error_logged_and_returned / library / stable | function downloadFonts logs error then returns it |
| internal/pdf/font/pdfa.go | 250 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function downloadFonts matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/pdfa.go | 228 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function downloadFonts matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/pdfa.go | 238 | go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse / performance / stable | function downloadFonts matches performance-layer rule go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse |
| internal/pdf/font/pdfa.go | 271 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function downloadFonts matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/pdf/font/pdfa.go | 243 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function downloadFonts matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/font/pdfa.go | 238 | missing_context / context / stable | function downloadFonts performs context-aware work without accepting context.Context |
| internal/pdf/font/pdfa.go | 238 | timeoutless_http_default_client_or_helper_call / idioms / stable | function downloadFonts uses timeout-less net/http helper or default client state |
| internal/pdf/font/pdfa.go | 99 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function initialize matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/font/pdfa.go | 100 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function initialize matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/pdf/font/pdfa.go | 99 | go_perf_layer_runtime_configuration_gomaxprocs_set_in_library / performance / stable | function initialize matches performance-layer rule go_perf_layer_runtime_configuration_gomaxprocs_set_in_library |
| internal/pdf/font/pdfa.go | 48 | mutable_package_global / idioms / stable | package variable LiberationFontFiles is mutated across function bodies |
| internal/pdf/font/pdfa.go | 27 | mutable_package_global / idioms / stable | package variable LiberationFontMapping is mutated across function bodies |
| internal/pdf/font/pdfa.go | 392 | public_bool_parameter_api / idioms / stable | public function GetMappedFontName uses boolean parameter to control behavior |
| internal/pdf/font/pdfa.go | 73 | rwmutex_without_clear_read_heavy_signal / concurrency / stable | sync.RWMutex appears without a clear read-heavy access pattern |
| internal/pdf/font/registry.go | 250 | go_perf_layer_async_concurrency_mutex_held_during_slow_call / performance / stable | function AssignObjectIDs matches performance-layer rule go_perf_layer_async_concurrency_mutex_held_during_slow_call |
| internal/pdf/font/registry.go | 281 | go_perf_layer_caching_cache_key_built_with_fmt / performance / stable | function AssignObjectIDs matches performance-layer rule go_perf_layer_caching_cache_key_built_with_fmt |
| internal/pdf/font/registry.go | 281 | go_perf_layer_caching_cache_miss_does_duplicate_work / performance / stable | function AssignObjectIDs matches performance-layer rule go_perf_layer_caching_cache_miss_does_duplicate_work |
| internal/pdf/font/registry.go | 281 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function AssignObjectIDs matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/font/registry.go | 239 | allocation_churn_in_loop / performance / stable | function CloneForGeneration allocates new objects inside a loop |
| internal/pdf/font/registry.go | 224 | go_perf_layer_async_concurrency_mutex_held_during_slow_call / performance / stable | function CloneForGeneration matches performance-layer rule go_perf_layer_async_concurrency_mutex_held_during_slow_call |
| internal/pdf/font/registry.go | 228 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function CloneForGeneration matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/font/registry.go | 239 | make_map_inside_hot_loop_same_shape / hot_path / stable | function CloneForGeneration recreates scratch maps inside a loop |
| internal/pdf/font/registry.go | 391 | go_perf_layer_caching_cache_key_built_with_fmt / performance / stable | function GeneratePDFFontResources matches performance-layer rule go_perf_layer_caching_cache_key_built_with_fmt |
| internal/pdf/font/registry.go | 387 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function GeneratePDFFontResources matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/font/registry.go | 387 | strings_builder_without_grow_known_bound / hot_path / stable | function GeneratePDFFontResources uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/font/registry.go | 148 | allocation_churn_in_loop / performance / stable | function GenerateSubsets allocates new objects inside a loop |
| internal/pdf/font/registry.go | 148 | likely_n_squared_allocation / performance / experimental | function GenerateSubsets appears to allocate inside a nested loop |
| internal/pdf/font/registry.go | 143 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function GenerateSubsets matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/font/registry.go | 148 | make_slice_inside_hot_loop_same_shape / hot_path / stable | function GenerateSubsets recreates scratch slices inside a loop |
| internal/pdf/font/registry.go | 290 | go_perf_layer_async_concurrency_mutex_held_during_slow_call / performance / stable | function GetFontReference matches performance-layer rule go_perf_layer_async_concurrency_mutex_held_during_slow_call |
| internal/pdf/font/registry.go | 296 | go_perf_layer_caching_cache_key_built_with_fmt / performance / stable | function GetFontReference matches performance-layer rule go_perf_layer_caching_cache_key_built_with_fmt |
| internal/pdf/font/registry.go | 296 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function GetFontReference matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/font/registry.go | 188 | slice_grow_without_cap_hint / performance / stable | function GetUsedFonts appends to fonts in a loop without a capacity hint |
| internal/pdf/font/registry.go | 332 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function LoadFontsFromDirectory matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/registry.go | 341 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function LoadFontsFromDirectory matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/font/registry.go | 51 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function NewFontRegistry matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/font/registry.go | 77 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function RegisterFontFromBase64 matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/registry.go | 208 | allocation_churn_in_loop / performance / stable | function ResetUsage allocates new objects inside a loop |
| internal/pdf/font/registry.go | 208 | make_map_inside_hot_loop_same_shape / hot_path / stable | function ResetUsage recreates scratch maps inside a loop |
| internal/pdf/font/registry.go | 411 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function ResolveFontName matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/font/registry.go | 16 | rwmutex_without_clear_read_heavy_signal / concurrency / stable | sync.RWMutex appears without a clear read-heavy access pattern |
| internal/pdf/font/subset.go | 23 | map_growth_without_size_hint / hot_path / stable | function SubsetTTF inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 19 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function SubsetTTF matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/font/subset.go | 111 | allocation_churn_in_loop / performance / stable | function buildSubsetFont allocates new objects inside a loop |
| internal/pdf/font/subset.go | 111 | likely_n_squared_allocation / performance / experimental | function buildSubsetFont appears to allocate inside a nested loop |
| internal/pdf/font/subset.go | 163 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildSubsetFont appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/subset.go | 163 | slice_grow_without_cap_hint / performance / stable | function buildSubsetFont appends to tag in a loop without a capacity hint |
| internal/pdf/font/subset.go | 111 | map_growth_without_size_hint / hot_path / stable | function buildSubsetFont inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 78 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function buildSubsetFont matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/font/subset.go | 65 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function buildSubsetFont matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/subset.go | 69 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function buildSubsetFont matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/font/subset.go | 65 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function buildSubsetFont matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/font/subset.go | 65 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function buildSubsetFont matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/font/subset.go | 58 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function buildSubsetFont matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/subset.go | 58 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function buildSubsetFont matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/font/subset.go | 111 | make_slice_inside_hot_loop_same_shape / hot_path / stable | function buildSubsetFont recreates scratch slices inside a loop |
| internal/pdf/font/subset.go | 58 | bytes_buffer_without_grow_known_bound / hot_path / stable | function buildSubsetFont uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 654 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function calculateChecksum matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/subset.go | 680 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function encodeUTF16BE matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/subset.go | 680 | bytes_buffer_without_grow_known_bound / hot_path / stable | function encodeUTF16BE uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 781 | slice_grow_without_cap_hint / performance / stable | function getCompositeComponentGIDs appends to components in a loop without a capacity hint |
| internal/pdf/font/subset.go | 801 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function getCompositeComponentGIDs matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/font/subset.go | 766 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function getCompositeComponentGIDs matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/font/subset.go | 780 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function getCompositeComponentGIDs matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/font/subset.go | 775 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function getCompositeComponentGIDs matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/font/subset.go | 781 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function getCompositeComponentGIDs matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/font/subset.go | 870 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function remapCompositeGIDs matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/font/subset.go | 396 | slice_append_without_prealloc_known_bound / hot_path / stable | function subsetCmap appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/subset.go | 396 | slice_grow_without_cap_hint / performance / stable | function subsetCmap appends to segments in a loop without a capacity hint |
| internal/pdf/font/subset.go | 358 | map_growth_without_size_hint / hot_path / stable | function subsetCmap inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 356 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function subsetCmap matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/font/subset.go | 354 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function subsetCmap matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/font/subset.go | 354 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function subsetCmap matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/pdf/font/subset.go | 351 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function subsetCmap matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/subset.go | 429 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function subsetCmap matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/font/subset.go | 452 | filter_then_count_then_iterate / hot_path / stable | function subsetCmap traverses the same collection multiple times for filter, count, and process |
| internal/pdf/font/subset.go | 351 | bytes_buffer_without_grow_known_bound / hot_path / stable | function subsetCmap uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 292 | allocation_churn_in_loop / performance / stable | function subsetGlyfAndLoca allocates new objects inside a loop |
| internal/pdf/font/subset.go | 271 | map_growth_without_size_hint / hot_path / stable | function subsetGlyfAndLoca inserts into a map in a loop without a visible size hint |
| internal/pdf/font/subset.go | 253 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function subsetGlyfAndLoca matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/subset.go | 269 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function subsetGlyfAndLoca matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/font/subset.go | 265 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function subsetGlyfAndLoca matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/subset.go | 280 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function subsetGlyfAndLoca matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/font/subset.go | 292 | make_slice_inside_hot_loop_same_shape / hot_path / stable | function subsetGlyfAndLoca recreates scratch slices inside a loop |
| internal/pdf/font/subset.go | 265 | bytes_buffer_without_grow_known_bound / hot_path / stable | function subsetGlyfAndLoca uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 210 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function subsetHead matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/subset.go | 225 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function subsetHhea matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/subset.go | 332 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function subsetHmtx matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/subset.go | 332 | bytes_buffer_without_grow_known_bound / hot_path / stable | function subsetHmtx uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/subset.go | 237 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function subsetMaxp matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/font/subset.go | 600 | slice_append_without_prealloc_known_bound / hot_path / stable | function subsetName appends to a slice inside a range loop without visible preallocation |
| internal/pdf/font/subset.go | 600 | slice_grow_without_cap_hint / performance / stable | function subsetName appends to records in a loop without a capacity hint |
| internal/pdf/font/subset.go | 566 | go_perf_layer_algorithmic_complexity_sort_before_linear_dedup / performance / stable | function subsetName matches performance-layer rule go_perf_layer_algorithmic_complexity_sort_before_linear_dedup |
| internal/pdf/font/subset.go | 600 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function subsetName matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/font/subset.go | 567 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function subsetName matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/subset.go | 567 | bytes_buffer_without_grow_known_bound / hot_path / stable | function subsetName uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/font/ttf.go | 761 | map_growth_without_size_hint / hot_path / stable | function GetUsedGlyphs inserts into a map in a loop without a visible size hint |
| internal/pdf/font/ttf.go | 756 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function GetUsedGlyphs matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/font/ttf.go | 61 | full_dataset_load / performance / stable | function LoadTTFFromFile loads an entire payload into memory |
| internal/pdf/font/ttf.go | 102 | error_logged_and_returned / library / stable | function ParseTTF logs error then returns it |
| internal/pdf/font/ttf.go | 113 | error_logged_and_returned / library / stable | function ParseTTF logs error then returns it |
| internal/pdf/font/ttf.go | 116 | error_logged_and_returned / library / stable | function ParseTTF logs error then returns it |
| internal/pdf/font/ttf.go | 119 | error_logged_and_returned / library / stable | function ParseTTF logs error then returns it |
| internal/pdf/font/ttf.go | 76 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ParseTTF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/ttf.go | 76 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ParseTTF matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/ttf.go | 90 | binary_read_for_single_field / performance / stable | function ParseTTF uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 101 | binary_read_for_single_field / performance / stable | function ParseTTF uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 115 | binary_read_for_single_field / performance / stable | function ParseTTF uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 118 | binary_read_for_single_field / performance / stable | function ParseTTF uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 121 | binary_read_for_single_field / performance / stable | function ParseTTF uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 310 | errors_new_for_static_sentinel / performance / stable | function parseCmap calls errors.New multiple times with static strings |
| internal/pdf/font/ttf.go | 331 | error_logged_and_returned / library / stable | function parseCmap logs error then returns it |
| internal/pdf/font/ttf.go | 334 | error_logged_and_returned / library / stable | function parseCmap logs error then returns it |
| internal/pdf/font/ttf.go | 310 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function parseCmap matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/ttf.go | 319 | binary_read_for_single_field / performance / stable | function parseCmap uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 330 | binary_read_for_single_field / performance / stable | function parseCmap uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 333 | binary_read_for_single_field / performance / stable | function parseCmap uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 336 | binary_read_for_single_field / performance / stable | function parseCmap uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 345 | binary_read_for_single_field / performance / stable | function parseCmap uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 360 | binary_read_for_single_field / performance / stable | function parseCmap uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 499 | error_logged_and_returned / library / stable | function parseCmapFormat12 logs error then returns it |
| internal/pdf/font/ttf.go | 502 | error_logged_and_returned / library / stable | function parseCmapFormat12 logs error then returns it |
| internal/pdf/font/ttf.go | 492 | binary_read_for_single_field / performance / stable | function parseCmapFormat12 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 498 | binary_read_for_single_field / performance / stable | function parseCmapFormat12 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 501 | binary_read_for_single_field / performance / stable | function parseCmapFormat12 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 504 | binary_read_for_single_field / performance / stable | function parseCmapFormat12 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 395 | error_logged_and_returned / library / stable | function parseCmapFormat4 logs error then returns it |
| internal/pdf/font/ttf.go | 412 | go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans / performance / stable | function parseCmapFormat4 matches performance-layer rule go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans |
| internal/pdf/font/ttf.go | 461 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function parseCmapFormat4 matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/font/ttf.go | 390 | go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop / performance / stable | function parseCmapFormat4 matches performance-layer rule go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop |
| internal/pdf/font/ttf.go | 388 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function parseCmapFormat4 matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/ttf.go | 412 | go_perf_layer_memory_allocation_closure_capture_allocates_in_loop / performance / stable | function parseCmapFormat4 matches performance-layer rule go_perf_layer_memory_allocation_closure_capture_allocates_in_loop |
| internal/pdf/font/ttf.go | 394 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function parseCmapFormat4 matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/font/ttf.go | 394 | binary_read_for_single_field / performance / stable | function parseCmapFormat4 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 402 | binary_read_for_single_field / performance / stable | function parseCmapFormat4 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 414 | binary_read_for_single_field / performance / stable | function parseCmapFormat4 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 426 | binary_read_for_single_field / performance / stable | function parseCmapFormat4 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 434 | binary_read_for_single_field / performance / stable | function parseCmapFormat4 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 443 | binary_read_for_single_field / performance / stable | function parseCmapFormat4 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 464 | binary_read_for_single_field / performance / stable | function parseCmapFormat4 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 176 | errors_new_for_static_sentinel / performance / stable | function parseHead calls errors.New multiple times with static strings |
| internal/pdf/font/ttf.go | 197 | error_logged_and_returned / library / stable | function parseHead logs error then returns it |
| internal/pdf/font/ttf.go | 200 | error_logged_and_returned / library / stable | function parseHead logs error then returns it |
| internal/pdf/font/ttf.go | 203 | error_logged_and_returned / library / stable | function parseHead logs error then returns it |
| internal/pdf/font/ttf.go | 188 | binary_read_for_single_field / performance / stable | function parseHead uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 196 | binary_read_for_single_field / performance / stable | function parseHead uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 199 | binary_read_for_single_field / performance / stable | function parseHead uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 202 | binary_read_for_single_field / performance / stable | function parseHead uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 205 | binary_read_for_single_field / performance / stable | function parseHead uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 216 | errors_new_for_static_sentinel / performance / stable | function parseHhea calls errors.New multiple times with static strings |
| internal/pdf/font/ttf.go | 229 | error_logged_and_returned / library / stable | function parseHhea logs error then returns it |
| internal/pdf/font/ttf.go | 232 | error_logged_and_returned / library / stable | function parseHhea logs error then returns it |
| internal/pdf/font/ttf.go | 228 | binary_read_for_single_field / performance / stable | function parseHhea uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 231 | binary_read_for_single_field / performance / stable | function parseHhea uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 234 | binary_read_for_single_field / performance / stable | function parseHhea uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 268 | errors_new_for_static_sentinel / performance / stable | function parseHmtx calls errors.New multiple times with static strings |
| internal/pdf/font/ttf.go | 290 | error_logged_and_returned / library / stable | function parseHmtx logs error then returns it |
| internal/pdf/font/ttf.go | 284 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function parseHmtx matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/font/ttf.go | 268 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function parseHmtx matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/ttf.go | 279 | binary_read_for_single_field / performance / stable | function parseHmtx uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 289 | binary_read_for_single_field / performance / stable | function parseHmtx uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 245 | errors_new_for_static_sentinel / performance / stable | function parseMaxp calls errors.New multiple times with static strings |
| internal/pdf/font/ttf.go | 257 | binary_read_for_single_field / performance / stable | function parseMaxp uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 535 | error_logged_and_returned / library / stable | function parseName logs error then returns it |
| internal/pdf/font/ttf.go | 546 | error_logged_and_returned / library / stable | function parseName logs error then returns it |
| internal/pdf/font/ttf.go | 549 | error_logged_and_returned / library / stable | function parseName logs error then returns it |
| internal/pdf/font/ttf.go | 552 | error_logged_and_returned / library / stable | function parseName logs error then returns it |
| internal/pdf/font/ttf.go | 555 | error_logged_and_returned / library / stable | function parseName logs error then returns it |
| internal/pdf/font/ttf.go | 558 | error_logged_and_returned / library / stable | function parseName logs error then returns it |
| internal/pdf/font/ttf.go | 525 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function parseName matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/font/ttf.go | 525 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function parseName matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/font/ttf.go | 534 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 537 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 545 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 548 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 551 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 554 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 557 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 560 | binary_read_for_single_field / performance / stable | function parseName uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 624 | errors_new_for_static_sentinel / performance / stable | function parseOS2 calls errors.New multiple times with static strings |
| internal/pdf/font/ttf.go | 669 | error_logged_and_returned / library / stable | function parseOS2 logs error then returns it |
| internal/pdf/font/ttf.go | 672 | error_logged_and_returned / library / stable | function parseOS2 logs error then returns it |
| internal/pdf/font/ttf.go | 634 | binary_read_for_single_field / performance / stable | function parseOS2 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 643 | binary_read_for_single_field / performance / stable | function parseOS2 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 653 | binary_read_for_single_field / performance / stable | function parseOS2 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 671 | binary_read_for_single_field / performance / stable | function parseOS2 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 674 | binary_read_for_single_field / performance / stable | function parseOS2 uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 699 | errors_new_for_static_sentinel / performance / stable | function parsePost calls errors.New multiple times with static strings |
| internal/pdf/font/ttf.go | 713 | binary_read_for_single_field / performance / stable | function parsePost uses binary.Read for a single scalar field |
| internal/pdf/font/ttf.go | 723 | binary_read_for_single_field / performance / stable | function parsePost uses binary.Read for a single scalar field |
| internal/pdf/form/xfdf.go | 685 | slice_append_without_prealloc_known_bound / hot_path / stable | function DetectFormFields appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 685 | slice_grow_without_cap_hint / performance / stable | function DetectFormFields appends to names in a loop without a capacity hint |
| internal/pdf/form/xfdf.go | 685 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function DetectFormFields matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/form/xfdf.go | 526 | slice_append_without_prealloc_known_bound / hot_path / stable | function DetectFormFieldsAdvanced appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 526 | slice_grow_without_cap_hint / performance / stable | function DetectFormFieldsAdvanced appends to pairs in a loop without a capacity hint |
| internal/pdf/form/xfdf.go | 497 | regexp_compile_in_hot_path / hot_path / stable | function DetectFormFieldsAdvanced compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 509 | regexp_compile_in_hot_path / hot_path / stable | function DetectFormFieldsAdvanced compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 547 | map_growth_without_size_hint / hot_path / stable | function DetectFormFieldsAdvanced inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 489 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function DetectFormFieldsAdvanced matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/form/xfdf.go | 471 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function DetectFormFieldsAdvanced matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/form/xfdf.go | 470 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function DetectFormFieldsAdvanced matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 492 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function DetectFormFieldsAdvanced matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/form/xfdf.go | 470 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function DetectFormFieldsAdvanced matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/form/xfdf.go | 893 | append_then_trim_each_iteration / hot_path / stable | function FillPDFWithXFDF appends and then reslices in a loop |
| internal/pdf/form/xfdf.go | 849 | slice_append_without_prealloc_known_bound / hot_path / stable | function FillPDFWithXFDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 953 | slice_append_without_prealloc_known_bound / hot_path / stable | function FillPDFWithXFDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/form/xfdf.go | 984 | three_index_slice_for_append_safety / performance / stable | function FillPDFWithXFDF appends to a subslice without capping its capacity |
| internal/pdf/form/xfdf.go | 849 | slice_grow_without_cap_hint / performance / stable | function FillPDFWithXFDF appends to allJobs in a loop without a capacity hint |
| internal/pdf/form/xfdf.go | 953 | slice_grow_without_cap_hint / performance / stable | function FillPDFWithXFDF appends to textJobs in a loop without a capacity hint |
| internal/pdf/form/xfdf.go | 891 | repeated_slice_clone_in_loop / hot_path / stable | function FillPDFWithXFDF clones slices inside a loop |
| internal/pdf/form/xfdf.go | 907 | repeated_slice_clone_in_loop / hot_path / stable | function FillPDFWithXFDF clones slices inside a loop |
| internal/pdf/form/xfdf.go | 928 | repeated_slice_clone_in_loop / hot_path / stable | function FillPDFWithXFDF clones slices inside a loop |
| internal/pdf/form/xfdf.go | 878 | regexp_compile_in_hot_path / hot_path / stable | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 886 | regexp_compile_in_hot_path / hot_path / stable | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 903 | regexp_compile_in_hot_path / hot_path / stable | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 909 | regexp_compile_in_hot_path / hot_path / stable | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 914 | regexp_compile_in_hot_path / hot_path / stable | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 924 | regexp_compile_in_hot_path / hot_path / stable | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 965 | regexp_compile_in_hot_path / hot_path / stable | function FillPDFWithXFDF compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 891 | byte_string_conversion_in_loop / hot_path / stable | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 907 | byte_string_conversion_in_loop / hot_path / stable | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 928 | byte_string_conversion_in_loop / hot_path / stable | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1015 | byte_string_conversion_in_loop / hot_path / stable | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1038 | byte_string_conversion_in_loop / hot_path / stable | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1051 | byte_string_conversion_in_loop / hot_path / stable | function FillPDFWithXFDF converts between bytes and strings inside a loop |
| internal/pdf/form/xfdf.go | 1058 | strconv_repeat_on_same_binding / hot_path / stable | function FillPDFWithXFDF converts the same string input with strconv multiple times |
| internal/pdf/form/xfdf.go | 878 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 902 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 965 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 971 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1010 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1013 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1033 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1049 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 1070 | fmt_hot_path / performance / stable | function FillPDFWithXFDF formats strings with fmt inside a loop |
| internal/pdf/form/xfdf.go | 874 | map_growth_without_size_hint / hot_path / stable | function FillPDFWithXFDF inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 1043 | error_logged_and_returned / library / stable | function FillPDFWithXFDF logs error then returns it |
| internal/pdf/form/xfdf.go | 744 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/form/xfdf.go | 752 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/form/xfdf.go | 783 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/form/xfdf.go | 763 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/form/xfdf.go | 745 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/form/xfdf.go | 752 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/form/xfdf.go | 889 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/form/xfdf.go | 744 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 744 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/form/xfdf.go | 1010 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/form/xfdf.go | 744 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/form/xfdf.go | 744 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function FillPDFWithXFDF matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/form/xfdf.go | 794 | map_lookup_double_access / performance / stable | function FillPDFWithXFDF performs double map lookup for same key |
| internal/pdf/form/xfdf.go | 1040 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function FillPDFWithXFDF recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 1020 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function FillPDFWithXFDF recreates a strings.Builder inside a loop |
| internal/pdf/form/xfdf.go | 872 | filter_then_count_then_iterate / hot_path / stable | function FillPDFWithXFDF traverses the same collection multiple times for filter, count, and process |
| internal/pdf/form/xfdf.go | 1021 | builder_write_string_single_byte_literal / performance / stable | function FillPDFWithXFDF uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/form/xfdf.go | 1025 | builder_write_string_single_byte_literal / performance / stable | function FillPDFWithXFDF uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/form/xfdf.go | 1028 | builder_write_string_single_byte_literal / performance / stable | function FillPDFWithXFDF uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/form/xfdf.go | 913 | strings_tolower_equalfold / performance / stable | function FillPDFWithXFDF uses `strings.ToLower(...) == strings.ToLower(...)` instead of `strings.EqualFold(...)`. |
| internal/pdf/form/xfdf.go | 1042 | writer_write_byte_slice_of_string / performance / stable | function FillPDFWithXFDF uses `writer.Write([]byte(s))` instead of `io.WriteString(writer, s)`. |
| internal/pdf/form/xfdf.go | 1040 | bytes_buffer_without_grow_known_bound / hot_path / stable | function FillPDFWithXFDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 1020 | strings_builder_without_grow_known_bound / hot_path / stable | function FillPDFWithXFDF uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 709 | map_growth_without_size_hint / hot_path / stable | function FillPDFWithXFDFAdvanced inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 707 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function FillPDFWithXFDFAdvanced matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/form/xfdf.go | 694 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function FillPDFWithXFDFAdvanced matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/form/xfdf.go | 693 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function FillPDFWithXFDFAdvanced matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 60 | map_growth_without_size_hint / hot_path / stable | function ParseXFDF inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 56 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function ParseXFDF matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/form/xfdf.go | 53 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ParseXFDF matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 721 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function buildXFDF matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/form/xfdf.go | 81 | strings_replace_all_for_single_char / performance / stable | function decodeHexString uses strings.ReplaceAll for single character replacement |
| internal/pdf/form/xfdf.go | 645 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function decompressStreams matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/form/xfdf.go | 651 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function decompressStreams matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 641 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function decompressStreams matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/form/xfdf.go | 659 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function decompressStreams recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 659 | bytes_buffer_without_grow_known_bound / hot_path / stable | function decompressStreams uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 624 | map_growth_without_size_hint / hot_path / stable | function detectFormFieldsNaive inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 602 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function detectFormFieldsNaive matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/form/xfdf.go | 604 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function detectFormFieldsNaive matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/form/xfdf.go | 604 | go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | function detectFormFieldsNaive matches performance-layer rule go_perf_layer_data_structure_choice_map_string_bool_for_membership |
| internal/pdf/form/xfdf.go | 604 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function detectFormFieldsNaive matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/form/xfdf.go | 602 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function detectFormFieldsNaive matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 1239 | regexp_compile_in_hot_path / hot_path / stable | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1252 | regexp_compile_in_hot_path / hot_path / stable | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1254 | regexp_compile_in_hot_path / hot_path / stable | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1260 | regexp_compile_in_hot_path / hot_path / stable | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1286 | regexp_compile_in_hot_path / hot_path / stable | function fillXFDFInObjStmBody compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 1251 | map_growth_without_size_hint / hot_path / stable | function fillXFDFInObjStmBody inserts into a map in a loop without a visible size hint |
| internal/pdf/form/xfdf.go | 1188 | go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans |
| internal/pdf/form/xfdf.go | 1203 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/form/xfdf.go | 1197 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/form/xfdf.go | 1202 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/form/xfdf.go | 1188 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/form/xfdf.go | 1288 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/form/xfdf.go | 1187 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 1187 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/form/xfdf.go | 1182 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/form/xfdf.go | 1187 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function fillXFDFInObjStmBody matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/form/xfdf.go | 1318 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function fillXFDFInObjStmBody recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 1336 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function fillXFDFInObjStmBody recreates a bytes.Buffer inside a loop |
| internal/pdf/form/xfdf.go | 1318 | bytes_buffer_without_grow_known_bound / hot_path / stable | function fillXFDFInObjStmBody uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 1356 | strings_builder_without_grow_known_bound / hot_path / stable | function fillXFDFInObjStmBody uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/form/xfdf.go | 1146 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function fillXFDFInObjectStreams matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/form/xfdf.go | 1150 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function fillXFDFInObjectStreams matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/form/xfdf.go | 1146 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function fillXFDFInObjectStreams matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/form/xfdf.go | 348 | regexp_compile_in_hot_path / hot_path / stable | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 360 | regexp_compile_in_hot_path / hot_path / stable | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 362 | regexp_compile_in_hot_path / hot_path / stable | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 364 | regexp_compile_in_hot_path / hot_path / stable | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 369 | regexp_compile_in_hot_path / hot_path / stable | function findWidgetAnnotationsForName compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 339 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function findWidgetAnnotationsForName matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/form/xfdf.go | 402 | regexp_compile_in_hot_path / hot_path / stable | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 441 | regexp_compile_in_hot_path / hot_path / stable | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/form/xfdf.go | 443 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/form/xfdf.go | 397 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/form/xfdf.go | 398 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/form/xfdf.go | 1453 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function replaceOrInsertPDFEntry matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/form/xfdf.go | 1453 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function replaceOrInsertPDFEntry matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/form/xfdf.go | 1454 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function replaceOrInsertPDFEntry matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/form/xfdf.go | 308 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function resolveValueRef matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/form/xfdf.go | 312 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function resolveValueRef matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/form/xfdf.go | 384 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function trailerHasEncrypt matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/form/xfdf.go | 235 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function traverseField matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/form/xfdf.go | 227 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function traverseField matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/form/xfdf.go | 227 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function traverseField matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/form/xfdf.go | 110 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function tryFlateDecompress matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/form/xfdf.go | 94 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function tryZlibDecompress matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/form/xfdf.go | 1431 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function updateObjStmFieldValue matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/form/xfdf.go | 1418 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function updateObjStmFieldValue matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/form/xfdf.go | 1475 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function xfdfValueToPDFName matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/generator.go | 56 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function AppendPageAnnot matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/generator.go | 57 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function AppendPageAnnot matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/generator.go | 667 | append_then_trim_each_iteration / hot_path / stable | function GenerateTemplatePDF appends and then reslices in a loop |
| internal/pdf/generator.go | 427 | slice_append_without_prealloc_known_bound / hot_path / stable | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 674 | slice_append_without_prealloc_known_bound / hot_path / stable | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 676 | slice_append_without_prealloc_known_bound / hot_path / stable | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 678 | slice_append_without_prealloc_known_bound / hot_path / stable | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 680 | slice_append_without_prealloc_known_bound / hot_path / stable | function GenerateTemplatePDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/generator.go | 427 | slice_grow_without_cap_hint / performance / stable | function GenerateTemplatePDF appends to allWidgetIDs in a loop without a capacity hint |
| internal/pdf/generator.go | 674 | slice_grow_without_cap_hint / performance / stable | function GenerateTemplatePDF appends to annotBuf in a loop without a capacity hint |
| internal/pdf/generator.go | 676 | slice_grow_without_cap_hint / performance / stable | function GenerateTemplatePDF appends to annotBuf in a loop without a capacity hint |
| internal/pdf/generator.go | 678 | slice_grow_without_cap_hint / performance / stable | function GenerateTemplatePDF appends to annotBuf in a loop without a capacity hint |
| internal/pdf/generator.go | 680 | slice_grow_without_cap_hint / performance / stable | function GenerateTemplatePDF appends to annotBuf in a loop without a capacity hint |
| internal/pdf/generator.go | 1313 | string_builder_write_string_vs_plus / performance / stable | function GenerateTemplatePDF concatenates strings before WriteString |
| internal/pdf/generator.go | 846 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 861 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 876 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1111 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1113 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1123 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1173 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1181 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1183 | fmt_hot_path / performance / stable | function GenerateTemplatePDF formats strings with fmt inside a loop |
| internal/pdf/generator.go | 162 | map_growth_without_size_hint / hot_path / stable | function GenerateTemplatePDF inserts into a map in a loop without a visible size hint |
| internal/pdf/generator.go | 606 | go_perf_layer_algorithmic_complexity_per_request_topk_full_sort / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_algorithmic_complexity_per_request_topk_full_sort |
| internal/pdf/generator.go | 105 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/generator.go | 1225 | go_perf_layer_algorithmic_complexity_sort_before_linear_dedup / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_algorithmic_complexity_sort_before_linear_dedup |
| internal/pdf/generator.go | 88 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/generator.go | 106 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/generator.go | 92 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/generator.go | 92 | go_perf_layer_data_structure_choice_linked_list_for_cache_iteration / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_data_structure_choice_linked_list_for_cache_iteration |
| internal/pdf/generator.go | 92 | go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_data_structure_choice_map_string_bool_for_membership |
| internal/pdf/generator.go | 88 | go_perf_layer_data_structure_choice_slice_queue_pop_front / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_data_structure_choice_slice_queue_pop_front |
| internal/pdf/generator.go | 83 | go_perf_layer_database_access_count_query_before_paged_fetch / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_database_access_count_query_before_paged_fetch |
| internal/pdf/generator.go | 250 | go_perf_layer_database_access_select_star_on_hot_query / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_database_access_select_star_on_hot_query |
| internal/pdf/generator.go | 171 | go_perf_layer_database_access_transaction_per_row_bulk_write / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_database_access_transaction_per_row_bulk_write |
| internal/pdf/generator.go | 107 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/generator.go | 92 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/generator.go | 95 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/generator.go | 88 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/generator.go | 83 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/generator.go | 90 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/pdf/generator.go | 85 | go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop |
| internal/pdf/generator.go | 97 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/generator.go | 916 | go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item |
| internal/pdf/generator.go | 95 | go_perf_layer_lazy_loading_eager_load_optional_config / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_lazy_loading_eager_load_optional_config |
| internal/pdf/generator.go | 177 | go_perf_layer_lazy_loading_eager_metric_label_cardinality_build / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_lazy_loading_eager_metric_label_cardinality_build |
| internal/pdf/generator.go | 95 | go_perf_layer_lazy_loading_eager_template_parse_for_unused_routes / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_lazy_loading_eager_template_parse_for_unused_routes |
| internal/pdf/generator.go | 498 | go_perf_layer_logging_overhead_log_fields_built_before_level_check / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_logging_overhead_log_fields_built_before_level_check |
| internal/pdf/generator.go | 83 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/generator.go | 83 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/generator.go | 83 | go_perf_layer_resource_pooling_db_pool_created_per_repository / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_resource_pooling_db_pool_created_per_repository |
| internal/pdf/generator.go | 177 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/generator.go | 95 | go_perf_layer_runtime_configuration_reflection_config_loaded_per_call / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_runtime_configuration_reflection_config_loaded_per_call |
| internal/pdf/generator.go | 126 | go_perf_layer_serialization_base64_roundtrip_for_binary_transport / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_serialization_base64_roundtrip_for_binary_transport |
| internal/pdf/generator.go | 481 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/generator.go | 92 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/pdf/generator.go | 83 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function GenerateTemplatePDF matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/generator.go | 606 | stable_value_normalization_in_inner_loop / hot_path / stable | function GenerateTemplatePDF normalizes a stable value inside a loop |
| internal/pdf/generator.go | 697 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function GenerateTemplatePDF recreates a strings.Builder inside a loop |
| internal/pdf/generator.go | 1006 | weak_hash_for_integrity / security / stable | function GenerateTemplatePDF relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/generator.go | 1019 | weak_hash_for_integrity / security / stable | function GenerateTemplatePDF relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/generator.go | 157 | filter_then_count_then_iterate / hot_path / stable | function GenerateTemplatePDF traverses the same collection multiple times for filter, count, and process |
| internal/pdf/generator.go | 916 | fmt_sprintf_hex_to_string / performance / stable | function GenerateTemplatePDF uses `fmt.Sprintf("%x", ...)` instead of a direct hex formatter when only the string is needed. |
| internal/pdf/generator.go | 499 | builder_write_string_single_byte_literal / performance / stable | function GenerateTemplatePDF uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/generator.go | 506 | builder_write_string_single_byte_literal / performance / stable | function GenerateTemplatePDF uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/generator.go | 83 | bytes_buffer_without_grow_known_bound / hot_path / stable | function GenerateTemplatePDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/generator.go | 264 | sprintf_for_simple_int_to_string / performance / stable | function GenerateTemplatePDF uses fmt.Sprintf for integer-to-string conversion |
| internal/pdf/generator.go | 518 | sprintf_for_simple_string_format / performance / stable | function GenerateTemplatePDF uses fmt.Sprintf with only %s verbs |
| internal/pdf/generator.go | 971 | sprintf_for_simple_string_format / performance / stable | function GenerateTemplatePDF uses fmt.Sprintf with only %s verbs |
| internal/pdf/generator.go | 986 | sprintf_for_simple_string_format / performance / stable | function GenerateTemplatePDF uses fmt.Sprintf with only %s verbs |
| internal/pdf/generator.go | 1020 | sprintf_for_simple_string_format / performance / stable | function GenerateTemplatePDF uses fmt.Sprintf with only %s verbs |
| internal/pdf/generator.go | 498 | strings_builder_without_grow_known_bound / hot_path / stable | function GenerateTemplatePDF uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/generator.go | 606 | strings_replace_all_for_single_char / performance / stable | function GenerateTemplatePDF uses strings.ReplaceAll for single character replacement |
| internal/pdf/generator.go | 1006 | weak_crypto / security / stable | function GenerateTemplatePDF uses weak cryptographic primitive crypto/md5 |
| internal/pdf/generator.go | 1019 | weak_crypto / security / stable | function GenerateTemplatePDF uses weak cryptographic primitive crypto/md5 |
| internal/pdf/generator.go | 1727 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function autoResolveMathFonts matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/generator.go | 1727 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function autoResolveMathFonts matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/generator.go | 1649 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function collectAllStandardFontsInTemplate matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/generator.go | 1599 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function collectAllStandardFontsInTemplate matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/generator.go | 1599 | go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | function collectAllStandardFontsInTemplate matches performance-layer rule go_perf_layer_data_structure_choice_map_string_bool_for_membership |
| internal/pdf/generator.go | 1599 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function collectAllStandardFontsInTemplate matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/generator.go | 1628 | filter_then_count_then_iterate / hot_path / stable | function collectAllStandardFontsInTemplate traverses the same collection multiple times for filter, count, and process |
| internal/pdf/generator.go | 1755 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function collectUnregisteredMathFontNames matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/generator.go | 1755 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function collectUnregisteredMathFontNames matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/generator.go | 1755 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function collectUnregisteredMathFontNames matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/generator.go | 1764 | strings_splitn_two_index_zero_cut / performance / stable | function collectUnregisteredMathFontNames uses `strings.SplitN(..., 2)[0]` instead of `strings.Cut(...)`. |
| internal/pdf/generator.go | 1524 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function collectUsedStandardFonts matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/generator.go | 1470 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function collectUsedStandardFonts matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/generator.go | 1470 | go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | function collectUsedStandardFonts matches performance-layer rule go_perf_layer_data_structure_choice_map_string_bool_for_membership |
| internal/pdf/generator.go | 1470 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function collectUsedStandardFonts matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/generator.go | 1503 | filter_then_count_then_iterate / hot_path / stable | function collectUsedStandardFonts traverses the same collection multiple times for filter, count, and process |
| internal/pdf/generator.go | 1377 | fmt_hot_path / performance / stable | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1380 | fmt_hot_path / performance / stable | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1404 | fmt_hot_path / performance / stable | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1414 | fmt_hot_path / performance / stable | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1427 | fmt_hot_path / performance / stable | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1440 | fmt_hot_path / performance / stable | function generateAllContentWithImages formats strings with fmt inside a loop |
| internal/pdf/generator.go | 1332 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function generateAllContentWithImages matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/generator.go | 1342 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function generateAllContentWithImages matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/generator.go | 1378 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function generateAllContentWithImages matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/generator.go | 1334 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function generateAllContentWithImages matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/generator.go | 1338 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function generateAllContentWithImages matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/generator.go | 1334 | go_perf_layer_lazy_loading_eager_template_parse_for_unused_routes / performance / stable | function generateAllContentWithImages matches performance-layer rule go_perf_layer_lazy_loading_eager_template_parse_for_unused_routes |
| internal/pdf/generator.go | 1332 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function generateAllContentWithImages matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/pdf/generator.go | 1380 | sprintf_for_simple_int_to_string / performance / stable | function generateAllContentWithImages uses fmt.Sprintf for integer-to-string conversion |
| internal/pdf/generator.go | 1427 | sprintf_for_simple_int_to_string / performance / stable | function generateAllContentWithImages uses fmt.Sprintf for integer-to-string conversion |
| internal/pdf/helpers.go | 108 | regexp_compile_in_hot_path / hot_path / stable | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/helpers.go | 147 | regexp_compile_in_hot_path / hot_path / stable | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/helpers.go | 149 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/helpers.go | 103 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/helpers.go | 104 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/helpers.go | 22 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function trailerHasEncrypt matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/helpers.go | 48 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function tryFlateDecompress matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/helpers.go | 32 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function tryZlibDecompress matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/image.go | 500 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function CreateEncryptedImageXObject matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/image.go | 443 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function CreateImageXObject matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/image.go | 203 | copy_append_idiom_waste / performance / stable | function DecodeImageData clones a slice via append(nil, src...) |
| internal/pdf/image.go | 236 | copy_append_idiom_waste / performance / stable | function DecodeImageData clones a slice via append(nil, src...) |
| internal/pdf/image.go | 98 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/image.go | 107 | go_perf_layer_async_concurrency_mutex_held_during_slow_call / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_async_concurrency_mutex_held_during_slow_call |
| internal/pdf/image.go | 107 | go_perf_layer_caching_cache_miss_does_duplicate_work / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_caching_cache_miss_does_duplicate_work |
| internal/pdf/image.go | 98 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/image.go | 130 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/image.go | 98 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/image.go | 203 | go_perf_layer_error_handling_cost_multierror_append_for_success_path / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_error_handling_cost_multierror_append_for_success_path |
| internal/pdf/image.go | 126 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/image.go | 98 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/image.go | 176 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/image.go | 106 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/pdf/image.go | 109 | go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_hot_path_optimization_defer_for_simple_unlock_in_hot_loop |
| internal/pdf/image.go | 203 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/image.go | 124 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/image.go | 98 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/image.go | 176 | go_perf_layer_resource_pooling_buffer_pool_without_max_capacity / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_resource_pooling_buffer_pool_without_max_capacity |
| internal/pdf/image.go | 191 | go_perf_layer_resource_pooling_db_pool_created_per_repository / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_resource_pooling_db_pool_created_per_repository |
| internal/pdf/image.go | 191 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/image.go | 124 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/pdf/image.go | 98 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function DecodeImageData matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/image.go | 126 | fmt_errorf_without_wrap_verb / performance / stable | function DecodeImageData uses %v instead of %w for error wrapping |
| internal/pdf/image.go | 150 | fmt_errorf_without_wrap_verb / performance / stable | function DecodeImageData uses %v instead of %w for error wrapping |
| internal/pdf/image.go | 126 | error_wrapping_misuse / errors / stable | function DecodeImageData uses fmt.Errorf without %w while referencing err |
| internal/pdf/image.go | 150 | error_wrapping_misuse / errors / stable | function DecodeImageData uses fmt.Errorf without %w while referencing err |
| internal/pdf/image.go | 59 | go_perf_layer_caching_unbounded_cache_map / performance / stable | function ResetImageCache matches performance-layer rule go_perf_layer_caching_unbounded_cache_map |
| internal/pdf/image.go | 250 | go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst / performance / stable | function convertToRGB matches performance-layer rule go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst |
| internal/pdf/image.go | 266 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function convertToRGB matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/image.go | 313 | go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst / performance / stable | function convertToRGBWithAlpha matches performance-layer rule go_perf_layer_async_concurrency_channel_buffer_too_small_for_known_burst |
| internal/pdf/image.go | 332 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function convertToRGBWithAlpha matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/image.go | 583 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function drawImageWithXObject matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/image.go | 21 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function fmtNumImg matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/image.go | 50 | range_over_string_by_index / performance / stable | function fnv1aHash iterates string by byte index |
| internal/pdf/image.go | 35 | rwmutex_without_clear_read_heavy_signal / concurrency / stable | sync.RWMutex appears without a clear read-heavy access pattern |
| internal/pdf/links.go | 57 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function CreateLinkAnnotation matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/links.go | 29 | sprintf_for_simple_string_format / performance / stable | function CreateLinkAnnotation uses fmt.Sprintf with only %s verbs |
| internal/pdf/links.go | 87 | strings_hasprefix_then_trimprefix / performance / stable | function ParseLink checks HasPrefix then TrimPrefix (use CutPrefix) |
| internal/pdf/links.go | 81 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function ParseLink matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/links.go | 80 | public_bool_parameter_api / idioms / stable | public function ParseLink uses boolean parameter to control behavior |
| internal/pdf/merge.go | 379 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function IsFormFieldObject matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge.go | 55 | allocation_churn_in_loop / performance / stable | function MergePDFs allocates new objects inside a loop |
| internal/pdf/merge.go | 69 | allocation_churn_in_loop / performance / stable | function MergePDFs allocates new objects inside a loop |
| internal/pdf/merge.go | 55 | likely_n_squared_allocation / performance / experimental | function MergePDFs appears to allocate inside a nested loop |
| internal/pdf/merge.go | 69 | likely_n_squared_allocation / performance / experimental | function MergePDFs appears to allocate inside a nested loop |
| internal/pdf/merge.go | 100 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 118 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 130 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 145 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 152 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 180 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 197 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 130 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to appended in a loop without a capacity hint |
| internal/pdf/merge.go | 197 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to b in a loop without a capacity hint |
| internal/pdf/merge.go | 118 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to fileObjects in a loop without a capacity hint |
| internal/pdf/merge.go | 180 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to kids in a loop without a capacity hint |
| internal/pdf/merge.go | 152 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to mergedFormFields in a loop without a capacity hint |
| internal/pdf/merge.go | 145 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to mergedPages in a loop without a capacity hint |
| internal/pdf/merge.go | 100 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to pagesFromTree in a loop without a capacity hint |
| internal/pdf/merge.go | 55 | repeated_map_clone_in_loop / hot_path / stable | function MergePDFs clones maps inside a loop |
| internal/pdf/merge.go | 209 | regexp_compile_in_hot_path / hot_path / stable | function MergePDFs compiles regular expressions inside a loop |
| internal/pdf/merge.go | 167 | string_concat_in_loop / performance / stable | function MergePDFs concatenates strings inside a loop |
| internal/pdf/merge.go | 169 | string_concat_in_loop / performance / stable | function MergePDFs concatenates strings inside a loop |
| internal/pdf/merge.go | 169 | fmt_hot_path / performance / stable | function MergePDFs formats strings with fmt inside a loop |
| internal/pdf/merge.go | 180 | fmt_hot_path / performance / stable | function MergePDFs formats strings with fmt inside a loop |
| internal/pdf/merge.go | 229 | fmt_hot_path / performance / stable | function MergePDFs formats strings with fmt inside a loop |
| internal/pdf/merge.go | 61 | map_growth_without_size_hint / hot_path / stable | function MergePDFs inserts into a map in a loop without a visible size hint |
| internal/pdf/merge.go | 37 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge.go | 51 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/merge.go | 137 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge.go | 20 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge.go | 20 | go_perf_layer_data_structure_choice_linked_list_for_cache_iteration / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_data_structure_choice_linked_list_for_cache_iteration |
| internal/pdf/merge.go | 46 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/merge.go | 27 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/merge.go | 17 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/merge.go | 15 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/merge.go | 16 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/merge.go | 70 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/merge.go | 26 | go_perf_layer_logging_overhead_log_fields_built_before_level_check / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_logging_overhead_log_fields_built_before_level_check |
| internal/pdf/merge.go | 16 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/merge.go | 15 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/merge.go | 17 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/merge.go | 15 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge.go | 33 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/merge.go | 55 | make_map_inside_hot_loop_same_shape / hot_path / stable | function MergePDFs recreates scratch maps inside a loop |
| internal/pdf/merge.go | 69 | make_map_inside_hot_loop_same_shape / hot_path / stable | function MergePDFs recreates scratch maps inside a loop |
| internal/pdf/merge.go | 27 | unnecessary_map_for_set_of_ints / performance / stable | function MergePDFs uses a map as a dense integer set |
| internal/pdf/merge.go | 16 | bytes_buffer_without_grow_known_bound / hot_path / stable | function MergePDFs uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/merge.go | 293 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function addParentRef matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/merge.go | 293 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function addParentRef matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/merge.go | 293 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function addParentRef matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/merge.go | 293 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function addParentRef matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge.go | 352 | nested_append_without_outer_capacity / hot_path / stable | function extractFormFieldsFromFile appends inside nested loops without visible preallocation |
| internal/pdf/merge.go | 327 | slice_append_without_prealloc_known_bound / hot_path / stable | function extractFormFieldsFromFile appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 352 | slice_append_without_prealloc_known_bound / hot_path / stable | function extractFormFieldsFromFile appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge.go | 327 | slice_grow_without_cap_hint / performance / stable | function extractFormFieldsFromFile appends to fields in a loop without a capacity hint |
| internal/pdf/merge.go | 352 | slice_grow_without_cap_hint / performance / stable | function extractFormFieldsFromFile appends to fields in a loop without a capacity hint |
| internal/pdf/merge.go | 343 | regexp_compile_in_hot_path / hot_path / stable | function extractFormFieldsFromFile compiles regular expressions inside a loop |
| internal/pdf/merge.go | 345 | regexp_compile_in_hot_path / hot_path / stable | function extractFormFieldsFromFile compiles regular expressions inside a loop |
| internal/pdf/merge.go | 347 | strconv_repeat_on_same_binding / hot_path / stable | function extractFormFieldsFromFile converts the same string input with strconv multiple times |
| internal/pdf/merge.go | 328 | map_growth_without_size_hint / hot_path / stable | function extractFormFieldsFromFile inserts into a map in a loop without a visible size hint |
| internal/pdf/merge.go | 313 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function extractFormFieldsFromFile matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge.go | 342 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function extractFormFieldsFromFile matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge.go | 310 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function extractFormFieldsFromFile matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge.go | 310 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function extractFormFieldsFromFile matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/merge.go | 319 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function extractFormFieldsFromFile matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge.go | 310 | unnecessary_map_for_set_of_ints / performance / stable | function extractFormFieldsFromFile uses a map as a dense integer set |
| internal/pdf/merge.go | 281 | strconv_repeat_on_same_binding / hot_path / stable | function replaceRefsOutsideStreams converts the same string input with strconv multiple times |
| internal/pdf/merge.go | 256 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function replaceRefsOutsideStreams matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge.go | 264 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function replaceRefsOutsideStreams matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/merge.go | 253 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function replaceRefsOutsideStreams matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/merge.go | 253 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function replaceRefsOutsideStreams matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/merge.go | 253 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function replaceRefsOutsideStreams matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge.go | 254 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function replaceRefsOutsideStreams matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/merge.go | 253 | bytes_buffer_without_grow_known_bound / hot_path / stable | function replaceRefsOutsideStreams uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/merge/annotations.go | 278 | unnecessary_map_for_set_of_ints / performance / stable | function CollectAllDependencies uses a map as a dense integer set |
| internal/pdf/merge/annotations.go | 71 | nested_append_without_outer_capacity / hot_path / stable | function ExtractAPDependencies appends inside nested loops without visible preallocation |
| internal/pdf/merge/annotations.go | 64 | slice_append_without_prealloc_known_bound / hot_path / stable | function ExtractAPDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 71 | slice_append_without_prealloc_known_bound / hot_path / stable | function ExtractAPDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 64 | slice_grow_without_cap_hint / performance / stable | function ExtractAPDependencies appends to deps in a loop without a capacity hint |
| internal/pdf/merge/annotations.go | 71 | slice_grow_without_cap_hint / performance / stable | function ExtractAPDependencies appends to deps in a loop without a capacity hint |
| internal/pdf/merge/annotations.go | 65 | map_growth_without_size_hint / hot_path / stable | function ExtractAPDependencies inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/annotations.go | 58 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function ExtractAPDependencies matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/annotations.go | 49 | go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans / performance / stable | function ExtractAPDependencies matches performance-layer rule go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans |
| internal/pdf/merge/annotations.go | 50 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function ExtractAPDependencies matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/annotations.go | 50 | unnecessary_map_for_set_of_ints / performance / stable | function ExtractAPDependencies uses a map as a dense integer set |
| internal/pdf/merge/annotations.go | 22 | slice_append_without_prealloc_known_bound / hot_path / stable | function ExtractAnnotationsFromPage appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 36 | slice_append_without_prealloc_known_bound / hot_path / stable | function ExtractAnnotationsFromPage appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/annotations.go | 22 | slice_grow_without_cap_hint / performance / stable | function ExtractAnnotationsFromPage appends to annots in a loop without a capacity hint |
| internal/pdf/merge/annotations.go | 36 | slice_grow_without_cap_hint / performance / stable | function ExtractAnnotationsFromPage appends to annots in a loop without a capacity hint |
| internal/pdf/merge/annotations.go | 35 | strconv_repeat_on_same_binding / hot_path / stable | function ExtractAnnotationsFromPage converts the same string input with strconv multiple times |
| internal/pdf/merge/annotations.go | 19 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function ExtractAnnotationsFromPage matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/annotations.go | 20 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function ExtractAnnotationsFromPage matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/annotations.go | 22 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function ExtractAnnotationsFromPage matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/annotations.go | 32 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function ExtractAnnotationsFromPage matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/merge/annotations.go | 105 | map_growth_without_size_hint / hot_path / stable | function ExtractFormFields inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/annotations.go | 91 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function ExtractFormFields matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/annotations.go | 87 | unnecessary_map_for_set_of_ints / performance / stable | function ExtractFormFields uses a map as a dense integer set |
| internal/pdf/merge/annotations.go | 203 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function addFieldRecursive matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/annotations.go | 199 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function addFieldRecursive matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/annotations.go | 310 | regexp_compile_in_hot_path / hot_path / stable | function collectDepsRecursive compiles regular expressions inside a loop |
| internal/pdf/merge/annotations.go | 295 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function collectDepsRecursive matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/annotations.go | 305 | go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans / performance / stable | function collectDepsRecursive matches performance-layer rule go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans |
| internal/pdf/merge/annotations.go | 299 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function collectDepsRecursive matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/annotations.go | 305 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function collectDepsRecursive matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/annotations.go | 185 | strconv_repeat_on_same_binding / hot_path / stable | function extractFieldsArray converts the same string input with strconv multiple times |
| internal/pdf/merge/merger.go | 38 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 77 | slice_append_without_prealloc_known_bound / hot_path / stable | function MergePDFs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 77 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to appendedObjects in a loop without a capacity hint |
| internal/pdf/merge/merger.go | 38 | slice_grow_without_cap_hint / performance / stable | function MergePDFs appends to fileContexts in a loop without a capacity hint |
| internal/pdf/merge/merger.go | 14 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/merger.go | 14 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/merge/merger.go | 15 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/merge/merger.go | 27 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/merge/merger.go | 38 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/merger.go | 46 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function MergePDFs matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/merge/merger.go | 300 | nested_append_without_outer_capacity / hot_path / stable | function collectObjectsWithDependencies appends inside nested loops without visible preallocation |
| internal/pdf/merge/merger.go | 290 | slice_append_without_prealloc_known_bound / hot_path / stable | function collectObjectsWithDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 300 | slice_append_without_prealloc_known_bound / hot_path / stable | function collectObjectsWithDependencies appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 290 | slice_grow_without_cap_hint / performance / stable | function collectObjectsWithDependencies appends to result in a loop without a capacity hint |
| internal/pdf/merge/merger.go | 300 | slice_grow_without_cap_hint / performance / stable | function collectObjectsWithDependencies appends to result in a loop without a capacity hint |
| internal/pdf/merge/merger.go | 276 | map_growth_without_size_hint / hot_path / stable | function collectObjectsWithDependencies inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/merger.go | 269 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function collectObjectsWithDependencies matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/merger.go | 264 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function collectObjectsWithDependencies matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/merger.go | 245 | slice_append_without_prealloc_known_bound / hot_path / stable | function extractKidsRecursive appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 251 | slice_append_without_prealloc_known_bound / hot_path / stable | function extractKidsRecursive appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 254 | slice_append_without_prealloc_known_bound / hot_path / stable | function extractKidsRecursive appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/merger.go | 245 | slice_grow_without_cap_hint / performance / stable | function extractKidsRecursive appends to pages in a loop without a capacity hint |
| internal/pdf/merge/merger.go | 251 | slice_grow_without_cap_hint / performance / stable | function extractKidsRecursive appends to pages in a loop without a capacity hint |
| internal/pdf/merge/merger.go | 254 | slice_grow_without_cap_hint / performance / stable | function extractKidsRecursive appends to pages in a loop without a capacity hint |
| internal/pdf/merge/merger.go | 251 | go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set / performance / stable | function extractKidsRecursive matches performance-layer rule go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set |
| internal/pdf/merge/merger.go | 241 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function extractKidsRecursive matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/merger.go | 245 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function extractKidsRecursive matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/merger.go | 243 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function extractKidsRecursive matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/merge/merger.go | 228 | go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set / performance / stable | function extractPagesFromTree matches performance-layer rule go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set |
| internal/pdf/merge/merger.go | 209 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function extractPagesFromTree matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/merge/merger.go | 181 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function findCatalogAndPages matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/merge/merger.go | 121 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function parseFile matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/merge/merger.go | 153 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function parseFile matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/merge/merger.go | 153 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function parseFile matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/merger.go | 374 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function updateParentRef matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/merge/merger.go | 374 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function updateParentRef matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/merge/merger.go | 374 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function updateParentRef matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/merge/merger.go | 372 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function updateParentRef matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/merge/merger.go | 374 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function updateParentRef matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge/merger.go | 321 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function writeCatalog matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/merger.go | 356 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function writeObject matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/merge/merger.go | 340 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function writePages matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/merger.go | 417 | nested_append_without_outer_capacity / hot_path / stable | function writeXRefAndTrailer appends inside nested loops without visible preallocation |
| internal/pdf/merge/merger.go | 395 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function writeXRefAndTrailer matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/merger.go | 402 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function writeXRefAndTrailer matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/merger.go | 415 | strconv_formatint_int64_cast_itoa / performance / stable | function writeXRefAndTrailer uses `strconv.FormatInt(int64(v), 10)` instead of `strconv.Itoa(v)`. |
| internal/pdf/merge/parser.go | 123 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function FindEndObj matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge/parser.go | 109 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function FindEndObj matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge/parser.go | 79 | slice_grow_without_cap_hint / performance / stable | function FindObjectBoundaries appends to results in a loop without a capacity hint |
| internal/pdf/merge/parser.go | 58 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function FindObjectBoundaries matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/merge/parser.go | 60 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function FindObjectBoundaries matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge/parser.go | 54 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function FindObjectBoundaries matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/merge/parser.go | 79 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function FindObjectBoundaries matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/merge/parser.go | 485 | slice_grow_without_cap_hint / performance / stable | function ParseObjectStream appends to entries in a loop without a capacity hint |
| internal/pdf/merge/parser.go | 501 | map_growth_without_size_hint / hot_path / stable | function ParseObjectStream inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/parser.go | 440 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function ParseObjectStream matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/parser.go | 440 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ParseObjectStream matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/merge/parser.go | 444 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function ParseObjectStream matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/merge/parser.go | 349 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function ReplaceRefsOutsideStreams matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge/parser.go | 309 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function ReplaceRefsOutsideStreams matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/merge/parser.go | 300 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ReplaceRefsOutsideStreams matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/merge/parser.go | 300 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function ReplaceRefsOutsideStreams matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/merge/parser.go | 300 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function ReplaceRefsOutsideStreams matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge/parser.go | 299 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function ReplaceRefsOutsideStreams matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/merge/parser.go | 300 | bytes_buffer_without_grow_known_bound / hot_path / stable | function ReplaceRefsOutsideStreams uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/merge/parser.go | 556 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function decompressFlate matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/merge/parser.go | 526 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function extractAndDecompressStream matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge/parser.go | 511 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function extractAndDecompressStream matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/merge/parser.go | 526 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function extractAndDecompressStream matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/merge/split.go | 62 | slice_append_without_prealloc_known_bound / hot_path / stable | function ParsePageSpec appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 62 | slice_grow_without_cap_hint / performance / stable | function ParsePageSpec appends to pages in a loop without a capacity hint |
| internal/pdf/merge/split.go | 47 | map_growth_without_size_hint / hot_path / stable | function ParsePageSpec inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/split.go | 24 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function ParsePageSpec matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/split.go | 24 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function ParsePageSpec matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/merge/split.go | 33 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ParsePageSpec matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/merge/split.go | 24 | unnecessary_map_for_set_of_ints / performance / stable | function ParsePageSpec uses a map as a dense integer set |
| internal/pdf/merge/split.go | 101 | slice_append_without_prealloc_known_bound / hot_path / stable | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 110 | slice_append_without_prealloc_known_bound / hot_path / stable | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 124 | slice_append_without_prealloc_known_bound / hot_path / stable | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 137 | slice_append_without_prealloc_known_bound / hot_path / stable | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 149 | slice_append_without_prealloc_known_bound / hot_path / stable | function SplitPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 137 | three_index_slice_for_append_safety / performance / stable | function SplitPDF appends to a subslice without capping its capacity |
| internal/pdf/merge/split.go | 137 | slice_grow_without_cap_hint / performance / stable | function SplitPDF appends to groups in a loop without a capacity hint |
| internal/pdf/merge/split.go | 124 | slice_grow_without_cap_hint / performance / stable | function SplitPDF appends to orderedPages in a loop without a capacity hint |
| internal/pdf/merge/split.go | 149 | slice_grow_without_cap_hint / performance / stable | function SplitPDF appends to outputs in a loop without a capacity hint |
| internal/pdf/merge/split.go | 101 | slice_grow_without_cap_hint / performance / stable | function SplitPDF appends to requestedObjNums in a loop without a capacity hint |
| internal/pdf/merge/split.go | 110 | slice_grow_without_cap_hint / performance / stable | function SplitPDF appends to requestedObjNums in a loop without a capacity hint |
| internal/pdf/merge/split.go | 125 | map_growth_without_size_hint / hot_path / stable | function SplitPDF inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/split.go | 76 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/split.go | 69 | go_perf_layer_algorithmic_complexity_sort_before_linear_dedup / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_algorithmic_complexity_sort_before_linear_dedup |
| internal/pdf/merge/split.go | 72 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/merge/split.go | 69 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/merge/split.go | 97 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/split.go | 120 | go_perf_layer_data_structure_choice_linked_list_for_cache_iteration / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_data_structure_choice_linked_list_for_cache_iteration |
| internal/pdf/merge/split.go | 77 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/merge/split.go | 72 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/merge/split.go | 83 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/merge/split.go | 94 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function SplitPDF matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/merge/split.go | 120 | unnecessary_map_for_set_of_ints / performance / stable | function SplitPDF uses a map as a dense integer set |
| internal/pdf/merge/split.go | 163 | append_then_trim_each_iteration / hot_path / stable | function buildPDFFromPageObjs appends and then reslices in a loop |
| internal/pdf/merge/split.go | 163 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 178 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 193 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 203 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 236 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 246 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildPDFFromPageObjs appends to a slice inside a range loop without visible preallocation |
| internal/pdf/merge/split.go | 246 | slice_grow_without_cap_hint / performance / stable | function buildPDFFromPageObjs appends to mergedFields in a loop without a capacity hint |
| internal/pdf/merge/split.go | 236 | slice_grow_without_cap_hint / performance / stable | function buildPDFFromPageObjs appends to mergedPages in a loop without a capacity hint |
| internal/pdf/merge/split.go | 203 | slice_grow_without_cap_hint / performance / stable | function buildPDFFromPageObjs appends to objs in a loop without a capacity hint |
| internal/pdf/merge/split.go | 163 | slice_grow_without_cap_hint / performance / stable | function buildPDFFromPageObjs appends to stack in a loop without a capacity hint |
| internal/pdf/merge/split.go | 178 | slice_grow_without_cap_hint / performance / stable | function buildPDFFromPageObjs appends to stack in a loop without a capacity hint |
| internal/pdf/merge/split.go | 193 | slice_grow_without_cap_hint / performance / stable | function buildPDFFromPageObjs appends to stack in a loop without a capacity hint |
| internal/pdf/merge/split.go | 162 | map_growth_without_size_hint / hot_path / stable | function buildPDFFromPageObjs inserts into a map in a loop without a visible size hint |
| internal/pdf/merge/split.go | 163 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/merge/split.go | 166 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/merge/split.go | 159 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/merge/split.go | 159 | go_perf_layer_data_structure_choice_linked_list_for_cache_iteration / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_data_structure_choice_linked_list_for_cache_iteration |
| internal/pdf/merge/split.go | 159 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/merge/split.go | 159 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/pdf/merge/split.go | 211 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/merge/split.go | 158 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function buildPDFFromPageObjs matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/merge/split.go | 222 | unnecessary_map_for_set_of_ints / performance / stable | function buildPDFFromPageObjs uses a map as a dense integer set |
| internal/pdf/merge/types.go | 85 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function NewFileContext matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/merge/types.go | 54 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function NewMergeContext matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/merge/types.go | 56 | unnecessary_map_for_set_of_ints / performance / stable | function NewMergeContext uses a map as a dense integer set |
| internal/pdf/metadata.go | 269 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateOutputIntent matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/metadata.go | 269 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function GenerateOutputIntent matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/metadata.go | 269 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function GenerateOutputIntent matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/metadata.go | 284 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateOutputIntent matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/metadata.go | 77 | string_builder_write_string_vs_plus / performance / stable | function GenerateXMPMetadata concatenates strings before WriteString |
| internal/pdf/metadata.go | 76 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateXMPMetadata matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/metadata.go | 245 | builder_write_string_single_byte_literal / performance / stable | function GenerateXMPMetadata uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/metadata.go | 76 | strings_builder_without_grow_known_bound / hot_path / stable | function GenerateXMPMetadata uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/metadata.go | 29 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function NewPDFAHandler matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/outline.go | 63 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function BuildOutlines matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/outline.go | 478 | uuid_hash_formatting_only_for_logs / hot_path / stable | function GetNamedDestinations formats identifiers inside a loop only for logging |
| internal/pdf/outline.go | 478 | fmt_hot_path / performance / stable | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 480 | fmt_hot_path / performance / stable | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 487 | fmt_hot_path / performance / stable | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 491 | fmt_hot_path / performance / stable | function GetNamedDestinations formats strings with fmt inside a loop |
| internal/pdf/outline.go | 463 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function GetNamedDestinations matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/outline.go | 443 | go_perf_layer_data_structure_choice_slice_queue_pop_front / performance / stable | function GetNamedDestinations matches performance-layer rule go_perf_layer_data_structure_choice_slice_queue_pop_front |
| internal/pdf/outline.go | 443 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function GetNamedDestinations matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/outline.go | 439 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function GetNamedDestinations matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/outline.go | 478 | hex_encode_to_string_in_loop / performance / stable | function GetNamedDestinations uses `hex.EncodeToString(...)` inside loops. |
| internal/pdf/outline.go | 440 | builder_write_string_single_byte_literal / performance / stable | function GetNamedDestinations uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/outline.go | 469 | builder_write_string_single_byte_literal / performance / stable | function GetNamedDestinations uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/outline.go | 495 | builder_write_string_single_byte_literal / performance / stable | function GetNamedDestinations uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/outline.go | 439 | strings_builder_without_grow_known_bound / hot_path / stable | function GetNamedDestinations uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/outline.go | 84 | go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set / performance / stable | function allocateOutlineIDs matches performance-layer rule go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set |
| internal/pdf/outline.go | 84 | go_perf_layer_algorithmic_complexity_sort_before_linear_dedup / performance / stable | function allocateOutlineIDs matches performance-layer rule go_perf_layer_algorithmic_complexity_sort_before_linear_dedup |
| internal/pdf/outline.go | 86 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function allocateOutlineIDs matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/outline.go | 97 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function allocateOutlineIDs matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/outline.go | 86 | go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item / performance / stable | function allocateOutlineIDs matches performance-layer rule go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item |
| internal/pdf/outline.go | 200 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function allocateOutlineIDs matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/outline.go | 216 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function buildTreeRelationships matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/outline.go | 264 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function calculateCounts matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/outline.go | 276 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function countDescendants matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/outline.go | 414 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function escapeTextUnicode matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/outline.go | 409 | strings_builder_without_grow_known_bound / hot_path / stable | function escapeTextUnicode uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/outline.go | 345 | slice_append_without_prealloc_known_bound / hot_path / stable | function generateOutlineObjects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/outline.go | 348 | slice_append_without_prealloc_known_bound / hot_path / stable | function generateOutlineObjects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/outline.go | 353 | slice_append_without_prealloc_known_bound / hot_path / stable | function generateOutlineObjects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/outline.go | 345 | slice_grow_without_cap_hint / performance / stable | function generateOutlineObjects appends to titleBytes in a loop without a capacity hint |
| internal/pdf/outline.go | 348 | slice_grow_without_cap_hint / performance / stable | function generateOutlineObjects appends to titleBytes in a loop without a capacity hint |
| internal/pdf/outline.go | 353 | slice_grow_without_cap_hint / performance / stable | function generateOutlineObjects appends to titleBytes in a loop without a capacity hint |
| internal/pdf/outline.go | 362 | uuid_hash_formatting_only_for_logs / hot_path / stable | function generateOutlineObjects formats identifiers inside a loop only for logging |
| internal/pdf/outline.go | 362 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 364 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 367 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 372 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 375 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 380 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 383 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 386 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 387 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 388 | fmt_hot_path / performance / stable | function generateOutlineObjects formats strings with fmt inside a loop |
| internal/pdf/outline.go | 296 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function generateOutlineObjects matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/outline.go | 295 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function generateOutlineObjects matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/outline.go | 316 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function generateOutlineObjects matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/outline.go | 345 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function generateOutlineObjects matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/outline.go | 328 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function generateOutlineObjects recreates a strings.Builder inside a loop |
| internal/pdf/outline.go | 327 | filter_then_count_then_iterate / hot_path / stable | function generateOutlineObjects traverses the same collection multiple times for filter, count, and process |
| internal/pdf/outline.go | 362 | hex_encode_to_string_in_loop / performance / stable | function generateOutlineObjects uses `hex.EncodeToString(...)` inside loops. |
| internal/pdf/outline.go | 316 | strings_builder_without_grow_known_bound / hot_path / stable | function generateOutlineObjects uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/pagemanager.go | 88 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function AddLinkAnnotation matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/pagemanager.go | 101 | sprintf_for_simple_string_format / performance / stable | function AddLinkAnnotation uses fmt.Sprintf with only %s verbs |
| internal/pdf/pagemanager.go | 146 | go_perf_layer_lazy_loading_eager_connect_to_all_backends / performance / stable | function AddLinkStructureElement matches performance-layer rule go_perf_layer_lazy_loading_eager_connect_to_all_backends |
| internal/pdf/pagemanager.go | 51 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function NewPageManager matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/pagemanager.go | 42 | public_bool_parameter_api / idioms / stable | public function NewPageManager uses boolean parameter to control behavior |
| internal/pdf/pdf.go | 104 | fmt_errorf_without_wrap_verb / performance / stable | function ConvertHTMLToImage uses %v instead of %w for error wrapping |
| internal/pdf/pdf.go | 104 | error_wrapping_misuse / errors / stable | function ConvertHTMLToImage uses fmt.Errorf without %w while referencing err |
| internal/pdf/pdf.go | 63 | fmt_errorf_without_wrap_verb / performance / stable | function ConvertHTMLToPDF uses %v instead of %w for error wrapping |
| internal/pdf/pdf.go | 63 | error_wrapping_misuse / errors / stable | function ConvertHTMLToPDF uses fmt.Errorf without %w while referencing err |
| internal/pdf/pdfa.go | 42 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function ConvertPDFDateToXMP matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/pdfa.go | 42 | sprintf_for_simple_string_format / performance / stable | function ConvertPDFDateToXMP uses fmt.Sprintf with only %s verbs |
| internal/pdf/pdfa.go | 364 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateGrayICCProfileObject matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/pdfa.go | 348 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateGrayICCProfileObject matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/pdfa.go | 333 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateICCProfileObject matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/pdfa.go | 312 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function GenerateICCProfileObject matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/pdfa.go | 317 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function GenerateICCProfileObject matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/pdfa.go | 485 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateOutputIntentObject matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/pdfa.go | 131 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GenerateXMPMetadataObject matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/pdfa.go | 375 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function buildGrayICCProfile matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/pdfa.go | 403 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function buildGrayICCProfile matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/pdfa.go | 143 | go_perf_layer_algorithmic_complexity_sort_before_linear_dedup / performance / stable | function buildSRGBICCProfile matches performance-layer rule go_perf_layer_algorithmic_complexity_sort_before_linear_dedup |
| internal/pdf/pdfa.go | 146 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function buildSRGBICCProfile matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/pdfa.go | 146 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function buildSRGBICCProfile matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/pdf/pdfa.go | 146 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function buildSRGBICCProfile matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/pdfa.go | 182 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function buildSRGBICCProfile matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/redact/encryption_inhouse.go | 51 | errors_new_for_static_sentinel / performance / stable | function decryptEncryptedPDFBytes calls errors.New multiple times with static strings |
| internal/pdf/redact/encryption_inhouse.go | 63 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function decryptEncryptedPDFBytes matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/encryption_inhouse.go | 50 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function decryptEncryptedPDFBytes matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/encryption_inhouse.go | 47 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function decryptEncryptedPDFBytes matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/encryption_inhouse.go | 319 | three_index_slice_for_append_safety / performance / stable | function decryptObjectStreams appends to a subslice without capping its capacity |
| internal/pdf/redact/encryption_inhouse.go | 318 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function decryptObjectStreams matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/redact/encryption_inhouse.go | 210 | timing_attack_on_token_comparison / security / stable | function deriveAndValidateUserKey compares tokens with non-constant-time equality |
| internal/pdf/redact/encryption_inhouse.go | 232 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function deriveFileKey matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/encryption_inhouse.go | 232 | weak_hash_for_integrity / security / stable | function deriveFileKey relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/redact/encryption_inhouse.go | 243 | weak_hash_for_integrity / security / stable | function deriveFileKey relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/redact/encryption_inhouse.go | 243 | md5_sum_in_loop / performance / stable | function deriveFileKey uses `md5.Sum(...)` inside loops. |
| internal/pdf/redact/encryption_inhouse.go | 232 | weak_crypto / security / stable | function deriveFileKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 243 | weak_crypto / security / stable | function deriveFileKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 332 | weak_hash_for_integrity / security / stable | function deriveObjectKey relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/redact/encryption_inhouse.go | 332 | weak_crypto / security / stable | function deriveObjectKey uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 292 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function deriveUserPasswordFromOwner matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/encryption_inhouse.go | 283 | weak_hash_for_integrity / security / stable | function deriveUserPasswordFromOwner relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/redact/encryption_inhouse.go | 287 | weak_hash_for_integrity / security / stable | function deriveUserPasswordFromOwner relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/redact/encryption_inhouse.go | 287 | md5_sum_in_loop / performance / stable | function deriveUserPasswordFromOwner uses `md5.Sum(...)` inside loops. |
| internal/pdf/redact/encryption_inhouse.go | 283 | weak_crypto / security / stable | function deriveUserPasswordFromOwner uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 287 | weak_crypto / security / stable | function deriveUserPasswordFromOwner uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/encryption_inhouse.go | 25 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function padPassword matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/redact/encryption_inhouse.go | 98 | errors_new_for_static_sentinel / performance / stable | function parseEncryptRefAndID calls errors.New multiple times with static strings |
| internal/pdf/redact/encryption_inhouse.go | 98 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function parseEncryptRefAndID matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/encryption_inhouse.go | 96 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function parseEncryptRefAndID matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/encryption_inhouse.go | 124 | strings_replace_all_for_single_char / performance / stable | function parseFirstID uses strings.ReplaceAll for single character replacement |
| internal/pdf/redact/encryption_inhouse.go | 181 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function parseHexOrLiteralField matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/redact/encryption_inhouse.go | 179 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function parseHexOrLiteralField matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/redact/encryption_inhouse.go | 181 | strings_replace_all_for_single_char / performance / stable | function parseHexOrLiteralField uses strings.ReplaceAll for single character replacement |
| internal/pdf/redact/encryption_inhouse.go | 136 | errors_new_for_static_sentinel / performance / stable | function parseStandardEncryptDict calls errors.New multiple times with static strings |
| internal/pdf/redact/encryption_inhouse.go | 136 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function parseStandardEncryptDict matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/redact/encryption_inhouse.go | 139 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function parseStandardEncryptDict matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/redact/encryption_inhouse.go | 358 | weak_crypto / security / stable | function rc4Crypt uses weak cryptographic primitive crypto/rc4 |
| internal/pdf/redact/encryption_inhouse.go | 255 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function validateUserPassword matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/encryption_inhouse.go | 255 | weak_hash_for_integrity / security / stable | function validateUserPassword relies on MD5/SHA-1 for integrity-sensitive hashing |
| internal/pdf/redact/encryption_inhouse.go | 255 | weak_crypto / security / stable | function validateUserPassword uses weak cryptographic primitive crypto/md5 |
| internal/pdf/redact/helpers.go | 105 | regexp_compile_in_hot_path / hot_path / stable | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/redact/helpers.go | 137 | regexp_compile_in_hot_path / hot_path / stable | function parseXRefStreams compiles regular expressions inside a loop |
| internal/pdf/redact/helpers.go | 102 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/redact/helpers.go | 101 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/helpers.go | 102 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function parseXRefStreams matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/redact/helpers.go | 22 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function trailerHasEncrypt matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/redact/helpers.go | 47 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function tryFlateDecompress matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/redact/helpers.go | 31 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function tryZlibDecompress matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/redact/ocr_adapter.go | 165 | slice_grow_without_cap_hint / performance / stable | function ExtractWords appends to words in a loop without a capacity hint |
| internal/pdf/redact/ocr_adapter.go | 79 | errors_new_for_static_sentinel / performance / stable | function ExtractWords calls errors.New multiple times with static strings |
| internal/pdf/redact/ocr_adapter.go | 110 | go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans |
| internal/pdf/redact/ocr_adapter.go | 79 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/ocr_adapter.go | 98 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/redact/ocr_adapter.go | 98 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/redact/ocr_adapter.go | 98 | go_perf_layer_io_operations_readall_on_known_large_file / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_io_operations_readall_on_known_large_file |
| internal/pdf/redact/ocr_adapter.go | 98 | go_perf_layer_io_operations_scanner_used_for_large_token_stream / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_io_operations_scanner_used_for_large_token_stream |
| internal/pdf/redact/ocr_adapter.go | 100 | go_perf_layer_io_operations_temporary_file_for_stream_transform / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_io_operations_temporary_file_for_stream_transform |
| internal/pdf/redact/ocr_adapter.go | 79 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ExtractWords matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/ocr_adapter.go | 114 | missing_context / context / stable | function ExtractWords performs context-aware work without accepting context.Context |
| internal/pdf/redact/ocr_adapter.go | 129 | missing_context / context / stable | function ExtractWords performs context-aware work without accepting context.Context |
| internal/pdf/redact/ocr_adapter.go | 116 | fmt_errorf_without_wrap_verb / performance / stable | function ExtractWords uses %v instead of %w for error wrapping |
| internal/pdf/redact/ocr_adapter.go | 132 | fmt_errorf_without_wrap_verb / performance / stable | function ExtractWords uses %v instead of %w for error wrapping |
| internal/pdf/redact/ocr_adapter.go | 116 | error_wrapping_misuse / errors / stable | function ExtractWords uses fmt.Errorf without %w while referencing err |
| internal/pdf/redact/ocr_adapter.go | 132 | error_wrapping_misuse / errors / stable | function ExtractWords uses fmt.Errorf without %w while referencing err |
| internal/pdf/redact/ocr_adapter.go | 36 | repeated_string_trim_normalize / performance / stable | function getOCRProvider chains multiple string normalization operations |
| internal/pdf/redact/ocr_adapter.go | 36 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function getOCRProvider matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/ocr_adapter.go | 36 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function getOCRProvider matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/redact/ocr_adapter.go | 63 | slice_append_without_prealloc_known_bound / hot_path / stable | function runOCRSearch appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/ocr_adapter.go | 63 | slice_grow_without_cap_hint / performance / stable | function runOCRSearch appends to rects in a loop without a capacity hint |
| internal/pdf/redact/ocr_adapter.go | 58 | repeated_string_trim_normalize / performance / stable | function runOCRSearch chains multiple string normalization operations |
| internal/pdf/redact/ocr_adapter.go | 63 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function runOCRSearch matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/ocr_adapter.go | 44 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function runOCRSearch matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/redact/ocr_adapter.go | 29 | single_impl_interface / idioms / stable | interface OCRProvider currently has one obvious repository-local implementation |
| internal/pdf/redact/pdf_utils.go | 735 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function appendStreamToPage matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/redact/pdf_utils.go | 733 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function appendStreamToPage matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/pdf_utils.go | 735 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function appendStreamToPage matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/redact/pdf_utils.go | 735 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function appendStreamToPage matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/redact/pdf_utils.go | 759 | sprintf_for_simple_string_format / performance / stable | function appendStreamToPage uses fmt.Sprintf with only %s verbs |
| internal/pdf/redact/pdf_utils.go | 763 | sprintf_for_simple_string_format / performance / stable | function appendStreamToPage uses fmt.Sprintf with only %s verbs |
| internal/pdf/redact/pdf_utils.go | 385 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function appendXObjectContentRecursive matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/pdf_utils.go | 385 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function appendXObjectContentRecursive matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/pdf_utils.go | 28 | regexp_compile_in_hot_path / hot_path / stable | function buildObjectMap compiles regular expressions inside a loop |
| internal/pdf/redact/pdf_utils.go | 38 | regexp_compile_in_hot_path / hot_path / stable | function buildObjectMap compiles regular expressions inside a loop |
| internal/pdf/redact/pdf_utils.go | 68 | map_growth_without_size_hint / hot_path / stable | function buildObjectMap inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/pdf_utils.go | 18 | go_perf_layer_algorithmic_complexity_sort_before_linear_dedup / performance / stable | function buildObjectMap matches performance-layer rule go_perf_layer_algorithmic_complexity_sort_before_linear_dedup |
| internal/pdf/redact/pdf_utils.go | 19 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function buildObjectMap matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/pdf_utils.go | 19 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function buildObjectMap matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/pdf_utils.go | 19 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function buildObjectMap matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/pdf_utils.go | 24 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function buildObjectMap matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/pdf_utils.go | 19 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function buildObjectMap matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/redact/pdf_utils.go | 684 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function decodePDFHexLiteral matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/pdf_utils.go | 642 | range_over_string_by_index / performance / stable | function decodePDFLiteral iterates string by byte index |
| internal/pdf/redact/pdf_utils.go | 641 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function decodePDFLiteral matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/pdf_utils.go | 641 | bytes_buffer_without_grow_known_bound / hot_path / stable | function decodePDFLiteral uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 576 | strings_builder_without_grow_known_bound / hot_path / stable | function decodeTJArray uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 425 | string_concat_in_loop / performance / stable | function extractKidsRefs concatenates strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 425 | byte_string_conversion_in_loop / hot_path / stable | function extractKidsRefs converts between bytes and strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 125 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function extractMediaBox matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/pdf_utils.go | 215 | slice_append_without_prealloc_known_bound / hot_path / stable | function extractPageContent appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/pdf_utils.go | 215 | slice_grow_without_cap_hint / performance / stable | function extractPageContent appends to contentKeys in a loop without a capacity hint |
| internal/pdf/redact/pdf_utils.go | 215 | string_concat_in_loop / performance / stable | function extractPageContent concatenates strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 215 | byte_string_conversion_in_loop / hot_path / stable | function extractPageContent converts between bytes and strings inside a loop |
| internal/pdf/redact/pdf_utils.go | 201 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/pdf_utils.go | 198 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/redact/pdf_utils.go | 214 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/pdf_utils.go | 206 | go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_data_structure_choice_map_string_bool_for_membership |
| internal/pdf/redact/pdf_utils.go | 209 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/pdf_utils.go | 219 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/pdf_utils.go | 220 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/pdf_utils.go | 200 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function extractPageContent matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/pdf_utils.go | 219 | bytes_buffer_without_grow_known_bound / hot_path / stable | function extractPageContent uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 341 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function extractResourcesBody matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/pdf_utils.go | 568 | strings_hasprefix_then_trimprefix / performance / stable | function extractTextFromOperator checks HasPrefix then TrimPrefix (use CutPrefix) |
| internal/pdf/redact/pdf_utils.go | 560 | strings_hassuffix_then_trimsuffix / performance / stable | function extractTextFromOperator checks HasSuffix then TrimSuffix (use CutSuffix) |
| internal/pdf/redact/pdf_utils.go | 568 | strings_hassuffix_then_trimsuffix / performance / stable | function extractTextFromOperator checks HasSuffix then TrimSuffix (use CutSuffix) |
| internal/pdf/redact/pdf_utils.go | 145 | errors_new_for_static_sentinel / performance / stable | function findPageObject calls errors.New multiple times with static strings |
| internal/pdf/redact/pdf_utils.go | 159 | go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set / performance / stable | function findPageObject matches performance-layer rule go_perf_layer_algorithmic_complexity_recursive_graph_walk_without_seen_set |
| internal/pdf/redact/pdf_utils.go | 148 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function findPageObject matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/pdf_utils.go | 145 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function findPageObject matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/pdf_utils.go | 143 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function findPageObject matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/pdf_utils.go | 148 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function findPageObject matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/pdf_utils.go | 321 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function findPageResources matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/pdf_utils.go | 252 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function locateStreamSegment matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/redact/pdf_utils.go | 251 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function locateStreamSegment matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/pdf_utils.go | 527 | slice_append_without_prealloc_known_bound / hot_path / stable | function parseTextOperators appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/pdf_utils.go | 527 | slice_grow_without_cap_hint / performance / stable | function parseTextOperators appends to positions in a loop without a capacity hint |
| internal/pdf/redact/pdf_utils.go | 506 | strconv_repeat_on_same_binding / hot_path / stable | function parseTextOperators converts the same string input with strconv multiple times |
| internal/pdf/redact/pdf_utils.go | 464 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function parseTextOperators matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/redact/pdf_utils.go | 487 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function parseTextOperators matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/redact/pdf_utils.go | 527 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function parseTextOperators matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/pdf_utils.go | 467 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function parseTextOperators matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/redact/pdf_utils.go | 881 | slice_append_without_prealloc_known_bound / hot_path / stable | function rebuildPDF appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/pdf_utils.go | 881 | slice_grow_without_cap_hint / performance / stable | function rebuildPDF appends to block in a loop without a capacity hint |
| internal/pdf/redact/pdf_utils.go | 839 | map_growth_without_size_hint / hot_path / stable | function rebuildPDF inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/pdf_utils.go | 779 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function rebuildPDF matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/pdf_utils.go | 791 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function rebuildPDF matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/pdf_utils.go | 786 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function rebuildPDF matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/pdf_utils.go | 778 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function rebuildPDF matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/pdf_utils.go | 778 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function rebuildPDF matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/redact/pdf_utils.go | 791 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function rebuildPDF matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/pdf_utils.go | 824 | bytes_buffer_without_grow_known_bound / hot_path / stable | function rebuildPDF uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/redact/pdf_utils.go | 363 | map_growth_without_size_hint / hot_path / stable | function resolveUsedXObjectRefs inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/pdf_utils.go | 360 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function resolveUsedXObjectRefs matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/pdf_utils.go | 360 | go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | function resolveUsedXObjectRefs matches performance-layer rule go_perf_layer_data_structure_choice_map_string_bool_for_membership |
| internal/pdf/redact/pdf_utils.go | 86 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function traversePages matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/pdf_utils.go | 104 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function traversePages matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/pdf_utils.go | 86 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function traversePages matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/pdf_utils.go | 4 | helper_or_utils_package_contains_domain_logic / architecture / stable | generic helper or utils package imports domain-specific layers |
| internal/pdf/redact/redactor.go | 138 | allocation_churn_in_loop / performance / stable | function AnalyzePageCapabilities allocates new objects inside a loop |
| internal/pdf/redact/redactor.go | 138 | likely_n_squared_allocation / performance / experimental | function AnalyzePageCapabilities appears to allocate inside a nested loop |
| internal/pdf/redact/redactor.go | 105 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/redactor.go | 145 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/redact/redactor.go | 128 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/redactor.go | 125 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/redact/redactor.go | 104 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/redactor.go | 106 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/redactor.go | 128 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/redactor.go | 106 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function AnalyzePageCapabilities matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/redact/redactor.go | 138 | make_slice_inside_hot_loop_same_shape / hot_path / stable | function AnalyzePageCapabilities recreates scratch slices inside a loop |
| internal/pdf/redact/redactor.go | 182 | errors_new_for_static_sentinel / performance / stable | function ApplyRedactionsAdvancedWithReport calls errors.New multiple times with static strings |
| internal/pdf/redact/redactor.go | 190 | repeated_string_trim_normalize / performance / stable | function ApplyRedactionsAdvancedWithReport chains multiple string normalization operations |
| internal/pdf/redact/redactor.go | 181 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/redactor.go | 200 | go_perf_layer_caching_cache_miss_does_duplicate_work / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_caching_cache_miss_does_duplicate_work |
| internal/pdf/redact/redactor.go | 190 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/redact/redactor.go | 180 | go_perf_layer_caching_per_request_config_cache_rebuild / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_caching_per_request_config_cache_rebuild |
| internal/pdf/redact/redactor.go | 182 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/redactor.go | 215 | go_perf_layer_error_handling_cost_multierror_append_for_success_path / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_error_handling_cost_multierror_append_for_success_path |
| internal/pdf/redact/redactor.go | 182 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/redact/redactor.go | 181 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/redactor.go | 253 | go_perf_layer_resource_pooling_rate_limiter_per_request / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_resource_pooling_rate_limiter_per_request |
| internal/pdf/redact/redactor.go | 180 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/redact/redactor.go | 181 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function ApplyRedactionsAdvancedWithReport matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/redact/redactor.go | 56 | errors_new_for_static_sentinel / performance / stable | function GetPageInfo calls errors.New multiple times with static strings |
| internal/pdf/redact/redactor.go | 78 | go_perf_layer_caching_cache_key_built_with_fmt / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_caching_cache_key_built_with_fmt |
| internal/pdf/redact/redactor.go | 78 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/redact/redactor.go | 78 | go_perf_layer_caching_unbounded_cache_map / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_caching_unbounded_cache_map |
| internal/pdf/redact/redactor.go | 56 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/redactor.go | 78 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function GetPageInfo matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/redactor.go | 24 | errors_new_for_static_sentinel / performance / stable | function NewRedactor calls errors.New multiple times with static strings |
| internal/pdf/redact/redactor.go | 23 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function NewRedactor matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/search.go | 32 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function ExtractTextPositions matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/redact/search.go | 69 | slice_append_without_prealloc_known_bound / hot_path / stable | function FindTextOccurrences appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 77 | slice_append_without_prealloc_known_bound / hot_path / stable | function FindTextOccurrences appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 69 | slice_grow_without_cap_hint / performance / stable | function FindTextOccurrences appends to redactions in a loop without a capacity hint |
| internal/pdf/redact/search.go | 77 | slice_grow_without_cap_hint / performance / stable | function FindTextOccurrences appends to redactions in a loop without a capacity hint |
| internal/pdf/redact/search.go | 45 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/search.go | 46 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/search.go | 69 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/search.go | 45 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/search.go | 45 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function FindTextOccurrences matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/redact/search.go | 92 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function FindTextOccurrencesMulti matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/search.go | 86 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function FindTextOccurrencesMulti matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/search.go | 85 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function FindTextOccurrencesMulti matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/search.go | 85 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function FindTextOccurrencesMulti matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/redact/search.go | 95 | stable_value_normalization_in_inner_loop / hot_path / stable | function FindTextOccurrencesMulti normalizes a stable value inside a loop |
| internal/pdf/redact/search.go | 100 | map_lookup_double_access / performance / stable | function FindTextOccurrencesMulti performs double map lookup for same key |
| internal/pdf/redact/search.go | 152 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function buildSubstringRects matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/redact/search.go | 300 | slice_append_without_prealloc_known_bound / hot_path / stable | function findAllCombinedMatchRects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 303 | slice_append_without_prealloc_known_bound / hot_path / stable | function findAllCombinedMatchRects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 400 | slice_append_without_prealloc_known_bound / hot_path / stable | function findAllCombinedMatchRects appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/search.go | 300 | slice_grow_without_cap_hint / performance / stable | function findAllCombinedMatchRects appends to lines in a loop without a capacity hint |
| internal/pdf/redact/search.go | 303 | slice_grow_without_cap_hint / performance / stable | function findAllCombinedMatchRects appends to lines in a loop without a capacity hint |
| internal/pdf/redact/search.go | 400 | slice_grow_without_cap_hint / performance / stable | function findAllCombinedMatchRects appends to results in a loop without a capacity hint |
| internal/pdf/redact/search.go | 241 | copy_append_idiom_waste / performance / stable | function findAllCombinedMatchRects clones a slice via append(nil, src...) |
| internal/pdf/redact/search.go | 236 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function findAllCombinedMatchRects matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/search.go | 241 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function findAllCombinedMatchRects matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/redact/search.go | 319 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function findAllCombinedMatchRects matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/redact/search.go | 241 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function findAllCombinedMatchRects matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/search.go | 226 | repeated_string_trim_normalize / performance / stable | function normalizeSearchText chains multiple string normalization operations |
| internal/pdf/redact/secure.go | 311 | unnecessary_slice_copy_for_readonly / performance / stable | function applyRectMaskToText clones a slice that appears to be read-only |
| internal/pdf/redact/secure.go | 311 | copy_append_idiom_waste / performance / stable | function applyRectMaskToText clones a slice via append(nil, src...) |
| internal/pdf/redact/secure.go | 284 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function applyRectMaskToText matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/redact/secure.go | 311 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function applyRectMaskToText matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/secure.go | 312 | filter_then_count_then_iterate / hot_path / stable | function applyRectMaskToText traverses the same collection multiple times for filter, count, and process |
| internal/pdf/redact/secure.go | 54 | allocation_churn_in_loop / performance / stable | function applySecureContentRedactions allocates new objects inside a loop |
| internal/pdf/redact/secure.go | 54 | likely_n_squared_allocation / performance / experimental | function applySecureContentRedactions appears to allocate inside a nested loop |
| internal/pdf/redact/secure.go | 27 | map_of_slices_prealloc / performance / stable | function applySecureContentRedactions appends to a map-of-slices entry without preallocating |
| internal/pdf/redact/secure.go | 43 | slice_append_without_prealloc_known_bound / hot_path / stable | function applySecureContentRedactions appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 50 | slice_append_without_prealloc_known_bound / hot_path / stable | function applySecureContentRedactions appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 62 | slice_append_without_prealloc_known_bound / hot_path / stable | function applySecureContentRedactions appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 43 | slice_grow_without_cap_hint / performance / stable | function applySecureContentRedactions appends to warnings in a loop without a capacity hint |
| internal/pdf/redact/secure.go | 50 | slice_grow_without_cap_hint / performance / stable | function applySecureContentRedactions appends to warnings in a loop without a capacity hint |
| internal/pdf/redact/secure.go | 62 | slice_grow_without_cap_hint / performance / stable | function applySecureContentRedactions appends to warnings in a loop without a capacity hint |
| internal/pdf/redact/secure.go | 27 | map_growth_without_size_hint / hot_path / stable | function applySecureContentRedactions inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/secure.go | 17 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function applySecureContentRedactions matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/secure.go | 25 | go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans / performance / stable | function applySecureContentRedactions matches performance-layer rule go_perf_layer_algorithmic_complexity_repeated_nested_slice_scans |
| internal/pdf/redact/secure.go | 25 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function applySecureContentRedactions matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/secure.go | 25 | go_perf_layer_data_structure_choice_map_string_bool_for_membership / performance / stable | function applySecureContentRedactions matches performance-layer rule go_perf_layer_data_structure_choice_map_string_bool_for_membership |
| internal/pdf/redact/secure.go | 18 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function applySecureContentRedactions matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/secure.go | 19 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function applySecureContentRedactions matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/secure.go | 54 | make_map_inside_hot_loop_same_shape / hot_path / stable | function applySecureContentRedactions recreates scratch maps inside a loop |
| internal/pdf/redact/secure.go | 422 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildRedactionTJArray appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 428 | slice_append_without_prealloc_known_bound / hot_path / stable | function buildRedactionTJArray appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 422 | slice_grow_without_cap_hint / performance / stable | function buildRedactionTJArray appends to segments in a loop without a capacity hint |
| internal/pdf/redact/secure.go | 428 | slice_grow_without_cap_hint / performance / stable | function buildRedactionTJArray appends to segments in a loop without a capacity hint |
| internal/pdf/redact/secure.go | 378 | fmt_hot_path / performance / stable | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 400 | fmt_hot_path / performance / stable | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 451 | fmt_hot_path / performance / stable | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 456 | fmt_hot_path / performance / stable | function buildRedactionTJArray formats strings with fmt inside a loop |
| internal/pdf/redact/secure.go | 373 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function buildRedactionTJArray matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/redact/secure.go | 422 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function buildRedactionTJArray matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/secure.go | 399 | filter_then_count_then_iterate / hot_path / stable | function buildRedactionTJArray traverses the same collection multiple times for filter, count, and process |
| internal/pdf/redact/secure.go | 376 | builder_write_string_single_byte_literal / performance / stable | function buildRedactionTJArray uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/redact/secure.go | 398 | builder_write_string_single_byte_literal / performance / stable | function buildRedactionTJArray uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/redact/secure.go | 443 | builder_write_string_single_byte_literal / performance / stable | function buildRedactionTJArray uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/redact/secure.go | 454 | builder_write_string_single_byte_literal / performance / stable | function buildRedactionTJArray uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/redact/secure.go | 460 | builder_write_string_single_byte_literal / performance / stable | function buildRedactionTJArray uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/redact/secure.go | 375 | strings_builder_without_grow_known_bound / hot_path / stable | function buildRedactionTJArray uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/redact/secure.go | 155 | slice_append_without_prealloc_known_bound / hot_path / stable | function extractContentKeys appends to a slice inside a range loop without visible preallocation |
| internal/pdf/redact/secure.go | 155 | slice_grow_without_cap_hint / performance / stable | function extractContentKeys appends to keys in a loop without a capacity hint |
| internal/pdf/redact/secure.go | 155 | string_concat_in_loop / performance / stable | function extractContentKeys concatenates strings inside a loop |
| internal/pdf/redact/secure.go | 155 | byte_string_conversion_in_loop / hot_path / stable | function extractContentKeys converts between bytes and strings inside a loop |
| internal/pdf/redact/secure.go | 148 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function extractContentKeys matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/redact/secure.go | 338 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function replaceCaseInsensitiveWithSpaces matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/redact/secure.go | 201 | three_index_slice_for_append_safety / performance / stable | function rewriteContentStreamSecure appends to a subslice without capping its capacity |
| internal/pdf/redact/secure.go | 178 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function rewriteContentStreamSecure matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/secure.go | 170 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function rewriteContentStreamSecure matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/redact/secure.go | 189 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function rewriteContentStreamSecure matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/redact/secure.go | 170 | go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write / performance / stable | function rewriteContentStreamSecure matches performance-layer rule go_perf_layer_memory_allocation_temporary_byte_slice_for_string_write |
| internal/pdf/redact/secure.go | 169 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function rewriteContentStreamSecure matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/redact/secure.go | 109 | regexp_compile_in_hot_path / hot_path / stable | function rewriteSecureStreamTree compiles regular expressions inside a loop |
| internal/pdf/redact/secure.go | 80 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function rewriteSecureStreamTree matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/secure.go | 217 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function scrubDecodedContent matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/redact/secure.go | 216 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function scrubDecodedContent matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/redact/secure.go | 280 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function scrubDecodedContent matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/redact/secure.go | 215 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function scrubDecodedContent matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/redact/secure.go | 262 | stable_value_normalization_in_inner_loop / hot_path / stable | function scrubDecodedContent normalizes a stable value inside a loop |
| internal/pdf/redact/secure.go | 221 | strings_builder_without_grow_known_bound / hot_path / stable | function scrubDecodedContent uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/redact/visual.go | 32 | map_of_slices_prealloc / performance / stable | function ApplyRedactions appends to a map-of-slices entry without preallocating |
| internal/pdf/redact/visual.go | 59 | fmt_hot_path / performance / stable | function ApplyRedactions formats strings with fmt inside a loop |
| internal/pdf/redact/visual.go | 65 | fmt_hot_path / performance / stable | function ApplyRedactions formats strings with fmt inside a loop |
| internal/pdf/redact/visual.go | 70 | fmt_hot_path / performance / stable | function ApplyRedactions formats strings with fmt inside a loop |
| internal/pdf/redact/visual.go | 32 | map_growth_without_size_hint / hot_path / stable | function ApplyRedactions inserts into a map in a loop without a visible size hint |
| internal/pdf/redact/visual.go | 30 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function ApplyRedactions matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/redact/visual.go | 14 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ApplyRedactions matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/redact/visual.go | 14 | go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop / performance / stable | function ApplyRedactions matches performance-layer rule go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop |
| internal/pdf/redact/visual.go | 13 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ApplyRedactions matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/redact/visual.go | 55 | builder_or_buffer_recreated_per_iteration / hot_path / stable | function ApplyRedactions recreates a strings.Builder inside a loop |
| internal/pdf/redact/visual.go | 55 | strings_builder_without_grow_known_bound / hot_path / stable | function ApplyRedactions uses strings.Builder without Grow when approximate size is locally visible |
| internal/pdf/signature/helpers.go | 10 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function fmtNum matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/signature/signature.go | 119 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function CreateSignatureField matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/signature/signature.go | 237 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function CreateSignatureField matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/signature/signature.go | 148 | go_perf_layer_framework_performance_template_execute_to_string_then_write / performance / stable | function CreateSignatureField matches performance-layer rule go_perf_layer_framework_performance_template_execute_to_string_then_write |
| internal/pdf/signature/signature.go | 178 | go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item / performance / stable | function CreateSignatureField matches performance-layer rule go_perf_layer_hot_path_optimization_time_now_called_many_times_per_item |
| internal/pdf/signature/signature.go | 148 | go_perf_layer_lazy_loading_eager_metric_label_cardinality_build / performance / stable | function CreateSignatureField matches performance-layer rule go_perf_layer_lazy_loading_eager_metric_label_cardinality_build |
| internal/pdf/signature/signature.go | 205 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function CreateSignatureField matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/signature/signature.go | 140 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function CreateSignatureField matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/signature/signature.go | 160 | builder_write_string_single_byte_literal / performance / stable | function CreateSignatureField uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/signature/signature.go | 161 | builder_write_string_single_byte_literal / performance / stable | function CreateSignatureField uses `strings.Builder.WriteString("x")` instead of `WriteByte('x')`. |
| internal/pdf/signature/signature.go | 187 | sprintf_for_simple_string_format / performance / stable | function CreateSignatureField uses fmt.Sprintf with only %s verbs |
| internal/pdf/signature/signature.go | 214 | sprintf_for_simple_string_format / performance / stable | function CreateSignatureField uses fmt.Sprintf with only %s verbs |
| internal/pdf/signature/signature.go | 101 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function NewPDFSigner matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/signature/signature.go | 583 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/signature/signature.go | 603 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/signature/signature.go | 584 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/signature/signature.go | 586 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/signature/signature.go | 583 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/signature/signature.go | 584 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/signature/signature.go | 583 | go_perf_layer_string_handling_byte_string_roundtrip_for_contains / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_string_handling_byte_string_roundtrip_for_contains |
| internal/pdf/signature/signature.go | 591 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function UpdatePDFWithSignature matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/signature/signature.go | 482 | slice_append_without_prealloc_known_bound / hot_path / stable | function createPKCS7SignedData appends to a slice inside a range loop without visible preallocation |
| internal/pdf/signature/signature.go | 482 | slice_grow_without_cap_hint / performance / stable | function createPKCS7SignedData appends to certBytes in a loop without a capacity hint |
| internal/pdf/signature/signature.go | 412 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function createPKCS7SignedData matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/signature/signature.go | 372 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function createPKCS7SignedData matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| internal/pdf/signature/signature.go | 378 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function createPKCS7SignedData matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/signature/signature.go | 413 | go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop / performance / stable | function createPKCS7SignedData matches performance-layer rule go_perf_layer_error_handling_cost_errors_wrap_in_tight_loop |
| internal/pdf/signature/signature.go | 385 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function createPKCS7SignedData matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/signature/signature.go | 383 | go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary / performance / stable | function createPKCS7SignedData matches performance-layer rule go_perf_layer_runtime_configuration_debug_build_tags_in_hot_binary |
| internal/pdf/signature/signature.go | 385 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function createPKCS7SignedData matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/pdf/signature/signature.go | 255 | sprintf_for_simple_string_format / performance / stable | function createSignatureAppearance uses fmt.Sprintf with only %s verbs |
| internal/pdf/signature/signature.go | 257 | sprintf_for_simple_string_format / performance / stable | function createSignatureAppearance uses fmt.Sprintf with only %s verbs |
| internal/pdf/signature/signature.go | 565 | panic_on_error / errors / stable | function mustMarshal escalates ordinary error handling through panic or fatal logging |
| internal/pdf/signature/signature.go | 566 | go_perf_layer_error_handling_cost_panic_recover_for_control_flow / performance / stable | function mustMarshal matches performance-layer rule go_perf_layer_error_handling_cost_panic_recover_for_control_flow |
| internal/pdf/structure.go | 125 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function BeginMarkedContent matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/structure.go | 161 | go_perf_layer_serialization_gzip_writer_created_per_small_payload / performance / stable | function BeginMarkedContent matches performance-layer rule go_perf_layer_serialization_gzip_writer_created_per_small_payload |
| internal/pdf/structure.go | 189 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function BeginMarkedContentBuf matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| internal/pdf/structure.go | 186 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function BeginMarkedContentBuf matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/structure.go | 208 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function BeginMarkedContentBuf matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/structure.go | 273 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function GenerateStructTreeRoot matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/structure.go | 96 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function NewStructureManager matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| internal/pdf/structure.go | 258 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function RegisterPageStructParents matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/svg/svg.go | 110 | allocation_churn_in_loop / performance / stable | function ConvertSVGToPDFCommands allocates new objects inside a loop |
| internal/pdf/svg/svg.go | 110 | likely_n_squared_allocation / performance / experimental | function ConvertSVGToPDFCommands appears to allocate inside a nested loop |
| internal/pdf/svg/svg.go | 110 | repeated_map_clone_in_loop / hot_path / stable | function ConvertSVGToPDFCommands clones maps inside a loop |
| internal/pdf/svg/svg.go | 124 | fmt_hot_path / performance / stable | function ConvertSVGToPDFCommands formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 130 | fmt_hot_path / performance / stable | function ConvertSVGToPDFCommands formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 167 | fmt_hot_path / performance / stable | function ConvertSVGToPDFCommands formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 98 | map_growth_without_size_hint / hot_path / stable | function ConvertSVGToPDFCommands inserts into a map in a loop without a visible size hint |
| internal/pdf/svg/svg.go | 87 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| internal/pdf/svg/svg.go | 84 | go_perf_layer_collection_iteration_range_over_map_for_deterministic_first / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_collection_iteration_range_over_map_for_deterministic_first |
| internal/pdf/svg/svg.go | 52 | go_perf_layer_data_structure_choice_small_enum_string_switch_map / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_data_structure_choice_small_enum_string_switch_map |
| internal/pdf/svg/svg.go | 52 | go_perf_layer_error_handling_cost_error_string_built_before_error_needed / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_error_handling_cost_error_string_built_before_error_needed |
| internal/pdf/svg/svg.go | 84 | go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_error_handling_cost_sentinel_error_allocated_per_call |
| internal/pdf/svg/svg.go | 98 | go_perf_layer_framework_performance_gin_context_copied_for_sync_path / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_framework_performance_gin_context_copied_for_sync_path |
| internal/pdf/svg/svg.go | 52 | go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_garbage_collection_cleanup_sync_pool_stores_large_unbounded_buffers |
| internal/pdf/svg/svg.go | 43 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| internal/pdf/svg/svg.go | 47 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/svg/svg.go | 67 | go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_memory_allocation_bytes_buffer_allocated_per_record |
| internal/pdf/svg/svg.go | 84 | go_perf_layer_memory_allocation_closure_capture_allocates_in_loop / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_memory_allocation_closure_capture_allocates_in_loop |
| internal/pdf/svg/svg.go | 87 | go_perf_layer_serialization_json_decoder_without_reuse_for_stream / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_serialization_json_decoder_without_reuse_for_stream |
| internal/pdf/svg/svg.go | 43 | go_perf_layer_serialization_json_marshal_for_deep_equal / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_serialization_json_marshal_for_deep_equal |
| internal/pdf/svg/svg.go | 43 | go_perf_layer_serialization_map_any_json_decode_in_hot_path / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_serialization_map_any_json_decode_in_hot_path |
| internal/pdf/svg/svg.go | 167 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function ConvertSVGToPDFCommands matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| internal/pdf/svg/svg.go | 110 | make_map_inside_hot_loop_same_shape / hot_path / stable | function ConvertSVGToPDFCommands recreates scratch maps inside a loop |
| internal/pdf/svg/svg.go | 67 | bytes_buffer_without_grow_known_bound / hot_path / stable | function ConvertSVGToPDFCommands uses bytes.Buffer without Grow when approximate size is locally visible |
| internal/pdf/svg/svg.go | 52 | strings_replace_all_for_single_char / performance / stable | function ConvertSVGToPDFCommands uses strings.ReplaceAll for single character replacement |
| internal/pdf/svg/svg.go | 257 | strconv_repeat_on_same_binding / hot_path / stable | function applyTransform converts the same string input with strconv multiple times |
| internal/pdf/svg/svg.go | 258 | strconv_repeat_on_same_binding / hot_path / stable | function applyTransform converts the same string input with strconv multiple times |
| internal/pdf/svg/svg.go | 252 | fmt_hot_path / performance / stable | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 259 | fmt_hot_path / performance / stable | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 262 | fmt_hot_path / performance / stable | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 267 | fmt_hot_path / performance / stable | function applyTransform formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 250 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function applyTransform matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/svg/svg.go | 242 | strings_replace_all_for_single_char / performance / stable | function applyTransform uses strings.ReplaceAll for single character replacement |
| internal/pdf/svg/svg.go | 281 | strings_replace_all_for_single_char / performance / stable | function extractArgs uses strings.ReplaceAll for single character replacement |
| internal/pdf/svg/svg.go | 289 | go_perf_layer_collection_iteration_len_called_after_materializing_channel / performance / stable | function parseColor matches performance-layer rule go_perf_layer_collection_iteration_len_called_after_materializing_channel |
| internal/pdf/svg/svg.go | 285 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function parseColor matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/svg/svg.go | 360 | strings_hassuffix_then_trimsuffix / performance / stable | function parseColorComponent checks HasSuffix then TrimSuffix (use CutSuffix) |
| internal/pdf/svg/svg.go | 459 | string_concat_in_loop / performance / stable | function parsePathData concatenates strings inside a loop |
| internal/pdf/svg/svg.go | 475 | strconv_repeat_on_same_binding / hot_path / stable | function parsePathData converts the same string input with strconv multiple times |
| internal/pdf/svg/svg.go | 477 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 486 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 493 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 502 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 508 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 513 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 519 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 524 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 539 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 555 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 579 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 604 | fmt_hot_path / performance / stable | function parsePathData formats strings with fmt inside a loop |
| internal/pdf/svg/svg.go | 456 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function parsePathData matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| internal/pdf/svg/svg.go | 456 | strings_replace_all_for_single_char / performance / stable | function parsePathData uses strings.ReplaceAll for single character replacement |
| internal/pdf/svg/svg.go | 459 | strings_replace_all_for_single_char / performance / stable | function parsePathData uses strings.ReplaceAll for single character replacement |
| internal/pdf/svg/svg.go | 207 | map_growth_without_size_hint / hot_path / stable | function processElement inserts into a map in a loop without a visible size hint |
| internal/pdf/svg/svg.go | 204 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function processElement matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/svg/svg.go | 375 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function processVisualElement matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/types.go | 74 | go_perf_layer_string_handling_string_lower_for_case_insensitive_compare / performance / stable | function getPageDimensions matches performance-layer rule go_perf_layer_string_handling_string_lower_for_case_insensitive_compare |
| internal/pdf/types.go | 9 | mutable_package_global / idioms / stable | package variable pageSizes is mutated across function bodies |
| internal/pdf/typst_math_test.go | 290 | likely_n_squared_string_concat / performance / experimental | function TestTypstMathStress_GenerateEquationBankPDF appears to concatenate strings inside a nested loop |
| internal/pdf/typst_math_test.go | 290 | string_concat_in_loop / performance / stable | function TestTypstMathStress_GenerateEquationBankPDF concatenates strings inside a loop |
| internal/pdf/typst_math_test.go | 421 | string_concat_in_loop / performance / stable | function TestTypstMathStress_GenerateImageStyleShowcasePDF concatenates strings inside a loop |
| internal/pdf/typst_math_test.go | 125 | string_concat_in_loop / performance / stable | function TestTypstMathStress_GenerateTemplatePDFWithIntegrals concatenates strings inside a loop |
| internal/pdf/utils.go | 402 | slice_append_without_prealloc_known_bound / hot_path / stable | function WrapText appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 406 | slice_append_without_prealloc_known_bound / hot_path / stable | function WrapText appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 424 | slice_append_without_prealloc_known_bound / hot_path / stable | function WrapText appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 402 | slice_grow_without_cap_hint / performance / stable | function WrapText appends to lines in a loop without a capacity hint |
| internal/pdf/utils.go | 406 | slice_grow_without_cap_hint / performance / stable | function WrapText appends to lines in a loop without a capacity hint |
| internal/pdf/utils.go | 424 | slice_grow_without_cap_hint / performance / stable | function WrapText appends to lines in a loop without a capacity hint |
| internal/pdf/utils.go | 413 | string_concat_in_loop / performance / stable | function WrapText concatenates strings inside a loop |
| internal/pdf/utils.go | 402 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function WrapText matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/utils.go | 311 | slice_append_without_prealloc_known_bound / hot_path / stable | function formatPageKids appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 314 | slice_append_without_prealloc_known_bound / hot_path / stable | function formatPageKids appends to a slice inside a range loop without visible preallocation |
| internal/pdf/utils.go | 311 | slice_grow_without_cap_hint / performance / stable | function formatPageKids appends to buf in a loop without a capacity hint |
| internal/pdf/utils.go | 314 | slice_grow_without_cap_hint / performance / stable | function formatPageKids appends to buf in a loop without a capacity hint |
| internal/pdf/utils.go | 311 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function formatPageKids matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/utils.go | 35 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function parseHexColor matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| internal/pdf/utils.go | 142 | strconv_repeat_on_same_binding / hot_path / stable | function parseProps converts the same string input with strconv multiple times |
| internal/pdf/utils.go | 85 | go_perf_layer_caching_json_cache_value_stored_as_string / performance / stable | function parseProps matches performance-layer rule go_perf_layer_caching_json_cache_value_stored_as_string |
| internal/pdf/utils.go | 83 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function parseProps matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| internal/pdf/utils.go | 182 | go_perf_layer_string_handling_regexp_compile_in_request_path / performance / stable | function resolveFontName matches performance-layer rule go_perf_layer_string_handling_regexp_compile_in_request_path |
| internal/pdf/utils.go | 470 | slice_grow_without_cap_hint / performance / stable | function wrapLongWord appends to lines in a loop without a capacity hint |
| internal/pdf/utils.go | 470 | byte_string_conversion_in_loop / hot_path / stable | function wrapLongWord converts between bytes and strings inside a loop |
| internal/pdf/utils.go | 470 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function wrapLongWord matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| internal/pdf/utils.go | 4 | helper_or_utils_package_contains_domain_logic / architecture / stable | generic helper or utils package imports domain-specific layers |
| internal/pdf/utils.go | 14 | mutable_package_global / idioms / stable | package variable hexNibble is mutated across function bodies |
| pkg/fontutils/fontutils.go | 88 | feature_flag_lookup_without_config_abstraction / architecture / stable | feature flag or config lookup happens inline |
| pkg/fontutils/fontutils.go | 147 | waitgroup_add_inside_loop / performance / stable | function EnsureMathFonts calls wg.Add(1) inside a loop |
| pkg/fontutils/fontutils.go | 147 | waitgroup_fanout_without_errgroup_on_error_path / concurrency / stable | function EnsureMathFonts fans out work with WaitGroup while errors still need coordinated cancellation |
| pkg/fontutils/fontutils.go | 148 | go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit / performance / stable | function EnsureMathFonts matches performance-layer rule go_perf_layer_async_concurrency_goroutine_per_item_without_worker_limit |
| pkg/fontutils/fontutils.go | 110 | slice_append_without_prealloc_known_bound / hot_path / stable | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 112 | slice_append_without_prealloc_known_bound / hot_path / stable | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 114 | slice_append_without_prealloc_known_bound / hot_path / stable | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 117 | slice_append_without_prealloc_known_bound / hot_path / stable | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 120 | slice_append_without_prealloc_known_bound / hot_path / stable | function MathFontCandidates appends to a slice inside a range loop without visible preallocation |
| pkg/fontutils/fontutils.go | 110 | slice_grow_without_cap_hint / performance / stable | function MathFontCandidates appends to paths in a loop without a capacity hint |
| pkg/fontutils/fontutils.go | 112 | slice_grow_without_cap_hint / performance / stable | function MathFontCandidates appends to paths in a loop without a capacity hint |
| pkg/fontutils/fontutils.go | 114 | slice_grow_without_cap_hint / performance / stable | function MathFontCandidates appends to paths in a loop without a capacity hint |
| pkg/fontutils/fontutils.go | 117 | slice_grow_without_cap_hint / performance / stable | function MathFontCandidates appends to paths in a loop without a capacity hint |
| pkg/fontutils/fontutils.go | 120 | slice_grow_without_cap_hint / performance / stable | function MathFontCandidates appends to paths in a loop without a capacity hint |
| pkg/fontutils/fontutils.go | 110 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function MathFontCandidates matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| pkg/fontutils/fontutils.go | 196 | http_client_allocated_per_call_without_reuse / idioms / stable | function downloadFont allocates http.Client inline instead of reusing shared client state |
| pkg/fontutils/fontutils.go | 196 | go_perf_layer_network_calls_http_client_created_per_call / performance / stable | function downloadFont constructs http.Client on a regular call path |
| pkg/fontutils/fontutils.go | 216 | go_perf_layer_collection_iteration_copy_slice_before_readonly_range / performance / stable | function downloadFont matches performance-layer rule go_perf_layer_collection_iteration_copy_slice_before_readonly_range |
| pkg/fontutils/fontutils.go | 197 | go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse / performance / stable | function downloadFont matches performance-layer rule go_perf_layer_garbage_collection_cleanup_response_body_not_drained_for_reuse |
| pkg/gopdflib/example_test.go | 131 | panic_on_error / errors / stable | function ExampleGeneratePDF escalates ordinary error handling through panic or fatal logging |
| pkg/gopdflib/example_test.go | 146 | panic_on_error / errors / stable | function ExampleMergePDFs escalates ordinary error handling through panic or fatal logging |
| sampledata/benchmarks/fpdf/bench.py | 30 | metric_name_contains_dynamic_user_or_data_values / observability / stable | function run_once appears to build metric names from dynamic data |
| sampledata/benchmarks/fpdf/bench.py | 30 | cross_cutting_policies_embedded_in_leaf_modules_instead_of_shared_boundary / structure / stable | function run_once embeds cross-cutting policy in a leaf module |
| sampledata/benchmarks/fpdf/bench.py | 30 | python_perf_layer_profiling_benchmarking_benchmark_missing_warmup / performance / stable | function run_once matches performance-layer rule python_perf_layer_profiling_benchmarking_benchmark_missing_warmup |
| sampledata/benchmarks/fpdf/bench.py | 30 | tuple_return_with_three_or_more_positional_fields_in_public_api / maintainability / stable | function run_once returns a wide positional tuple from a public-looking API |
| sampledata/benchmarks/fpdf/bench.py | 30 | helper_returns_index_based_tuple_instead_of_named_structure / maintainability / stable | function run_once returns an index-based tuple that callers must remember positionally |
| sampledata/benchmarks/fpdf/bench.py | 30 | function_returns_multiple_unlabeled_shape_variants / discipline / stable | function run_once returns multiple unlabeled shapes from the same function |
| sampledata/benchmarks/fpdf/bench.py | 36 | exception_swallowed / maintainability / stable | function run_once suppresses a broad exception instead of surfacing or narrowing it |
| sampledata/benchmarks/fpdf/bench.py | 30 | cache_object_exists_without_size_or_eviction_policy_documentation / maintainability / stable | function run_once uses cache-like state without a visible size or eviction policy signal |
| sampledata/benchmarks/fpdf/bench.py | 11 | import_time_file_io / quality / stable | module performs filesystem work while being imported |
| sampledata/benchmarks/gen_data.go | 17 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/benchmarks/gen_data.go | 42 | go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_microbenchmark_dead_code_eliminated |
| sampledata/benchmarks/gopdflib/bench.go | 11 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 42 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function readBenchmarkData matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 153 | mutex_in_loop / concurrency / stable | function runDataBenchGoPDFLib acquires a mutex inside a loop |
| sampledata/benchmarks/gopdflib/databench_gopdflib.go | 110 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function runDataBenchGoPDFLib matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/benchmarks/gopdfsuit/bench.go | 8 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/benchmarks/pypdfsuit/bench.py | 28 | python_perf_layer_profiling_benchmarking_benchmark_missing_warmup / performance / stable | function main matches performance-layer rule python_perf_layer_profiling_benchmarking_benchmark_missing_warmup |
| sampledata/benchmarks/pypdfsuit/bench.py | 28 | full_collection_sorted_when_partial_order_or_selection_suffices / performance / stable | function main sorts whole collections where partial selection may suffice |
| sampledata/benchmarks/pypdfsuit/bench.py | 21 | helper_returns_index_based_tuple_instead_of_named_structure / maintainability / stable | function run_once returns an index-based tuple that callers must remember positionally |
| sampledata/benchmarks/pypdfsuit/bench.py | 21 | function_returns_multiple_unlabeled_shape_variants / discipline / stable | function run_once returns multiple unlabeled shapes from the same function |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 25 | repeated_string_literal / duplication / stable | file repeats the same long string literal instead of centralizing it |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 20 | helper_name_hides_mutation_or_io_side_effect / discipline / stable | function load_data helper name sounds pure but the body performs mutation or I/O |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 21 | no_schema_validation_on_external_data / mlops / stable | function load_data parses external data without schema validation; corrupt input propagates silently |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 93 | metric_name_contains_dynamic_user_or_data_values / observability / stable | function main appears to build metric names from dynamic data |
| sampledata/benchmarks/pypdfsuit/databench_pypdfsuit.py | 93 | python_perf_layer_profiling_benchmarking_benchmark_missing_warmup / performance / stable | function main matches performance-layer rule python_perf_layer_profiling_benchmarking_benchmark_missing_warmup |
| sampledata/filler/compressed/generate_medical_form.py | 26 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function compress performs expensive transforms before cheap reject checks |
| sampledata/filler/compressed/generate_medical_form.py | 203 | string_concat_in_loop / performance / stable | function construct_object_stream concatenates strings inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 187 | quadratic_string_building_via_plus_equals / performance / stable | function construct_object_stream grows strings incrementally with += inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 199 | builtin_reduction_candidate / maintainability / stable | function construct_object_stream uses a loop shape that may read better with a Python built-in |
| sampledata/filler/compressed/generate_medical_form.py | 29 | invariant_computation_not_hoisted_out_of_nested_loop / hot_path_ext / stable | function generate_pdf appears to recompute invariant work inside nested loops |
| sampledata/filler/compressed/generate_medical_form.py | 92 | string_concat_in_loop / performance / stable | function generate_pdf concatenates strings inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 100 | string_concat_in_loop / performance / stable | function generate_pdf concatenates strings inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 29 | quadratic_string_building_via_plus_equals / performance / stable | function generate_pdf grows strings incrementally with += inside a loop |
| sampledata/filler/compressed/generate_medical_form.py | 29 | parallel_lists_used_instead_of_record_object / maintainability / stable | function generate_pdf maintains related data in parallel lists instead of one record shape |
| sampledata/filler/compressed/generate_medical_form.py | 106 | string_join_without_generator / hot_path_ext / stable | function generate_pdf passes a list comprehension to .join(); use a generator expression to avoid an intermediate list |
| sampledata/filler/compressed/generate_medical_form.py | 136 | string_join_without_generator / hot_path_ext / stable | function generate_pdf passes a list comprehension to .join(); use a generator expression to avoid an intermediate list |
| sampledata/filler/compressed/generate_medical_form.py | 29 | lookup_table_derived_from_constants_rebuilt_per_invocation / hot_path_ext / stable | function generate_pdf rebuilds lookup tables from constants per invocation |
| sampledata/filler/compressed/generate_medical_form.py | 29 | repeated_split_or_join_on_invariant_separator_inside_loop / hot_path / stable | function generate_pdf repeatedly splits or joins strings with invariant separators |
| sampledata/filler/compressed/generate_medical_form.py | 70 | builtin_reduction_candidate / maintainability / stable | function generate_pdf uses a loop shape that may read better with a Python built-in |
| sampledata/filler/compressed/generate_medical_form.py | 305 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function generate_xfdf performs blocking I/O per item without an obvious batching strategy |
| sampledata/filler/compressed/generate_medical_form.py | 305 | repeated_small_writes_without_buffering_or_join / performance / stable | function generate_xfdf performs repeated small writes without visible buffering |
| sampledata/filler/compressed/generate_medical_form.py | 229 | invariant_computation_not_hoisted_out_of_nested_loop / hot_path_ext / stable | function write_file appears to recompute invariant work inside nested loops |
| sampledata/filler/compressed/generate_medical_form.py | 229 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function write_file performs blocking I/O per item without an obvious batching strategy |
| sampledata/filler/compressed/generate_medical_form.py | 229 | same_buffer_or_prefix_reencoded_each_iteration / hot_path / stable | function write_file re-encodes the same buffer or prefix repeatedly |
| sampledata/filler/compressed/generate_medical_form.py | 229 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function write_file repeats context-manager patterns that may want one shared helper |
| sampledata/filler/compressed/generate_medical_form.py | 265 | sorted_full_collection_to_extract_top_n_elements / observability / stable | function write_file sorts full collection to extract top-N; use heapq.nsmallest()/nlargest() instead |
| sampledata/financialreport/data/generate_charts.py | 11 | invariant_computation_not_hoisted_out_of_nested_loop / hot_path_ext / stable | function generate_bar_chart appears to recompute invariant work inside nested loops |
| sampledata/financialreport/data/generate_charts.py | 11 | full_response_or_export_buffered_before_incremental_consumer_use / performance / stable | function generate_bar_chart buffers full output before handing it to a downstream consumer |
| sampledata/financialreport/data/generate_charts.py | 11 | magic_thresholds_duplicated_across_modules / maintainability / stable | function generate_bar_chart duplicates threshold-like numeric values that deserve named ownership |
| sampledata/financialreport/data/generate_charts.py | 37 | print_debugging_leftover / maintainability / stable | function generate_bar_chart leaves print()-based debugging in production code |
| sampledata/financialreport/data/generate_charts.py | 11 | binary_or_multipart_response_missing_explicit_content_type / observability / stable | function generate_bar_chart streams binary response without explicit Content-Type; clients rely on content sniffing |
| sampledata/financialreport/data/generate_charts.py | 47 | full_response_or_export_buffered_before_incremental_consumer_use / performance / stable | function generate_pie_chart buffers full output before handing it to a downstream consumer |
| sampledata/financialreport/data/generate_charts.py | 63 | print_debugging_leftover / maintainability / stable | function generate_pie_chart leaves print()-based debugging in production code |
| sampledata/financialreport/data/generate_charts.py | 47 | binary_or_multipart_response_missing_explicit_content_type / observability / stable | function generate_pie_chart streams binary response without explicit Content-Type; clients rely on content sniffing |
| sampledata/financialreport/data/generate_charts.py | 73 | text_bytes_boundary_relies_on_implicit_default_encoding / boundaries / stable | function main crosses text and bytes boundaries without an explicit encoding contract |
| sampledata/financialreport/data/generate_charts.py | 73 | repeated_file_open_for_same_resource_within_single_operation / performance / stable | function main reopens files repeatedly within one operation |
| sampledata/financialreport/data/generate_charts.py | 73 | repeated_split_or_join_on_invariant_separator_inside_loop / hot_path / stable | function main repeatedly splits or joins strings with invariant separators |
| sampledata/financialreport/data/generate_charts.py | 73 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function main repeats context-manager patterns that may want one shared helper |
| sampledata/gopdflib/financial_report/main.go | 81 | waitgroup_fanout_without_errgroup_on_error_path / concurrency / stable | function main fans out work with WaitGroup while errors still need coordinated cancellation |
| sampledata/gopdflib/financial_report/main.go | 87 | goroutine_without_shutdown_path / concurrency / stable | function main launches a looping goroutine without an obvious shutdown path |
| sampledata/gopdflib/load_from_json/main.go | 17 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/gopdflib/text_wrapping/main.go | 45 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function buildTextWrappingTemplate matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/gopdflib/zerodha/analyze_bench.py | 5 | orchestrator_performs_low_level_tokenization_or_parsing / architecture / stable | function parse_runs combines high-level orchestration with low-level parsing or tokenization work |
| sampledata/gopdflib/zerodha/analyze_bench.py | 5 | text_bytes_boundary_relies_on_implicit_default_encoding / boundaries / stable | function parse_runs crosses text and bytes boundaries without an explicit encoding contract |
| sampledata/gopdflib/zerodha/analyze_bench.py | 5 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function parse_runs performs blocking I/O per item without an obvious batching strategy |
| sampledata/gopdflib/zerodha/analyze_bench.py | 5 | repeated_split_or_join_on_invariant_separator_inside_loop / hot_path / stable | function parse_runs repeatedly splits or joins strings with invariant separators |
| sampledata/gopdflib/zerodha/analyze_bench.py | 5 | fallback_branch_swallows_invariant_violation_and_returns_plausible_default / quality / stable | function parse_runs returns a plausible default from a failure path and may hide invariant violations |
| sampledata/gopdflib/zerodha/analyze_bench.py | 17 | exception_handler_returns_default_without_any_logging / discipline / stable | function parse_runs returns default from except block with no logging; suppressed failures are invisible |
| sampledata/gopdflib/zerodha/main.go | 295 | rand_new_per_call / performance / stable | function buildActiveTraderTemplate uses `rand.New(...)` inside regular call paths instead of reusing a source or generator. |
| sampledata/gopdflib/zerodha/main.go | 295 | rand_newsource_per_call / performance / stable | function buildActiveTraderTemplate uses `rand.NewSource(...)` inside regular call paths. |
| sampledata/gopdflib/zerodha/main.go | 466 | rand_new_per_call / performance / stable | function buildHFTTemplate uses `rand.New(...)` inside regular call paths instead of reusing a source or generator. |
| sampledata/gopdflib/zerodha/main.go | 466 | rand_newsource_per_call / performance / stable | function buildHFTTemplate uses `rand.NewSource(...)` inside regular call paths. |
| sampledata/gopdflib/zerodha/main.go | 685 | goroutine_without_shutdown_path / concurrency / stable | function main launches a looping goroutine without an obvious shutdown path |
| sampledata/gopdflib/zerodha/main.go | 626 | go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report / performance / stable | function main matches performance-layer rule go_perf_layer_profiling_benchmarking_benchmark_missing_allocs_report |
| sampledata/gopdflib/zerodha/main.go | 687 | rand_new_per_call / performance / stable | function main uses `rand.New(...)` inside regular call paths instead of reusing a source or generator. |
| sampledata/gopdflib/zerodha/main.go | 687 | rand_newsource_per_call / performance / stable | function main uses `rand.NewSource(...)` inside regular call paths. |
| sampledata/gopdflib/zerodha/main.go | 687 | rand_newsource_with_time_now_per_call / performance / stable | function main uses `rand.NewSource(...)` seeded from wall clock on each call path. |
| sampledata/gopdflib/zerodha/main.go | 68 | mutable_package_global / idioms / stable | package variable actions is mutated across function bodies |
| sampledata/gopdflib/zerodha/main.go | 60 | mutable_package_global / idioms / stable | package variable symbols is mutated across function bodies |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 137 | duplicate_error_handler_block / duplication / stable | file repeats highly similar exception-handling blocks |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 157 | repeated_string_literal / duplication / stable | file repeats the same long string literal instead of centralizing it |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 333 | repeated_string_literal / duplication / stable | file repeats the same long string literal instead of centralizing it |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 105 | optional_parameter_used_without_none_guard / discipline / stable | function bookmark has Optional parameter but dereferences it without a None guard |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 250 | quadratic_string_building_via_plus_equals / performance / stable | function build_active_trader_template grows strings incrementally with += inside a loop |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 262 | builtin_reduction_candidate / maintainability / stable | function build_active_trader_template uses a loop shape that may read better with a Python built-in |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 73 | quadratic_string_building_via_plus_equals / performance / stable | function generate_trades grows strings incrementally with += inside a loop |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 73 | magic_value_branching / maintainability / stable | function generate_trades repeats branch-shaping literals instead of naming them explicitly |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 77 | builtin_reduction_candidate / maintainability / stable | function generate_trades uses a loop shape that may read better with a Python built-in |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 134 | temporary_collection_in_loop / performance / stable | function get_machine_info allocates a temporary collection inside a loop |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | god_function / structure / stable | function get_machine_info concentrates too much control flow and behavior |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function get_machine_info contains a deeply nested decision tree that would benefit from decomposition |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | function get_machine_info has opaque branching that suggests the code structure itself could be clearer |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | subprocess_or_shell_call_inside_record_processing_loop / hot_path / stable | function get_machine_info invokes subprocess work inside a record-processing loop |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function get_machine_info performs blocking I/O per item without an obvious batching strategy |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | repeated_split_or_join_on_invariant_separator_inside_loop / hot_path / stable | function get_machine_info repeatedly splits or joins strings with invariant separators |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 153 | exception_handler_returns_default_without_any_logging / discipline / stable | function get_machine_info returns default from except block with no logging; suppressed failures are invisible |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 109 | cache_object_exists_without_size_or_eviction_policy_documentation / maintainability / stable | function get_machine_info uses cache-like state without a visible size or eviction policy signal |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 49 | eager_full_file_or_stream_read_when_incremental_iteration_suffices / performance / stable | function read_chain eagerly reads full file or stream payloads where incremental iteration may suffice |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 45 | eager_full_file_or_stream_read_when_incremental_iteration_suffices / performance / stable | function read_text eagerly reads full file or stream payloads where incremental iteration may suffice |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 59 | cross_cutting_policies_embedded_in_leaf_modules_instead_of_shared_boundary / structure / stable | function retail_signature embeds cross-cutting policy in a leaf module |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | metric_name_contains_dynamic_user_or_data_values / observability / stable | function run_benchmark appears to build metric names from dynamic data |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 477 | concurrent_futures_executor_not_shut_down / architecture / stable | function run_benchmark creates a futures Executor without context manager or .shutdown(wait=True) |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | thread_pool_or_process_pool_created_and_destroyed_per_call / performance / stable | function run_benchmark creates an executor per call instead of reusing one |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | string_sentinel_values_duplicated_instead_of_constant_or_enum / maintainability / stable | function run_benchmark duplicates string sentinel values instead of centralizing them |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | quadratic_string_building_via_plus_equals / performance / stable | function run_benchmark grows strings incrementally with += inside a loop |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | python_perf_layer_profiling_benchmarking_benchmark_missing_warmup / performance / stable | function run_benchmark matches performance-layer rule python_perf_layer_profiling_benchmarking_benchmark_missing_warmup |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | mixed_concerns_function / structure / stable | function run_benchmark mixes multiple infrastructure concerns in one body |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function run_benchmark repeats context-manager patterns that may want one shared helper |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 415 | cache_object_exists_without_size_or_eviction_policy_documentation / maintainability / stable | function run_benchmark uses cache-like state without a visible size or eviction policy signal |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 35 | module_constant_rebound_after_public_import / boundaries / stable | module defines mutable constant-like state at top level |
| sampledata/gopdflib/zerodha/pypdfsuit_bench.py | 42 | module_constant_rebound_after_public_import / boundaries / stable | module defines mutable constant-like state at top level |
| sampledata/librarybook/data/generate_codes.py | 46 | atomic_replace_semantics_implemented_with_non_atomic_file_write / quality / stable | function generate_barcode appears to promise atomic replace semantics with non-atomic file writes |
| sampledata/librarybook/data/generate_codes.py | 46 | full_response_or_export_buffered_before_incremental_consumer_use / performance / stable | function generate_barcode buffers full output before handing it to a downstream consumer |
| sampledata/librarybook/data/generate_codes.py | 46 | batchable_writes_executed_one_at_a_time / performance / stable | function generate_barcode executes writes one at a time on an iteration path |
| sampledata/librarybook/data/generate_codes.py | 69 | print_debugging_leftover / maintainability / stable | function generate_barcode leaves print()-based debugging in production code |
| sampledata/librarybook/data/generate_codes.py | 46 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function generate_barcode performs blocking I/O per item without an obvious batching strategy |
| sampledata/librarybook/data/generate_codes.py | 46 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function generate_barcode repeats context-manager patterns that may want one shared helper |
| sampledata/librarybook/data/generate_codes.py | 46 | binary_or_multipart_response_missing_explicit_content_type / observability / stable | function generate_barcode streams binary response without explicit Content-Type; clients rely on content sniffing |
| sampledata/librarybook/data/generate_codes.py | 18 | full_response_or_export_buffered_before_incremental_consumer_use / performance / stable | function generate_qr_code buffers full output before handing it to a downstream consumer |
| sampledata/librarybook/data/generate_codes.py | 36 | print_debugging_leftover / maintainability / stable | function generate_qr_code leaves print()-based debugging in production code |
| sampledata/librarybook/data/generate_codes.py | 18 | binary_or_multipart_response_missing_explicit_content_type / observability / stable | function generate_qr_code streams binary response without explicit Content-Type; clients rely on content sniffing |
| sampledata/librarybook/data/generate_codes.py | 83 | text_bytes_boundary_relies_on_implicit_default_encoding / boundaries / stable | function main crosses text and bytes boundaries without an explicit encoding contract |
| sampledata/librarybook/data/generate_codes.py | 83 | batchable_writes_executed_one_at_a_time / performance / stable | function main executes writes one at a time on an iteration path |
| sampledata/librarybook/data/generate_codes.py | 83 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function main performs blocking I/O per item without an obvious batching strategy |
| sampledata/librarybook/data/generate_codes.py | 83 | repeated_small_writes_without_buffering_or_join / performance / stable | function main performs repeated small writes without visible buffering |
| sampledata/librarybook/data/generate_codes.py | 83 | repeated_file_open_for_same_resource_within_single_operation / performance / stable | function main reopens files repeatedly within one operation |
| sampledata/librarybook/data/generate_codes.py | 83 | repeated_split_or_join_on_invariant_separator_inside_loop / hot_path / stable | function main repeatedly splits or joins strings with invariant separators |
| sampledata/librarybook/data/generate_codes.py | 83 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function main repeats context-manager patterns that may want one shared helper |
| sampledata/python/JsonFileExample.py | 16 | repeated_path_exists_check_before_open_or_replace_in_loop / hot_path_ext / stable | function main checks path existence before repeated file operations |
| sampledata/python/JsonFileExample.py | 26 | f_string_evaluated_eagerly_inside_logging_call / observability / stable | function main evaluates f-string eagerly in logging call; use lazy % formatting or logger.isEnabledFor() |
| sampledata/python/JsonFileExample.py | 16 | expensive_work_starts_before_input_validation / discipline / stable | function main starts expensive work before validating cheap preconditions |
| sampledata/python/JsonFileExample.py | 14 | logging_basic_config_called_from_library_package / observability / stable | library module calls logging.basicConfig or addHandler at import time; this overrides the host application's log configuration |
| sampledata/python/amazonReceipt/amazonReceipt.py | 25 | over_abstracted_wrapper / structure / stable | class ReceiptItem looks ceremonial enough that a function or dataclass may suffice |
| sampledata/python/amazonReceipt/amazonReceipt.py | 109 | tuple_return_with_three_or_more_positional_fields_in_public_api / maintainability / stable | function _png_chunk returns a wide positional tuple from a public-looking API |
| sampledata/python/amazonReceipt/amazonReceipt.py | 109 | helper_returns_index_based_tuple_instead_of_named_structure / maintainability / stable | function _png_chunk returns an index-based tuple that callers must remember positionally |
| sampledata/python/amazonReceipt/amazonReceipt.py | 109 | function_returns_multiple_unlabeled_shape_variants / discipline / stable | function _png_chunk returns multiple unlabeled shapes from the same function |
| sampledata/python/amazonReceipt/amazonReceipt.py | 31 | helper_returns_index_based_tuple_instead_of_named_structure / maintainability / stable | function amount returns an index-based tuple that callers must remember positionally |
| sampledata/python/amazonReceipt/amazonReceipt.py | 31 | function_returns_multiple_unlabeled_shape_variants / discipline / stable | function amount returns multiple unlabeled shapes from the same function |
| sampledata/python/amazonReceipt/amazonReceipt.py | 154 | cross_cutting_policies_embedded_in_leaf_modules_instead_of_shared_boundary / structure / stable | function build_summary_table embeds cross-cutting policy in a leaf module |
| sampledata/python/amazonReceipt/amazonReceipt.py | 376 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function generate_pdf_with_page_margin performs expensive transforms before cheap reject checks |
| sampledata/python/amazonReceipt/amazonReceipt.py | 376 | generic_class_used_without_type_parameter_application / discipline / stable | function generate_pdf_with_page_margin uses bare generic container annotation without type parameters |
| sampledata/python/amazonReceipt/amazonReceipt.py | 452 | exception_raised_without_chaining_original_cause / discipline / stable | function main raises a new exception inside except without `from e`; original cause is lost |
| sampledata/python/amazonReceipt/amazonReceipt.py | 118 | quadratic_string_building_via_plus_equals / performance / stable | function sample_product_image grows strings incrementally with += inside a loop |
| sampledata/python/amazonReceipt/amazonReceipt.py | 118 | comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | function sample_product_image has opaque branching that suggests the code structure itself could be clearer |
| sampledata/python/amazonReceipt/amazonReceipt.py | 118 | compression_hashing_or_encoding_performed_before_cheap_reject_checks / performance / stable | function sample_product_image performs expensive transforms before cheap reject checks |
| sampledata/python/amazonReceipt/amazonReceipt.py | 118 | magic_value_branching / maintainability / stable | function sample_product_image repeats branch-shaping literals instead of naming them explicitly |
| sampledata/python/amazonReceipt/amazonReceipt.py | 118 | same_sequence_scanned_multiple_times_for_related_aggregates / hot_path_ext / stable | function sample_product_image scans the same sequence multiple times for related aggregates |
| sampledata/python/amazonReceipt/amazonReceipt.py | 382 | cross_cutting_policies_embedded_in_leaf_modules_instead_of_shared_boundary / structure / stable | function sample_receipt embeds cross-cutting policy in a leaf module |
| sampledata/python/amazonReceipt/amazonReceipt.py | 388 | rate_limit_429_response_missing_retry_after_header_or_stable_body / observability / stable | function sample_receipt returns HTTP 429 without a Retry-After header; clients cannot determine backoff interval |
| sampledata/python/amazonReceipt/amazonReceipt.py | 3 | cli_only_dependency_imported_by_library_entry_module / packaging / stable | library-style module imports CLI-only dependency argparse |
| sampledata/python/financial_report_pypdfsuit.py | 42 | repeated_string_literal / duplication / stable | file repeats the same long string literal instead of centralizing it |
| sampledata/python/financial_report_pypdfsuit.py | 342 | metric_name_contains_dynamic_user_or_data_values / observability / stable | function main appears to build metric names from dynamic data |
| sampledata/python/financial_report_pypdfsuit.py | 356 | broad_exception_handler / maintainability / stable | function main catches a broad exception without narrowing the failure type |
| sampledata/python/financial_report_pypdfsuit.py | 342 | python_perf_layer_profiling_benchmarking_benchmark_missing_warmup / performance / stable | function main matches performance-layer rule python_perf_layer_profiling_benchmarking_benchmark_missing_warmup |
| sampledata/python/financial_report_pypdfsuit.py | 342 | same_sequence_scanned_multiple_times_for_related_aggregates / hot_path_ext / stable | function main scans the same sequence multiple times for related aggregates |
| sampledata/python/gopdf/__init__.py | 1 | public_api_surface_defined_only_by_import_side_effects / packaging / stable | package root appears to define its public API primarily by import side effects |
| sampledata/python/gopdf/__init__.py | 1 | package_root_reexports_large_dependency_tree_by_default / packaging / stable | package root re-exports a large dependency tree by default |
| sampledata/python/gopdf/client.py | 27 | duplicate_error_handler_block / duplication / stable | file repeats highly similar exception-handling blocks |
| sampledata/python/gopdf/client.py | 9 | logger_instance_created_inside_function_body / observability / stable | function __init__ creates a logger inside the function body instead of reusing module scope logging |
| sampledata/python/gopdf/client.py | 57 | broad_exception_handler / maintainability / stable | function generate_from_file catches a broad exception without narrowing the failure type |
| sampledata/python/gopdf/client.py | 49 | text_bytes_boundary_relies_on_implicit_default_encoding / boundaries / stable | function generate_from_file crosses text and bytes boundaries without an explicit encoding contract |
| sampledata/python/gopdf/client.py | 57 | logger_error_inside_except_without_exc_info / observability / stable | function generate_from_file logs error inside except without exc_info=True; stack trace is lost from output |
| sampledata/python/gopdf/client.py | 49 | exception_log_omits_operation_identifier_or_input_summary / observability / stable | function generate_from_file logs exceptions without an operation identifier or input summary |
| sampledata/python/gopdf/client.py | 59 | redundant_return_none / maintainability / stable | function generate_from_file returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 49 | public_api_returns_none_or_value_without_explicit_optional_contract / quality / stable | function generate_from_file returns None on some paths without an explicit optional contract |
| sampledata/python/gopdf/client.py | 13 | validation_only_happens_after_expensive_side_effect_has_started / quality / stable | function generate_pdf begins expensive work before completing validation |
| sampledata/python/gopdf/client.py | 13 | type_branch_and_mode_branch_compounded_in_same_function / discipline / stable | function generate_pdf branches on both runtime type and mode-like values in the same function |
| sampledata/python/gopdf/client.py | 13 | network_boundary_without_timeout / maintainability / stable | function generate_pdf calls an external HTTP boundary without an obvious timeout or retry policy |
| sampledata/python/gopdf/client.py | 27 | broad_exception_handler / maintainability / stable | function generate_pdf catches a broad exception without narrowing the failure type |
| sampledata/python/gopdf/client.py | 13 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function generate_pdf contains a deeply nested decision tree that would benefit from decomposition |
| sampledata/python/gopdf/client.py | 32 | f_string_evaluated_eagerly_inside_logging_call / observability / stable | function generate_pdf evaluates f-string eagerly in logging call; use lazy % formatting or logger.isEnabledFor() |
| sampledata/python/gopdf/client.py | 13 | third_party_exception_type_leaks_across_architecture_boundary / architecture / stable | function generate_pdf exposes third-party exception types directly across an application boundary |
| sampledata/python/gopdf/client.py | 13 | public_api_forwards_library_specific_exception_shape / boundaries / stable | function generate_pdf forwards library-specific exception shapes directly through a public boundary |
| sampledata/python/gopdf/client.py | 27 | logger_error_inside_except_without_exc_info / observability / stable | function generate_pdf logs error inside except without exc_info=True; stack trace is lost from output |
| sampledata/python/gopdf/client.py | 13 | exception_log_omits_operation_identifier_or_input_summary / observability / stable | function generate_pdf logs exceptions without an operation identifier or input summary |
| sampledata/python/gopdf/client.py | 13 | warning_or_error_logs_emit_unbounded_payload_text / observability / stable | function generate_pdf logs potentially unbounded payload text on warning/error paths |
| sampledata/python/gopdf/client.py | 29 | redundant_return_none / maintainability / stable | function generate_pdf returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 43 | redundant_return_none / maintainability / stable | function generate_pdf returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 47 | redundant_return_none / maintainability / stable | function generate_pdf returns None explicitly where falling through would be clearer |
| sampledata/python/gopdf/client.py | 13 | public_api_returns_none_or_value_without_explicit_optional_contract / quality / stable | function generate_pdf returns None on some paths without an explicit optional contract |
| sampledata/python/gopdf/client.py | 13 | debug_log_serializes_full_large_object_graph / observability / stable | function generate_pdf serializes large object graphs for debug logging |
| sampledata/python/gopdf/client.py | 13 | default_timeout_missing_on_external_boundary_wrapper / quality / stable | function generate_pdf wraps an external call without a visible timeout policy |
| sampledata/python/gopdf/client.py | 1 | name_responsibility_mismatch / structure / stable | module client uses a utility-style name but coordinates multiple infrastructure concerns |
| sampledata/python/main.py | 18 | overloaded_dispatch_without_typing_overload_decorator / discipline / stable | function fill_template dispatches on isinstance for multiple types without @typing.overload signatures |
| sampledata/python/main.py | 14 | recursive_walk_over_untrusted_input_lacks_depth_limit / quality / stable | function fill_template recurses without an obvious depth limit |
| sampledata/python/main.py | 14 | function_returns_multiple_unlabeled_shape_variants / discipline / stable | function fill_template returns multiple unlabeled shapes from the same function |
| sampledata/python/main.py | 20 | recursive_traversal_risk / performance / stable | function fill_template uses direct recursion and may need an iterative traversal for deep inputs |
| sampledata/python/main.py | 22 | recursive_traversal_risk / performance / stable | function fill_template uses direct recursion and may need an iterative traversal for deep inputs |
| sampledata/python/main.py | 36 | type_branch_and_mode_branch_compounded_in_same_function / discipline / stable | function main branches on both runtime type and mode-like values in the same function |
| sampledata/python/main.py | 80 | broad_exception_handler / maintainability / stable | function main catches a broad exception without narrowing the failure type |
| sampledata/python/main.py | 36 | repeated_path_exists_check_before_open_or_replace_in_loop / hot_path_ext / stable | function main checks path existence before repeated file operations |
| sampledata/python/main.py | 36 | god_function / structure / stable | function main concentrates too much control flow and behavior |
| sampledata/python/main.py | 36 | condition_tree_nests_past_two_business_decision_levels / discipline / stable | function main contains a deeply nested decision tree that would benefit from decomposition |
| sampledata/python/main.py | 74 | f_string_evaluated_eagerly_inside_logging_call / observability / stable | function main evaluates f-string eagerly in logging call; use lazy % formatting or logger.isEnabledFor() |
| sampledata/python/main.py | 36 | comment_required_to_explain_opaque_branching_that_code_could_express / maintainability / stable | function main has opaque branching that suggests the code structure itself could be clearer |
| sampledata/python/main.py | 36 | mixed_concerns_function / structure / stable | function main mixes multiple infrastructure concerns in one body |
| sampledata/python/main.py | 36 | feature_logic_embedded_in_process_entrypoint / architecture / stable | function main owns feature or business branching directly in an entrypoint-style function |
| sampledata/python/main.py | 36 | blocking_io_call_executed_per_item_without_batching / hot_path_ext / stable | function main performs blocking I/O per item without an obvious batching strategy |
| sampledata/python/main.py | 36 | repeated_small_writes_without_buffering_or_join / performance / stable | function main performs repeated small writes without visible buffering |
| sampledata/python/main.py | 36 | repeated_file_open_for_same_resource_within_single_operation / performance / stable | function main reopens files repeatedly within one operation |
| sampledata/python/main.py | 36 | magic_value_branching / maintainability / stable | function main repeats branch-shaping literals instead of naming them explicitly |
| sampledata/python/main.py | 36 | same_contextmanager_pattern_copied_across_modules / maintainability / stable | function main repeats context-manager patterns that may want one shared helper |
| sampledata/python/main.py | 36 | expensive_work_starts_before_input_validation / discipline / stable | function main starts expensive work before validating cheap preconditions |
| sampledata/python/test_redact.py | 28 | test_wraps_sut_in_try_except_hiding_exception_detail / discipline / stable | function test_redaction wraps SUT in try/except without asserting exception type; hides unexpected exceptions |
| sampledata/svg/generate_math_svg.py | 79 | print_debugging_leftover / maintainability / stable | function save_math_svg leaves print()-based debugging in production code |
| sampledata/svg/generate_math_svg.py | 80 | print_debugging_leftover / maintainability / stable | function save_math_svg leaves print()-based debugging in production code |
| sampledata/svg/generate_math_svg.py | 59 | comparison_or_merge_logic_assumes_unique_keys_without_assertion / quality / stable | function save_math_svg merges or compares records without asserting uniqueness assumptions |
| sampledata/svg/generate_math_svg.py | 59 | multiple_regex_passes_over_same_text_without_precompiled_plan / performance / stable | function save_math_svg performs multiple regex passes over the same text |
| typstsyntax/parser.go | 131 | go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate / performance / stable | function parseExpr matches performance-layer rule go_perf_layer_garbage_collection_cleanup_large_slice_retained_after_truncate |
| typstsyntax/parser.go | 260 | slice_grow_without_cap_hint / performance / stable | function parseFunctionArgSequence appends to nodes in a loop without a capacity hint |
| typstsyntax/parser.go | 233 | slice_grow_without_cap_hint / performance / stable | function parseFunctionCall appends to args in a loop without a capacity hint |
| typstsyntax/parser.go | 235 | slice_grow_without_cap_hint / performance / stable | function parseFunctionCall appends to args in a loop without a capacity hint |
| typstsyntax/parser.go | 232 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function parseFunctionCall matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| typstsyntax/parser.go | 233 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function parseFunctionCall matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| typstsyntax/parser.go | 107 | slice_grow_without_cap_hint / performance / stable | function parseSequence appends to nodes in a loop without a capacity hint |
| typstsyntax/renderer.go | 1256 | go_perf_layer_string_handling_fmt_sprintf_for_simple_concat / performance / stable | function fmtFloat matches performance-layer rule go_perf_layer_string_handling_fmt_sprintf_for_simple_concat |
| typstsyntax/renderer.go | 440 | go_perf_layer_memory_allocation_map_recreated_for_static_lookup / performance / stable | function layoutAccent matches performance-layer rule go_perf_layer_memory_allocation_map_recreated_for_static_lookup |
| typstsyntax/renderer.go | 741 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutBigOperatorLimits appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 752 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutBigOperatorLimits appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 762 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutBigOperatorLimits appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 741 | slice_grow_without_cap_hint / performance / stable | function layoutBigOperatorLimits appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 752 | slice_grow_without_cap_hint / performance / stable | function layoutBigOperatorLimits appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 762 | slice_grow_without_cap_hint / performance / stable | function layoutBigOperatorLimits appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 704 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function layoutBigOperatorLimits matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| typstsyntax/renderer.go | 741 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function layoutBigOperatorLimits matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| typstsyntax/renderer.go | 801 | slice_grow_without_cap_hint / performance / stable | function layoutBinom appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 808 | slice_grow_without_cap_hint / performance / stable | function layoutBinom appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 846 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutCases appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 846 | slice_grow_without_cap_hint / performance / stable | function layoutCases appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 832 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function layoutCases matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| typstsyntax/renderer.go | 832 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function layoutCases matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| typstsyntax/renderer.go | 292 | slice_grow_without_cap_hint / performance / stable | function layoutFraction appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 308 | slice_grow_without_cap_hint / performance / stable | function layoutFraction appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 1112 | slice_grow_without_cap_hint / performance / stable | function layoutGenericFunc appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 1121 | slice_grow_without_cap_hint / performance / stable | function layoutGenericFunc appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 1102 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function layoutGenericFunc matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| typstsyntax/renderer.go | 1097 | go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func / performance / stable | function layoutGenericFunc matches performance-layer rule go_perf_layer_hot_path_optimization_allocation_in_hash_or_less_func |
| typstsyntax/renderer.go | 424 | slice_grow_without_cap_hint / performance / stable | function layoutGroup appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 900 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutLR appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 920 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutLR appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 900 | slice_grow_without_cap_hint / performance / stable | function layoutLR appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 920 | slice_grow_without_cap_hint / performance / stable | function layoutLR appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 956 | slice_grow_without_cap_hint / performance / stable | function layoutLR appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 885 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function layoutLR matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| typstsyntax/renderer.go | 575 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutMatrix appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 575 | slice_grow_without_cap_hint / performance / stable | function layoutMatrix appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 570 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function layoutMatrix matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| typstsyntax/renderer.go | 477 | allocation_churn_in_loop / performance / stable | function layoutMatrixGrid allocates new objects inside a loop |
| typstsyntax/renderer.go | 477 | likely_n_squared_allocation / performance / experimental | function layoutMatrixGrid appears to allocate inside a nested loop |
| typstsyntax/renderer.go | 547 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutMatrixGrid appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 547 | slice_grow_without_cap_hint / performance / stable | function layoutMatrixGrid appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 477 | make_slice_inside_hot_loop_same_shape / hot_path / stable | function layoutMatrixGrid recreates scratch slices inside a loop |
| typstsyntax/renderer.go | 366 | slice_grow_without_cap_hint / performance / stable | function layoutRoot appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 388 | slice_grow_without_cap_hint / performance / stable | function layoutRoot appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 354 | go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need / performance / stable | function layoutRoot matches performance-layer rule go_perf_layer_collection_iteration_manual_index_loop_without_bounds_need |
| typstsyntax/renderer.go | 1081 | nested_append_without_outer_capacity / hot_path / stable | function layoutSequence appends inside nested loops without visible preallocation |
| typstsyntax/renderer.go | 1081 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutSequence appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 1081 | slice_grow_without_cap_hint / performance / stable | function layoutSequence appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 1081 | go_perf_layer_memory_allocation_append_without_known_capacity / performance / stable | function layoutSequence matches performance-layer rule go_perf_layer_memory_allocation_append_without_known_capacity |
| typstsyntax/renderer.go | 347 | slice_grow_without_cap_hint / performance / stable | function layoutSqrt appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 642 | slice_append_without_prealloc_known_bound / hot_path / stable | function layoutVector appends to a slice inside a range loop without visible preallocation |
| typstsyntax/renderer.go | 642 | slice_grow_without_cap_hint / performance / stable | function layoutVector appends to elements in a loop without a capacity hint |
| typstsyntax/renderer.go | 589 | go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline / performance / stable | function layoutVector matches performance-layer rule go_perf_layer_algorithmic_complexity_quadratic_append_filter_pipeline |
| typstsyntax/renderer.go | 1235 | string_builder_write_string_vs_plus / performance / stable | function renderElements concatenates strings before WriteString |
| typstsyntax/renderer.go | 1237 | string_builder_write_string_vs_plus / performance / stable | function renderElements concatenates strings before WriteString |
| typstsyntax/renderer.go | 1240 | string_builder_write_string_vs_plus / performance / stable | function renderElements concatenates strings before WriteString |
| typstsyntax/renderer.go | 1242 | string_builder_write_string_vs_plus / performance / stable | function renderElements concatenates strings before WriteString |
| typstsyntax/typst.go | 81 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function RenderMathToLayout matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
| typstsyntax/typst.go | 74 | go_perf_layer_hot_path_optimization_invariant_parse_inside_handler / performance / stable | function RenderMathToText matches performance-layer rule go_perf_layer_hot_path_optimization_invariant_parse_inside_handler |
