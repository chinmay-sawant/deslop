# gopdfsuit temp report delta

This report compares `/home/chinmay/ChinmayPersonalProjects/deslop/verified_gopdfsuit_results.txt` against `/home/chinmay/ChinmayPersonalProjects/deslop/temp_gopdfsuit.txt`.

## Summary

- Verified findings: 386
- Temp findings: 432
- Newly added findings after file/line/rule filtering: 46
- Largest delta: `bindings/python/pypdfsuit/types.py` with 31 new rows

The diff was identified with `git diff --no-index`, then normalized to `file:line|rule` tuples so wording-only changes did not get counted as new rows.

## Likely False Positive

### `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/handlers/handlers.go:179` - `rows_without_close`

```go
func handleGetTemplateData(c *gin.Context) {
    filename := c.Query("file")
    if filename == "" {
        c.JSON(http.StatusBadRequest, gin.H{"error": "Missing 'file' query parameter"})
        return
    }

    if filepath.Ext(filename) != ".json" {
        c.JSON(http.StatusBadRequest, gin.H{"error": "Only JSON files are allowed"})
        return
    }

    filename = filepath.Base(filename)
    filePath := filepath.Join(getProjectRoot(), filename)

    data, err := os.ReadFile(filePath)
    if err != nil {
        c.JSON(http.StatusNotFound, gin.H{"error": "Template file not found: " + filename})
        return
    }
```

This row does not look like a real `rows.Close()` omission. The function only reads a JSON file and unmarshals it; there is no database `rows` handle in the code shown, so this is likely a heuristic misfire.

## Python DTO and API-shape warnings

### Representative snippet

```python
@dataclass
class SecurityConfig:
    enabled: bool = False
    user_password: str = ""
    owner_password: str = ""
    allow_printing: bool = True
    allow_modifying: bool = False
    allow_copying: bool = True
    allow_annotations: bool = False

    def to_dict(self) -> Dict[str, Any]:
        return _to_dict(self)
```

The repeated pattern in this file is a large set of option-bag models plus broad `to_dict()` return types. That is not an immediate runtime bug, but it does make the public shape harder to validate, type-check, and evolve safely.

### `option_bag_model`

| Line | Class | Why it matters |
| --- | --- | --- |
| 89 | `SecurityConfig` | Several permission toggles are packed into one config object. |
| 109 | `PDFAConfig` | A small feature flag object is easier to misuse when it grows optional fields. |
| 125 | `SignatureConfig` | Many boolean and optional switches create a combinatorial API surface. |
| 175 | `Config` | Page/layout/security toggles are bundled into one broad option bag. |
| 226 | `Cell` | Cell-level optional flags make the row content contract harder to reason about. |
| 262 | `Table` | Width, height, and color options are all optional and loosely coupled. |
| 340 | `Title` | Multiple optional presentation fields are encoded as a single bag of flags. |
| 379 | `PDFTemplate` | The top-level template model combines several optional substructures. |
| 425 | `HtmlToPDFRequest` | Request shape is mostly a collection of knobs and defaults. |
| 447 | `HtmlToImageRequest` | Same issue: many independent switches in one request object. |

### `public_any_type_leak`

| Line | Method | Why it matters |
| --- | --- | --- |
| 104 | `SecurityConfig.to_dict` | Public API returns `Dict[str, Any]`, which hides the contract from callers. |
| 120 | `PDFAConfig.to_dict` | Same broad dictionary contract. |
| 143 | `SignatureConfig.to_dict` | Same broad dictionary contract. |
| 155 | `CustomFontConfig.to_dict` | Same broad dictionary contract. |
| 170 | `Bookmark.to_dict` | Same broad dictionary contract. |
| 192 | `Config.to_dict` | Same broad dictionary contract. |
| 206 | `Image.to_dict` | Same broad dictionary contract. |
| 221 | `FormField.to_dict` | Same broad dictionary contract. |
| 243 | `Cell.to_dict` | Same broad dictionary contract. |
| 257 | `Row.to_dict` | Same broad dictionary contract. |
| 272 | `Table.to_dict` | Same broad dictionary contract. |
| 294 | `Spacer.to_dict` | Same broad dictionary contract. |
| 308 | `Element.to_dict` | Same broad dictionary contract. |
| 329 | `TitleTable.to_dict` | Same broad dictionary contract. |
| 350 | `Title.to_dict` | Same broad dictionary contract. |
| 371 | `Footer.to_dict` | Same broad dictionary contract. |
| 391 | `PDFTemplate.to_dict` | Same broad dictionary contract. |
| 420 | `FontInfo.to_dict` | Same broad dictionary contract. |
| 442 | `HtmlToPDFRequest.to_dict` | Same broad dictionary contract. |
| 464 | `HtmlToImageRequest.to_dict` | Same broad dictionary contract. |
| 476 | `SplitSpec.to_dict` | Same broad dictionary contract. |

## Shared mutable package state

Representative snippet:

