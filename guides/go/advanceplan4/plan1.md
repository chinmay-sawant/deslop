# Plan 1 — Low-Level Performance Worst Practices (Go)

Date: 2026-04-03

## Status

- [x] Implemented on 2026-04-03.
- [x] All 50 plan1 performance rules are now shipped in `src/heuristics/go/advanceplan4/performance.rs`.
- [x] Grouped positive and clean fixture coverage ships in `tests/fixtures/go/advanceplan4_perf_{positive,clean}.txt`.
- [x] Integration verification ships in `tests/integration_scan/go_advanceplan4.rs`.
- [x] The detailed rule bullets below remain as the drafting inventory; the shipped status above is the source of truth.

## Already Covered And Excluded From This Plan

- [x] `allocation_churn_in_loop` — advanceplan1
- [x] `string_concat_in_loop` — advanceplan1
- [x] `fmt_hot_path` — advanceplan1
- [x] `reflection_hot_path` — advanceplan1
- [x] `repeated_json_marshaling` — advanceplan1
- [x] `likely_n_squared_allocation` — advanceplan1
- [x] `likely_n_squared_string_concat` — advanceplan1
- [x] `regexp_compile_in_hot_path` — advanceplan3/plan1
- [x] `template_parse_in_hot_path` — advanceplan3/plan1
- [x] `builder_or_buffer_recreated_per_iteration` — advanceplan3/plan1
- [x] `make_slice_inside_hot_loop_same_shape` — advanceplan3/plan1
- [x] `byte_string_conversion_in_loop` — advanceplan3/plan1
- [x] `slice_membership_in_loop_map_candidate` — advanceplan3/plan1
- [x] `time_parse_layout_in_loop` — advanceplan3/plan1
- [x] All SQL/GORM/Gin rules — advanceplan3/plan2, plan3

## Objective

Build a pack of 50 low-level Go performance worst practices that target measurable CPU-cycle, allocation, and memory-bandwidth waste in hot paths. Each rule should explain the concrete cost difference and suggest the cheaper alternative. The emphasis is on patterns that static heuristics can detect with high confidence.

## Phase Completion

- [x] Section A shipped all 12 string and byte-operation rules.
- [x] Section B shipped all 13 slice/map rules.
- [x] Section C shipped all 10 runtime/sync rules.
- [x] Section D shipped all 8 I/O/encoding rules.
- [x] Section E shipped all 7 error/interface rules.

---

## Section A — String And Byte Operations (12 rules)

### A1. `strings_contains_vs_index`
- [ ] Detect `strings.Index(s, sub) != -1` or `strings.Index(s, sub) >= 0` patterns.
- **Why**: `strings.Contains` short-circuits identically but avoids the comparison boilerplate and communicates intent. No cycle difference, but prevents bugs and improves readability in hot paths.
- **Use this**: `strings.Contains(s, sub)`
- **Instead of**: `strings.Index(s, sub) != -1`

### A2. `string_to_byte_for_single_char_check`
- [ ] Detect `[]byte(s)[0]` or `string(b) == "x"` for single-character comparisons.
- **Why**: `s[0]` directly accesses the byte without allocation. Converting `string→[]byte` allocates a copy (~24 bytes + len). For single-char checks, direct byte indexing avoids the allocation entirely.
- **Use this**: `s[0] == 'x'` or `bytes.Equal`
- **Instead of**: `string(b) == "x"` — 1 alloc vs 0 allocs

### A3. `string_concatenation_for_path_join`
- [ ] Detect `dir + "/" + file` or manual path assembly via `+` concatenation.
- **Why**: `filepath.Join` handles separators correctly and uses a builder internally (~1 allocation). Manual `+` chains create N-1 intermediate strings. For 3 segments: 2 allocs vs 1 alloc.
- **Use this**: `filepath.Join(dir, file)`
- **Instead of**: `dir + "/" + file`

