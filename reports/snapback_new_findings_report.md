# SnapBack temp report delta

This report compares `/home/chinmay/ChinmayPersonalProjects/deslop/verified_snapback_results.txt` against `/home/chinmay/ChinmayPersonalProjects/deslop/temp_snapback.txt`.

## Diff Method

```bash
git diff --no-index --unified=0 -- /home/chinmay/ChinmayPersonalProjects/deslop/verified_snapback_results.txt /home/chinmay/ChinmayPersonalProjects/deslop/temp_snapback.txt \
  | grep '^+  - '
```

That raw diff produces 37 added finding rows. I did not count wording-only changes as separate rows.

## Summary

- Verified finding rows: 67
- Temp finding rows: 104
- Added finding rows: 37
- New file groups: `apps/api/main.py`, `services/analysis/detector.py`, `services/analysis/summarizer.py`, `services/exporters/export.py`, `services/storage/database.py`

## Why These Matter

`import_time_config_load` means the module reads configuration or builds runtime objects while it is being imported. That makes imports stateful, can surprise tests, and can trigger side effects before the app startup path runs.

`public_any_type_leak` means a public function or model exposes broad `Any`-shaped values. That hides the schema from callers, weakens static checking, and makes refactors harder to verify.

## Import-Time Config Load

Representative snippet:

```python
ROOT_DIR = Path(__file__).resolve().parents[2]
load_dotenv(ROOT_DIR / "config" / "env" / ".env")

AUTO_DELETE_AFTER_HOURS = int(os.getenv("AUTO_DELETE_AFTER_HOURS", "24"))
NOTION_API_KEY = os.getenv("NOTION_API_KEY", "")

scheduler = BackgroundScheduler()
summarizer = GroqSummarizer(api_key=os.getenv("GROQ_API_KEY"))
```

This is the core issue behind the new `import_time_config_load` rows in `apps/api/main.py`. Importing the module now loads environment state and constructs a summarizer immediately, which creates hidden coupling between import order, environment configuration, and runtime behavior.

| Line | Exact row | Why it matters |
| --- | --- | --- |
| 46 | `module loads configuration or secrets while being imported` | Import has a side effect. |
| 48 | `module loads configuration or secrets while being imported` | Repeated import-time config read. |
| 48 | `module loads configuration or secrets while being imported` | Repeated import-time config read. |
| 48 | `module initializes AUTO_DELETE_AFTER_HOURS from configuration or secrets at import time` | Binds config eagerly, before app startup. |
| 49 | `module loads configuration or secrets while being imported` | Repeated import-time config read. |
| 49 | `module initializes NOTION_API_KEY from configuration or secrets at import time` | Pulls a secret at import time. |
| 122 | `module loads configuration or secrets while being imported` | Repeated import-time config read. |
| 122 | `module loads configuration or secrets while being imported` | Repeated import-time config read. |
| 122 | `module initializes summarizer from configuration or secrets at import time` | Builds the client during module import. |

## Public Type-Contract Leaks

Representative snippet:

```python
@app.post("/session/start")
def start_session(payload: SessionStartRequest) -> dict[str, Any]:
    ...


@dataclass
class SessionBundle:
    session: dict[str, Any]
    transcript: list[dict[str, Any]]
    recaps: list[dict[str, Any]]
```

The same pattern repeats across the API, storage, export, and analysis layers. These are not immediate runtime failures, but they are a real contract risk: callers cannot rely on a precise schema, and the code becomes harder to evolve safely.

### `apps/api/main.py`

| Line | Symbol | Why it matters |
| --- | --- | --- |
| 159 | `start_session` | Returns `dict[str, Any]`, so the response shape is not explicit. |
| 174 | `ingest_transcript` | Returns `dict[str, Any]`, so the response shape is not explicit. |
| 188 | `ingest_audio_chunk` | Returns `dict[str, Any]`, so the response shape is not explicit. |
| 220 | `generate_recap` | Returns `dict[str, Any]`, which hides a large response contract. |
| 283 | `get_session_transcript` | Returns `dict[str, Any]`, which hides a large response contract. |
| 296 | `complete_session` | Returns `dict[str, Any]`, which hides a large response contract. |
| 362 | `export_notion` | Returns `dict[str, Any]`, which hides a large response contract. |
| 382 | `generate_study_pack` | Returns `dict[str, Any]`, which hides a large response contract. |

### `services/analysis/detector.py`

| Line | Symbol | Why it matters |
| --- | --- | --- |
| 47 | `detect_missed_alerts` | Takes and returns `Any`-shaped dict data, so the alert schema is weakly specified. |

### `services/analysis/summarizer.py`

| Line | Symbol | Why it matters |
| --- | --- | --- |
| 146 | `generate_study_pack` | Returns `dict[str, Any]` from model output or fallback data, so the contract is broad. |

### `services/exporters/export.py`

| Line | Symbol | Why it matters |
| --- | --- | --- |
| 15 | `build_markdown_export` | Accepts `dict[str, Any]`, so the bundle shape is loosely defined. |
| 62 | `build_pdf_export` | Accepts `dict[str, Any]`, so the bundle shape is loosely defined. |
| 122 | `export_to_notion` | Accepts `dict[str, Any]`, so the bundle shape is loosely defined. |

### `services/storage/database.py`

| Line | Symbol | Why it matters |
| --- | --- | --- |
| 99 | `create_session` | Returns `dict[str, Any]`, so the session schema is not strongly typed. |
| 121 | `get_session` | Returns `dict[str, Any] | None`, so the session schema is not strongly typed. |
| 131 | `end_session` | Returns `dict[str, Any] | None`, so the session schema is not strongly typed. |
| 146 | `append_transcript_chunk` | Returns `dict[str, Any]`, so the chunk schema is not strongly typed. |
| 173 | `get_transcript` | Returns `list[dict[str, Any]]`, so the chunk schema is not strongly typed. |
| 188 | `get_transcript_window` | Returns `list[dict[str, Any]]`, so the chunk schema is not strongly typed. |
| 209 | `get_last_chunk_before` | Returns `dict[str, Any] | None`, so the chunk schema is not strongly typed. |
| 226 | `get_first_chunk_after` | Returns `dict[str, Any] | None`, so the chunk schema is not strongly typed. |
| 243 | `save_recap` | Returns `dict[str, Any]`, so the recap schema is not strongly typed. |
| 284 | `get_recaps` | Returns `list[dict[str, Any]]`, so the recap schema is not strongly typed. |
| 298 | `hydrate_recap` | Accepts and returns `dict[str, Any]`, so the recap schema stays broad. |
| 306 | `save_audio_chunk` | Returns `dict[str, Any]`, so the audio chunk schema is not strongly typed. |
| 395 | `SessionBundle.session` | The bundle exposes a very broad session field. |
| 396 | `SessionBundle.transcript` | The bundle exposes a very broad transcript field. |
| 397 | `SessionBundle.recaps` | The bundle exposes a very broad recaps field. |

## What I Checked

- I compared the verified and temp reports with `git diff --no-index`.
- I used the raw diff additions to keep repeated rows on the same line.
- The temp report adds only two issue families: import-time configuration loading and broad public type contracts.

## Notes

- No obvious resource-leak, concurrency, or error-handling additions appeared in this delta.
- The main behavioral concern is the import-time side effect in `apps/api/main.py`; the rest are mostly API-contract and maintainability issues.