```go
var LiberationFontMapping = map[string]string{
    "Helvetica":             "LiberationSans-Regular",
    "Helvetica-Bold":        "LiberationSans-Bold",
    "Helvetica-Oblique":     "LiberationSans-Italic",
    "Helvetica-BoldOblique": "LiberationSans-BoldItalic",
}

var LiberationFontFiles = map[string]string{
    "LiberationSans-Regular":    "LiberationSans-Regular.ttf",
    "LiberationSans-Bold":       "LiberationSans-Bold.ttf",
    "LiberationSans-Italic":     "LiberationSans-Italic.ttf",
    "LiberationSans-BoldItalic": "LiberationSans-BoldItalic.ttf",
}
```

```go
var pageSizes = map[string][2]float64{ ... }
var hexNibble [256]byte
```

These are classic shared-state smells. If any code mutates them after initialization, it creates hidden coupling and possible race conditions. If they are truly read-only after setup, the practical risk is lower, but the scanner is still flagging a maintenance hazard.

| File:line | Finding |
| --- | --- |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/font/pdfa.go:27` | `mutable_package_global` on `LiberationFontMapping` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/font/pdfa.go:48` | `mutable_package_global` on `LiberationFontFiles` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/types.go:9` | `mutable_package_global` on `pageSizes` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/utils.go:14` | `mutable_package_global` on `hexNibble` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/sampledata/gopdflib/zerodha/main.go:60` | `mutable_package_global` on `symbols` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/sampledata/gopdflib/zerodha/main.go:68` | `mutable_package_global` on `actions` |

## Boolean-parameter APIs

Representative snippet:

```go
func GetMappedFontName(standardFontName string, pdfaMode bool) string
func ParseLink(link string) (isExternal bool, uri string, dest string)
func NewPageManager(pageDims PageDimensions, margins PageMargins, arlingtonCompatible bool, fontRegistry *CustomFontRegistry) *PageManager
```

Boolean parameters are easy to misread at call sites and hard to extend when a second mode arrives. A named option struct or separate entry points is usually clearer.

| File:line | Finding |
| --- | --- |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/font/pdfa.go:392` | `public_bool_parameter_api` on `GetMappedFontName` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/links.go:80` | `public_bool_parameter_api` on `ParseLink` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/pagemanager.go:42` | `public_bool_parameter_api` on `NewPageManager` |

## Single-implementation interface

Representative snippet:

```go
type OCRProvider interface {
    ExtractWords(pdfBytes []byte, settings models.OCRSettings) ([]ocrWord, error)
}

type tesseractProvider struct{}

func getOCRProvider(settings models.OCRSettings) (OCRProvider, error) {
    provider := strings.TrimSpace(strings.ToLower(settings.Provider))
    if provider == "" || provider == "tesseract" {
        return tesseractProvider{}, nil
    }
```

There is only one obvious repository-local implementation, so the interface mostly adds indirection today. That is not wrong, but it is a sign the abstraction may be premature unless a second backend is expected soon.

| File:line | Finding |
| --- | --- |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/pdf/redact/ocr_adapter.go:29` | `single_impl_interface` on `OCRProvider` |

## Deferred cleanup inside loops

Representative snippet:

```go
for _, font := range mathFonts {
    if fontExistsOnSystem(font) {
        log.Printf("[fontutils] Font %s found on system", font.Name)
        continue
    }

    wg.Add(1)
    go func(f MathFontInfo) {
        defer wg.Done()
        if err := downloadFont(f); err != nil {
            log.Printf("[fontutils] WARNING: failed to download font %s: %v", f.Name, err)
        }
    }(font)
}
```

These rows are lower severity than a real resource leak, but the pattern can still be problematic if the loop starts deferring file closes, locks, or other cleanup that should happen sooner. Here the cleanup is mostly synchronization, so the practical risk is limited.

| File:line | Finding |
| --- | --- |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/pkg/fontutils/fontutils.go:149` | `defer_in_loop_resource_growth` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/sampledata/gopdflib/financial_report/main.go:88` | `defer_in_loop_resource_growth` |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/sampledata/gopdflib/zerodha/main.go:686` | `defer_in_loop_resource_growth` |

## Import-time file I/O

Representative snippet:

```python
DATA_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data.json")
with open(DATA_PATH, "r") as f:
    data = json.load(f)
```

Import-time I/O makes the module slower to load and introduces side effects just by importing it. That is usually a bad fit for reusable library code and test fixtures.

| File:line | Finding |
| --- | --- |
| `/home/chinmay/ChinmayPersonalProjects/gopdfsuit/sampledata/benchmarks/fpdf/bench.py:11` | `import_time_file_io` |

## Notes

- The wording-only changes in the temp report were ignored. The 46 rows above are only the findings that are truly new after comparing the two reports tuple-by-tuple.
- The biggest real change is the added coverage in `bindings/python/pypdfsuit/types.py`; the rest are isolated single-row additions.