### A4. `sprintf_for_simple_int_to_string`
- [ ] Detect `fmt.Sprintf("%d", n)` where `n` is clearly an integer type.
- **Why**: `fmt.Sprintf` uses reflection internally (~3 allocs, ~200ns). `strconv.Itoa(n)` uses a lookup table and direct formatting (~1 alloc, ~30ns). ~6× faster.
- **Use this**: `strconv.Itoa(n)` — ~30ns, 1 alloc
- **Instead of**: `fmt.Sprintf("%d", n)` — ~200ns, 3 allocs

### A5. `sprintf_for_simple_string_format`
- [ ] Detect `fmt.Sprintf("%s:%s", a, b)` where only `%s` verbs are used.
- **Why**: `fmt.Sprintf` reflectively inspects each argument. For pure string concatenation, `a + ":" + b` or `strings.Join` avoids reflection entirely. ~4× faster for 2-3 segments.
- **Use this**: `a + ":" + b` or `strings.Join([]string{a, b}, ":")`
- **Instead of**: `fmt.Sprintf("%s:%s", a, b)` — saves ~150ns per call

### A6. `strings_replace_all_for_single_char`
- [ ] Detect `strings.ReplaceAll(s, "x", "y")` where both old and new are single characters.
- **Why**: `strings.Map` with a character switch avoids substring search overhead. For single-char replacement, `strings.Map` is ~2× faster because it avoids the Rabin-Karp search setup.
- **Use this**: `strings.Map(func(r rune) rune { if r == 'x' { return 'y' }; return r }, s)`
- **Instead of**: `strings.ReplaceAll(s, "x", "y")` for single-char pairs

### A7. `repeated_string_trim_normalize`
- [ ] Detect chains like `strings.TrimSpace(strings.ToLower(strings.TrimPrefix(s, ...)))` that scan the string multiple times.
- **Why**: Each function scans the entire string. Chaining 3 operations means 3 full scans. A single-pass custom normalizer or combined `strings.Map` does 1 scan. ~3× less memory bandwidth.
- **Use this**: Single-pass normalizer or combined `strings.Map`
- **Instead of**: Chained trim/lower/replace — N scans vs 1 scan

### A8. `len_string_for_empty_check`
- [ ] Detect `len(s) == 0` used interchangeably with `s == ""`.
- **Why**: Both compile to the same machine code. However, `s == ""` communicates intent better for empty-string checks. `len(s) > 0` is preferred when checking non-empty because it avoids a string comparison. No cycle difference but consistency helps reviewers.
- **Use this**: `s == ""` for empty checks; `len(s) > 0` for non-empty checks
- **Instead of**: Mixed usage in the same file

### A9. `string_format_for_error_wrap`
- [ ] Detect `fmt.Errorf("failed: %s", err.Error())` where `%s` on `err.Error()` is used instead of `%w` on `err`.
- **Why**: `err.Error()` forces a string allocation of the error message. `%w` wraps the error without materializing the string until needed, and preserves the error chain for `errors.Is`/`errors.As`. Saves 1 alloc + preserves unwrap.
- **Use this**: `fmt.Errorf("failed: %w", err)` — 0 extra string allocs, preserves chain
- **Instead of**: `fmt.Errorf("failed: %s", err.Error())` — 1 extra string alloc, breaks chain

### A10. `strings_hasprefix_then_trimprefix`
- [ ] Detect `if strings.HasPrefix(s, p) { s = strings.TrimPrefix(s, p) }`.
- **Why**: Both `HasPrefix` and `TrimPrefix` scan the prefix. `strings.CutPrefix` (Go 1.20+) does both in one scan. ~2× fewer comparisons for the prefix bytes.
- **Use this**: `after, found := strings.CutPrefix(s, p)` — 1 prefix scan
- **Instead of**: `HasPrefix` + `TrimPrefix` — 2 prefix scans

### A11. `strings_hassuffix_then_trimsuffix`
- [ ] Detect `if strings.HasSuffix(s, p) { s = strings.TrimSuffix(s, p) }`.
- **Why**: Same as A10 but for suffixes. `strings.CutSuffix` (Go 1.20+) does both in one scan.
- **Use this**: `before, found := strings.CutSuffix(s, p)` — 1 suffix scan
- **Instead of**: `HasSuffix` + `TrimSuffix` — 2 suffix scans

### A12. `string_builder_write_string_vs_plus`
- [ ] Detect `builder.WriteString(a + b)` where `a` and `b` are separate bindings.
- **Why**: `a + b` creates a temporary concatenated string (1 alloc). Two separate `WriteString` calls write directly to the builder's buffer with 0 intermediate allocs. Saves 1 alloc per call.
- **Use this**: `builder.WriteString(a); builder.WriteString(b)` — 0 intermediate allocs
- **Instead of**: `builder.WriteString(a + b)` — 1 intermediate alloc

---

## Section B — Slice And Map Operations (13 rules)

### B1. `copy_append_idiom_waste`
- [ ] Detect `dst = append(dst, src...)` when `dst` is known empty and `len(src)` is known.
- **Why**: `append` on a nil/empty slice forces a grow+copy cycle. `make([]T, len(src))` + `copy` avoids the grow heuristic and allocates exactly once. ~1.5× fewer allocs for small-to-medium slices.
- **Use this**: `dst := make([]T, len(src)); copy(dst, src)`
- **Instead of**: `dst = append([]T(nil), src...)` when size is known

### B2. `map_delete_in_loop_vs_new_map`
- [ ] Detect `for k := range m { delete(m, k) }` patterns.
- **Why**: Deleting every key iteratively is O(n) with per-key overhead (~50ns per `delete`). Creating a new `make(map[K]V)` is O(1) and lets the GC reclaim the old map. For maps > ~100 entries, new map is ~10× faster.
- **Use this**: `m = make(map[K]V, hint)` — O(1)
- **Instead of**: Loop `delete(m, k)` — O(n) × ~50ns per key

### B3. `sort_slice_vs_sort_sort`
- [ ] Detect `sort.Sort(sort.StringSlice(s))` or custom `sort.Interface` implementations for basic types.
- **Why**: `sort.Strings(s)` or `slices.Sort(s)` (Go 1.21+) avoids the interface dispatch overhead. `sort.Sort` uses 3 virtual calls per comparison. `slices.Sort` uses generics with zero interface overhead. ~20-30% faster for large slices.
- **Use this**: `slices.Sort(s)` (Go 1.21+) or `sort.Strings(s)`
- **Instead of**: `sort.Sort(sort.StringSlice(s))` — 3 virtual calls per comparison

### B4. `range_over_string_by_index`
- [ ] Detect `for i := 0; i < len(s); i++ { c := s[i] }` on strings that should iterate runes.
- **Why**: Index-based byte iteration on UTF-8 strings misses multi-byte runes. `for _, r := range s` correctly decodes runes. If only byte access is needed, cast to `[]byte` once. The index loop is not faster and introduces correctness bugs on non-ASCII.
- **Use this**: `for _, r := range s` for rune iteration
- **Instead of**: `for i := 0; i < len(s); i++ { s[i] }` — mishandles multi-byte runes

### B5. `map_lookup_double_access`
- [ ] Detect `if _, ok := m[k]; ok { v := m[k] }` — two map lookups for the same key.
- **Why**: Each map lookup is ~100-200ns depending on key size. The comma-ok idiom `v, ok := m[k]` does one lookup. Saves 1 hash + 1 probe (~150ns per access).
- **Use this**: `v, ok := m[k]; if ok { ... use v ... }` — 1 lookup
- **Instead of**: `_, ok := m[k]; ok { v := m[k] }` — 2 lookups

### B6. `slice_grow_without_cap_hint`
- [ ] Detect `var result []T` followed by `append` in a loop where the iteration count is visible from a `len()` or range source.
- **Why**: Without `make([]T, 0, len(source))`, Go's append doubles the backing array on each grow. For 1000 items: ~10 reallocs + copies vs 1 alloc. ~5–10× more GC pressure on medium slices.
- **Use this**: `result := make([]T, 0, len(source))` — 1 alloc
- **Instead of**: `var result []T` + loop append — ~log₂(n) reallocs

### B7. `interface_slice_allocation`
- [ ] Detect `[]interface{}` or `[]any` used to pass homogeneous typed data.
- **Why**: Each element in `[]any` requires a 2-word interface header (16 bytes) + potential heap escape of the value. For `[]int` the values are stored inline. 1000 ints: `[]any` = 24KB + 1000 heap allocs; `[]int` = 8KB, 0 heap allocs.
- **Use this**: Typed slices `[]int`, `[]string`, generic `[]T`
- **Instead of**: `[]any` for homogeneous data — 3× memory + N heap allocs

### B8. `map_of_slices_prealloc`
- [ ] Detect `m[k] = append(m[k], v)` in loops without pre-allocating inner slices.
- **Why**: Each `m[k] = append(m[k], v)` copies the slice header back to the map entry. If the inner slice grows, the old backing array is wasted. Pre-grouping with known sizes or using `make([]T, 0, hint)` for inner slices reduces reallocs.
- **Use this**: Pre-group keys first, then `make([]T, 0, count)` for each group
- **Instead of**: Blind `m[k] = append(m[k], v)` — O(n) wasted intermediate arrays

### B9. `clear_map_go121`
- [ ] Detect `for k := range m { delete(m, k) }` in Go 1.21+ codebases.
- **Why**: Go 1.21 introduced `clear(m)` which resets the map in a single runtime call without per-key delete overhead. ~10× faster than iterative delete for maps > 50 entries.
- **Use this**: `clear(m)` (Go 1.21+) — single runtime call
- **Instead of**: `for k := range m { delete(m, k) }` — O(n) deletes

### B10. `unnecessary_slice_copy_for_readonly`
- [ ] Detect `copy := append([]T(nil), original...)` when `copy` is only read, never mutated.
- **Why**: If the copy is read-only, the original slice can be used directly. The `append`-clone allocates a new backing array and copies all elements. For 1000 elements of 8 bytes: 8KB wasted + 1 alloc.
- **Use this**: Direct reference to `original` when only reading
- **Instead of**: `append([]T(nil), original...)` — wastes len(original)×sizeof(T) bytes

### B11. `three_index_slice_for_append_safety`
- [ ] Detect `sub := original[a:b]` followed by `sub = append(sub, ...)` with no capacity bound.
- **Why**: Without the three-index slice `original[a:b:b]`, `append` can overwrite elements in `original` beyond `b`. The three-index form caps the capacity so `append` always allocates a new backing array. Not a performance win but prevents subtle data corruption bugs.
- **Use this**: `sub := original[a:b:b]` when appending to sub-slices
- **Instead of**: `sub := original[a:b]` + `append` — risks corrupting original

### B12. `range_copy_large_struct`
- [ ] Detect `for _, v := range largeStructSlice` where the struct is > 64 bytes.
- **Why**: `range` copies each element to the loop variable. For a 256-byte struct over 1000 elements: 256KB of copies. Using `for i := range slice { v := &slice[i] }` or `for i := range slice` with index access avoids the copy entirely.
- **Use this**: `for i := range s { use &s[i] }` — 0 copies
- **Instead of**: `for _, v := range s` for large structs — sizeof(T)×n bytes copied

### B13. `unnecessary_map_for_set_of_ints`
- [ ] Detect `map[int]bool` or `map[int]struct{}` used as a set for small dense integer ranges.
- **Why**: For dense integer ranges (0-N), a `[]bool` or bitset is dramatically faster. Map overhead per entry: ~120 bytes (hash bucket + key + value + overflow pointer). `[]bool` per entry: 1 byte. For 1000 entries: `map` = ~120KB; `[]bool` = 1KB. ~100× less memory.
- **Use this**: `[]bool` or bitset for dense integer sets — 1 byte per entry
- **Instead of**: `map[int]struct{}` — ~120 bytes per entry for dense ranges

---

## Section C — Runtime And Sync Primitives (10 rules)

### C1. `sync_mutex_for_atomic_counter`
- [ ] Detect `mu.Lock(); count++; mu.Unlock()` for simple integer counters.
- **Why**: `sync/atomic.AddInt64` is lock-free and takes ~5ns. `Mutex.Lock()` + `Unlock()` takes ~25ns uncontended, ~1μs+ contended. ~5× faster uncontended, ~200× faster under contention.
- **Use this**: `atomic.AddInt64(&count, 1)` — ~5ns, lock-free
- **Instead of**: `mu.Lock(); count++; mu.Unlock()` — ~25ns uncontended

### C2. `sync_mutex_for_readonly_config`
- [ ] Detect `mu.RLock(); v := config.X; mu.RUnlock()` for read-mostly config that changes rarely.
- **Why**: `atomic.Value` or `sync.Map` with `Load`/`Store` for read-mostly data avoids all lock overhead on the read path. `atomic.Value.Load()` is ~2ns. `RWMutex.RLock()` is ~15ns. For 99% read workloads: ~7× faster reads.
- **Use this**: `atomic.Value` with `Load()`/`Store()` — ~2ns reads
- **Instead of**: `RWMutex.RLock()/RUnlock()` — ~15ns reads

### C3. `sync_pool_ignored_for_frequent_small_allocs`
- [ ] Detect repeated `make([]byte, size)` or `new(T)` in hot paths where the object is short-lived and could be pooled.
- **Why**: `sync.Pool.Get()` + `Put()` reuses objects across GC cycles. For a 4KB buffer allocated 10K times/sec: without pool = 40MB/sec allocation pressure; with pool = near-zero allocation after warmup. ~10× less GC pressure.
- **Use this**: `sync.Pool` for short-lived, frequently allocated objects
- **Instead of**: `make([]byte, 4096)` per request — 1 alloc vs 0 allocs (amortized)

### C4. `mutex_value_receiver`
- [ ] Detect `func (s MyStruct) Method()` where `MyStruct` contains a `sync.Mutex` or `sync.RWMutex` field.
- **Why**: Value receivers copy the struct including the mutex. A copied mutex is unlocked regardless of the original's state — this is a data race, not just a performance issue. The Go vet tool catches this but codegen often misses it.
- **Use this**: `func (s *MyStruct) Method()` — pointer receiver
- **Instead of**: `func (s MyStruct) Method()` — copies mutex, creates race

### C5. `time_now_in_tight_loop`
- [ ] Detect `time.Now()` called on every iteration of a tight inner loop.
- **Why**: `time.Now()` makes a VDSO system call (~20ns on Linux, ~50ns on macOS). In tight loops doing < 100ns of work, this dominates. Cache the timestamp outside the loop if millisecond precision is sufficient.
- **Use this**: `start := time.Now()` before loop; use `start` inside
- **Instead of**: `time.Now()` per iteration — ~20-50ns syscall overhead each time

### C6. `defer_in_tight_loop`
- [ ] Detect `defer` statements inside loops with > 100 iterations or visible hot-path markers.
- **Why**: `defer` is not free — each deferred call costs ~35ns and allocates a `_defer` struct. In a 10K-iteration loop: 350μs of defer overhead. Move the deferred work to a helper function so defer runs once per call.
- **Use this**: Extract loop body to `func() { defer cleanup(); ... }` — 1 defer per call
- **Instead of**: `for { defer cleanup() }` — N defers accumulated until function exit

### C7. `select_with_single_case`
- [ ] Detect `select { case v := <-ch: ... }` with only one case and no default.
- **Why**: Single-case `select` has unnecessary runtime overhead vs direct channel receive `v := <-ch`. The `select` machinery costs ~50ns for runtime channel registration. Direct receive is ~30ns. ~40% overhead for no benefit.
- **Use this**: `v := <-ch` — ~30ns direct receive
- **Instead of**: `select { case v := <-ch: }` — ~50ns select overhead

### C8. `goroutine_for_sync_work`
- [ ] Detect `go func() { result <- compute() }()` followed by `<-result` where the goroutine is immediately awaited.
- **Why**: Spawning a goroutine for synchronous work costs ~1μs (stack allocation + scheduler). Direct function call: ~5ns. If the caller immediately waits, there's no parallelism gained. ~200× overhead.
- **Use this**: `result := compute()` — ~5ns function call
- **Instead of**: `go func()` + `<-chan` — ~1μs goroutine spawn, 0 parallelism gained

### C9. `unbuffered_channel_for_known_producer_count`
- [ ] Detect unbuffered channels `make(chan T)` when the number of producers/messages is known at construction time.
- **Why**: Unbuffered channels force goroutine synchronization on every send (~100ns blocked wait). Buffered channels `make(chan T, n)` allow producers to proceed without blocking. For 100 known messages: unbuffered = 100 goroutine park/unpark cycles; buffered = 0 if buffer ≥ 100.
- **Use this**: `make(chan T, n)` where `n` bounds expected messages
- **Instead of**: `make(chan T)` when message count is known — saves n park/unpark cycles

### C10. `waitgroup_add_inside_loop`
- [ ] Detect `for { wg.Add(1); go func() { ... wg.Done() }() }` where `wg.Add` could be called once before the loop with the count.
- **Why**: `wg.Add(1)` per iteration uses an atomic CAS (~5ns) and introduces a happens-before edge each time. `wg.Add(n)` before the loop does one atomic operation. For 1000 iterations: 1000 atomics vs 1 atomic.
- **Use this**: `wg.Add(len(items))` before loop — 1 atomic op
- **Instead of**: `wg.Add(1)` per iteration — n atomic ops

---

## Section D — I/O And Encoding (8 rules)

### D1. `ioutil_readall_still_used`
- [ ] Detect `ioutil.ReadAll` usage when `io.ReadAll` is available (Go 1.16+).
- **Why**: `ioutil.ReadAll` is deprecated since Go 1.16 and is a thin wrapper around `io.ReadAll`. No performance difference but signals stale code and may trigger deprecation warnings. The `ioutil` package is frozen.
- **Use this**: `io.ReadAll(r)` — canonical since Go 1.16
- **Instead of**: `ioutil.ReadAll(r)` — deprecated wrapper

### D2. `json_marshal_then_write`
- [ ] Detect `data, _ := json.Marshal(v); w.Write(data)` when `json.NewEncoder(w).Encode(v)` would stream directly.
- **Why**: `json.Marshal` allocates a `[]byte` buffer, serializes, then copies to the writer. `json.NewEncoder` writes directly to the writer with an internal buffer. Saves 1 alloc + 1 copy for the full payload size. For a 10KB response: saves 10KB allocation.
- **Use this**: `json.NewEncoder(w).Encode(v)` — streams directly
- **Instead of**: `json.Marshal(v)` + `w.Write(data)` — 1 extra alloc of full payload size

### D3. `binary_read_for_single_field`
- [ ] Detect `binary.Read(r, order, &singleField)` for reading a single integer.
- **Why**: `binary.Read` uses reflection to determine the field size (~200ns). `binary.BigEndian.Uint32(buf)` or direct byte manipulation is ~2ns. ~100× faster for single-field reads.
- **Use this**: `binary.BigEndian.Uint32(buf[:4])` — ~2ns, 0 allocs
- **Instead of**: `binary.Read(r, binary.BigEndian, &val)` — ~200ns, uses reflection

### D4. `json_number_vs_float64_decode`
- [ ] Detect `json.Unmarshal` into `map[string]any` for numeric data without `UseNumber()`.
- **Why**: By default, JSON numbers decode as `float64`, losing precision for int64 values > 2^53. `decoder.UseNumber()` decodes as `json.Number` (a string), preserving precision. Not a CPU cost issue but a data-correctness issue in financial/ID applications.
- **Use this**: `decoder.UseNumber()` when integer precision matters
- **Instead of**: Default `float64` decoding — lossy for large integers

### D5. `xml_decoder_without_strict`
- [ ] Detect `xml.NewDecoder(r)` without setting `Strict = false` when processing trusted XML.
- **Why**: Strict XML parsing validates entity references and namespace correctness (~15% overhead). For trusted internal XML feeds, `d.Strict = false` skips validation. Saves ~15% decode time on large documents.
- **Use this**: `d.Strict = false` for trusted XML sources — ~15% faster
- **Instead of**: Default strict parsing for internal/trusted feeds

### D6. `csv_reader_reuse_record`
- [ ] Detect `csv.NewReader(r)` without `ReuseRecord = true` when records are processed one at a time and not stored.
- **Why**: By default, `csv.Reader.Read()` allocates a new `[]string` per row. `ReuseRecord = true` reuses the same slice, reducing GC pressure. For a 100K-row CSV: 100K allocs vs 1 alloc.
- **Use this**: `reader.ReuseRecord = true` — 1 alloc total
- **Instead of**: Default per-row allocation — N allocs

### D7. `bufio_scanner_small_buffer_for_large_lines`
- [ ] Detect `bufio.NewScanner(r)` without `scanner.Buffer()` when processing files with lines > 64KB.
- **Why**: Default scanner buffer is 64KB. Lines exceeding this cause `bufio.ErrTooLong`. `scanner.Buffer(buf, maxSize)` allows scanning larger lines without error. Not a performance issue per se but causes silent data truncation.
- **Use this**: `scanner.Buffer(make([]byte, 0, maxSize), maxSize)` for large-line files
- **Instead of**: Default 64KB buffer — silently fails on large lines

### D8. `http_body_readall_without_limitreader`
- [ ] Detect `io.ReadAll(req.Body)` in HTTP handlers without `io.LimitReader`.
- **Why**: Without `LimitReader`, a malicious client can send a multi-GB body and OOM the server. `io.LimitReader(req.Body, maxBytes)` caps memory usage. Not a CPU-cycle issue but a critical memory safety / DoS concern.
- **Use this**: `io.ReadAll(io.LimitReader(req.Body, maxBytes))` — bounded memory
- **Instead of**: `io.ReadAll(req.Body)` — unbounded memory, DoS vector

---

## Section E — Error Handling And Interface Patterns (7 rules)

### E1. `type_assertion_without_comma_ok`
- [ ] Detect `v := i.(T)` without the comma-ok form in non-panic-safe code.
- **Why**: The single-return type assertion panics if the type doesn't match. `v, ok := i.(T)` returns false instead. Not a CPU issue but causes runtime panics in production. The comma-ok form has identical cost (~5ns).
- **Use this**: `v, ok := i.(T)` — safe, same cost
- **Instead of**: `v := i.(T)` — panics on type mismatch

### E2. `type_switch_vs_repeated_assertions`
- [ ] Detect multiple sequential `if _, ok := i.(T1); ok { ... } else if _, ok := i.(T2); ok { ... }` patterns.
- **Why**: A `switch v := i.(type)` compiles to a single type-switch jump table. Sequential assertions each perform a separate runtime type check (~5ns each). For 5 types: switch = 1 dispatch; sequential = up to 5 checks.
- **Use this**: `switch v := i.(type) { case T1: ... case T2: ... }` — 1 dispatch
- **Instead of**: Sequential `if _, ok := i.(T); ok` — up to N runtime checks

### E3. `errors_new_for_static_sentinel`
- [ ] Detect `errors.New("some error")` called repeatedly in hot paths instead of a package-level sentinel.
- **Why**: `errors.New` allocates a new `errorString` struct each time (~1 alloc, ~40 bytes). A package-level `var ErrFoo = errors.New("...")` allocates once at init. For 10K errors/sec: 10K allocs vs 0.
- **Use this**: `var ErrFoo = errors.New("...")` at package level — 1 alloc total
- **Instead of**: `errors.New("...")` inline — 1 alloc per call

### E4. `fmt_errorf_without_wrap_verb`
- [ ] Detect `fmt.Errorf("context: %v", err)` instead of `%w`.
- **Why**: `%v` stringifies the error (~1 alloc for the string). `%w` wraps without stringifying and preserves `errors.Is`/`errors.As` chains. Same formatting cost but preserves the error chain.
- **Use this**: `fmt.Errorf("context: %w", err)` — wraps, preserves chain
- **Instead of**: `fmt.Errorf("context: %v", err)` — stringifies, breaks chain

### E5. `error_string_comparison`
- [ ] Detect `if err.Error() == "some error"` string comparison for error checking.
- **Why**: `err.Error()` allocates a string (~1 alloc). String comparison is O(n) on the message length. `errors.Is(err, ErrSentinel)` does pointer comparison (~2ns). ~10× faster and semantically correct across wrapping.
- **Use this**: `errors.Is(err, ErrSentinel)` — ~2ns, 0 allocs
- **Instead of**: `err.Error() == "..."` — 1 alloc + O(n) string compare

### E6. `empty_interface_parameter_overuse`
- [ ] Detect exported functions with `any` or `interface{}` parameters when concrete types would suffice.
- **Why**: `interface{}` parameters force heap escape of the passed value (~1 alloc per call for non-pointer types). Concrete-typed parameters allow stack allocation. For int/float64 arguments: interface = 1 heap alloc; concrete = 0 allocs.
- **Use this**: Concrete types or generics `func Foo[T any](v T)` — stack allocated
- **Instead of**: `func Foo(v any)` — heap-escapes the value

### E7. `panic_for_expected_errors`
- [ ] Detect `panic()` used for expected error conditions like invalid input or missing config.
- **Why**: `panic` + `recover` costs ~1μs for the stack unwinding. Returning an error costs ~5ns. ~200× more expensive. Panics also crash the process if not recovered, making them inappropriate for expected failures.
- **Use this**: Return `error` — ~5ns, caller handles gracefully
- **Instead of**: `panic("invalid input")` — ~1μs, process crash risk

---

## Shared Implementation Checklist

- [x] Implement each rule family as a function in `src/heuristics/go/advanceplan4/` following the existing pattern from `advanceplan3`.
- [x] Use `body_lines()` with `in_loop` tracking for hot-path-sensitive rules.
- [x] Use `import_aliases_for()` to resolve stdlib package aliases.
- [x] Default to `Info` severity for micro-optimization candidates; use `Warning` only when the cost difference is > 5×.
- [x] Skip test files via `is_test_file` / `is_test_function` suppression in the rule pack entrypoint.
- [x] Add one positive and one clean fixture for every rule section before enabling.
- [x] Verify the full plan1 pack with `cargo test go_advanceplan4 -- --nocapture`.

## Acceptance Criteria

- [x] Every shipped rule includes a concrete "use this / instead of this" with approximate cycle or allocation costs.
- [x] Clean fixtures for correct usage patterns stay quiet.
- [x] Rules remain function-local and parser-driven without requiring type checking or SSA.
- [x] No rule claims precise benchmarks — all cost numbers are approximate and documented as such.
