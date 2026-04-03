# Plan 3 — Library-Specific Performance And Security Worst Practices (Go)

Date: 2026-04-03

## Status

- [ ] Draft created on 2026-04-03.
- [ ] This plan covers worst practices in popular Go libraries: Gin, GORM, gRPC, Redis, Kafka, AWS SDK, Cobra, Viper, Zap/Logrus, and Prometheus.
- [ ] Every rule is detectable via static heuristics using import resolution and function-local analysis.
- [ ] Intentionally excludes rules already shipped in `advanceplan1`–`advanceplan3` for Gin and GORM.

## Already Covered And Excluded From This Plan

- [x] All advanceplan3/plan2 GORM rules (50 scenarios)
- [x] All advanceplan3/plan3 Gin rules (34 scenarios)
- [x] `http_client_without_timeout`, `http_server_without_timeouts` — advanceplan2
- [x] `sql_string_concat` — security.rs

## Objective

Build a pack of 30 library-specific worst practice detections targeting the most commonly used Go libraries beyond Gin and GORM (which are already well-covered). Focus on patterns that cause measurable performance degradation, resource leaks, or security vulnerabilities in production.

---

## Section A — Redis Client Worst Practices (6 rules)

### A1. `redis_ping_per_request`
- [ ] Detect `rdb.Ping(ctx)` inside handler functions or loops.
- **Why**: Redis PING per request adds a round-trip (~0.5ms) before the actual operation. The connection pool already validates connections. For 1000 req/sec: 500ms of pure PING overhead.
- **Use this**: Rely on pool health checks; ping only during startup/health endpoints
- **Instead of**: `rdb.Ping(ctx)` in every handler — 1 extra RTT per request

### A2. `redis_get_set_without_pipeline`
- [ ] Detect multiple sequential `rdb.Get(ctx, key)` or `rdb.Set(ctx, key, val, ...)` calls in the same function without pipeline.
- **Why**: Each Redis command is a network round-trip (~0.5ms). 5 sequential GETs = 2.5ms. Pipelining batches them into 1 RTT: ~0.5ms total. ~5× faster.
- **Use this**: `pipe := rdb.Pipeline(); pipe.Get(ctx, k1); pipe.Get(ctx, k2); pipe.Exec(ctx)` — 1 RTT
- **Instead of**: Sequential `rdb.Get()` calls — N RTTs

### A3. `redis_keys_command_in_handler`
- [ ] Detect `rdb.Keys(ctx, pattern)` in handler or loop code.
- **Why**: `KEYS` scans the entire keyspace, blocking Redis for O(n) time. For 1M keys: ~1 second block. `SCAN` iterates incrementally without blocking.
- **Use this**: `rdb.Scan(ctx, cursor, pattern, count)` — non-blocking iteration
- **Instead of**: `rdb.Keys(ctx, "*")` — blocks Redis for O(n)

### A4. `redis_connection_per_request`
- [ ] Detect `redis.NewClient(&redis.Options{...})` inside handler/request functions.
- **Why**: Creating a new Redis client per request creates a new connection pool per request (~1ms connection setup + pool overhead). Should be created once at startup.
- **Use this**: Package-level `var rdb = redis.NewClient(opts)` — single pool
- **Instead of**: `redis.NewClient()` per request — N pools, connection exhaustion

### A5. `redis_large_value_without_compression`
- [ ] Detect `rdb.Set(ctx, key, largePayload, ...)` where `largePayload` is the result of `json.Marshal` on a large struct or slice, without visible compression.
- **Why**: Storing uncompressed JSON in Redis wastes memory and network bandwidth. A 10KB JSON payload compresses to ~2KB with gzip. For 100K keys: 1GB vs 200MB Redis memory.
- **Use this**: `compressed := gzip(jsonData); rdb.Set(ctx, key, compressed, ttl)`
- **Instead of**: `rdb.Set(ctx, key, jsonData, ttl)` for large values — 5× more Redis memory

### A6. `redis_no_ttl_on_cache_keys`
- [ ] Detect `rdb.Set(ctx, key, val, 0)` or `rdb.Set(ctx, key, val, redis.KeepTTL)` for cache-like keys without TTL.
- **Why**: Cache keys without TTL grow indefinitely until Redis OOM. Even with eviction policies, TTL-less keys are never evicted by `volatile-*` policies.
- **Use this**: `rdb.Set(ctx, key, val, 24*time.Hour)` — explicit TTL
- **Instead of**: `rdb.Set(ctx, key, val, 0)` — never expires, memory leak

---

## Section B — gRPC Worst Practices (5 rules)

### B1. `grpc_dial_per_request`
- [ ] Detect `grpc.Dial(addr, opts...)` or `grpc.NewClient(addr, opts...)` inside handler functions.
- **Why**: gRPC connections are expensive to establish (~10-50ms including TLS handshake). The connection should be shared across requests. Per-request dial creates N connections.
- **Use this**: Process-level `conn, _ := grpc.Dial(addr, opts...)` at startup
- **Instead of**: `grpc.Dial(addr)` per request — ~10-50ms setup per call

### B2. `grpc_large_message_without_streaming`
- [ ] Detect unary RPC handlers returning or receiving messages > 4MB (inferred from large slice/struct serialization patterns).
- **Why**: gRPC default max message size is 4MB. Large unary messages hold memory for the entire payload. Streaming RPCs send data incrementally.
- **Use this**: Server-streaming RPC for large result sets
- **Instead of**: Single unary response with 10K+ items — 4MB+ single allocation

### B3. `grpc_context_not_propagated`
- [ ] Detect gRPC handler methods that create new `context.Background()` instead of using the stream/request context.
- **Why**: gRPC contexts carry deadline, cancellation, and metadata. Dropping the context means the handler doesn't respect client timeouts or cancellation — a resource waste vector.
- **Use this**: Use `ctx` from `UnaryHandler` or `stream.Context()`
- **Instead of**: `context.Background()` in gRPC handler — ignores client deadlines

### B4. `grpc_no_keepalive_config`
- [ ] Detect `grpc.NewServer()` without keepalive server parameters in long-running services.
- **Why**: Without keepalive, idle connections aren't detected as dead. Load balancers and proxies may silently drop connections, causing failed RPCs. Keepalive detects dead connections proactively.
- **Use this**: `grpc.KeepaliveParams(keepalive.ServerParameters{Time: 30*time.Second})`
- **Instead of**: Default no-keepalive — dead connections not detected

### B5. `grpc_unary_interceptor_per_rpc`
- [ ] Detect interceptor/middleware construction inside RPC handler methods instead of at server setup.
- **Why**: Interceptors should be registered once at `grpc.NewServer(grpc.UnaryInterceptor(...))`. Creating them per-RPC wastes allocations and misses the decorator pattern.
- **Use this**: `grpc.NewServer(grpc.ChainUnaryInterceptor(i1, i2))` — registered once
- **Instead of**: Per-RPC interceptor construction — N allocs per call

---

## Section C — Logging Library Worst Practices (5 rules)

### C1. `log_level_check_after_format`
- [ ] Detect `zap.S().Debugf("expensive %v", computeDebug())` or `logrus.Debugf("data: %v", expensiveCall())` where the expensive computation happens regardless of log level.
- **Why**: The arguments are evaluated even if debug logging is disabled. `computeDebug()` runs for nothing. Check the level first or use conditional logging.
- **Use this**: `if logger.Level() <= zap.DebugLevel { logger.Debug("data", zap.Any("d", computeDebug())) }`
- **Instead of**: `logger.Debugf("data: %v", computeDebug())` — argument always evaluated

### C2. `logger_created_per_request`
- [ ] Detect `zap.NewProduction()` or `logrus.New()` inside handler functions.
- **Why**: Logger construction involves allocating encoders, level atomics, and output sinks (~5μs + several allocs). Should be created once at startup with `.With()` for request-scoped fields.
- **Use this**: `logger.With(zap.String("request_id", id))` — reuses base logger
- **Instead of**: `zap.NewProduction()` per request — ~5μs + allocs each time

### C3. `string_format_in_structured_logger`
- [ ] Detect `logger.Info(fmt.Sprintf("user %s logged in", user))` instead of structured fields.
- **Why**: `fmt.Sprintf` allocates a formatted string (~3 allocs, ~200ns). Structured logging with `zap.String("user", user)` avoids the intermediate string and enables machine-parseable logs. ~4× faster.
- **Use this**: `logger.Info("user logged in", zap.String("user", user))` — structured, ~50ns
- **Instead of**: `logger.Info(fmt.Sprintf("user %s logged in", user))` — ~200ns, unstructured

### C4. `log_printf_for_production`
- [ ] Detect `log.Printf` (stdlib) usage in web service handler code.
- **Why**: stdlib `log.Printf` uses a global mutex, has no level control, outputs unstructured text, and contends under concurrency. For 10K log lines/sec: significant mutex contention.
- **Use this**: `slog.Info("message", "key", value)` (Go 1.21+) or `zap`/`zerolog`
- **Instead of**: `log.Printf("msg: %s", val)` — global mutex, no levels, unstructured

### C5. `error_logged_and_returned`
- [ ] Detect `log.Error(err); return err` or `logger.Error("failed", zap.Error(err)); return fmt.Errorf("failed: %w", err)` — logging the error then returning it.
- **Why**: The caller will also likely log the error, causing duplicate log entries. Log at the handling boundary, not at every layer. Double-logging produces noise and wastes I/O.
- **Use this**: Return the error; let the top-level handler log it once
- **Instead of**: `log.Error(err); return err` — every layer logs the same error

---

## Section D — Configuration And CLI Library Worst Practices (5 rules)

### D1. `viper_get_in_hot_path`
- [ ] Detect `viper.GetString("key")` or `viper.GetInt("key")` inside handler functions or loops.
- **Why**: Viper's `Get*` uses a global mutex + case-insensitive map lookup (~200ns). For config values accessed per-request, cache them in a struct at startup. For 10K req/sec: 2ms mutex contention.
- **Use this**: Cache config at startup: `cfg := Config{Port: viper.GetInt("port")}; use cfg.Port`
- **Instead of**: `viper.GetString("key")` per request — ~200ns + mutex per access

### D2. `os_getenv_in_hot_path`
- [ ] Detect `os.Getenv("KEY")` inside handler functions or loops.
- **Why**: `os.Getenv` acquires a lock and scans the environment array (~100ns). For per-request config, cache at startup. The env doesn't change at runtime.
- **Use this**: `var apiKey = os.Getenv("API_KEY")` at package level — read once
- **Instead of**: `os.Getenv("API_KEY")` per request — ~100ns + lock each time

### D3. `config_file_read_per_request`
- [ ] Detect `os.ReadFile("config.json")` or `viper.ReadInConfig()` inside handler or loop code.
- **Why**: Reading and parsing config files per-request involves syscalls (~50μs) and JSON/YAML parsing (~500μs). Config should be read once and reloaded on signal/interval.
- **Use this**: Read config at startup; use `fsnotify` or signal-based reload
- **Instead of**: `os.ReadFile("config.json")` per request — ~550μs I/O + parse

### D4. `cobra_flag_lookup_in_run`
- [ ] Detect `cmd.Flags().GetString("flag")` inside `RunE` when the flag could be bound to a variable with `StringVar`.
- **Why**: `Flags().GetString()` does string-keyed map lookup + type assertion (~100ns). `StringVar` binds directly to a pointer, so reading is a direct memory access (~1ns). ~100× faster for frequently-accessed flags.
- **Use this**: `cmd.Flags().StringVar(&flagVar, "flag", "", "usage")` — direct pointer
- **Instead of**: `cmd.Flags().GetString("flag")` in RunE — map lookup per access

### D5. `env_parsing_repeated_in_init`
- [ ] Detect multiple `os.Getenv` + `strconv.Atoi` / `strconv.ParseBool` chains that could be replaced with a config struct + `envconfig` or `env` library.
- **Why**: Manual env parsing is error-prone (missing defaults, no validation, repeated code). A single `envconfig.Process("", &cfg)` validates everything at once.
- **Use this**: Config struct with `envconfig.Process` — validated, typed, single point
- **Instead of**: 10+ `os.Getenv` + `strconv` chains — repetitive, error-prone

---

## Section E — Prometheus And Metrics Worst Practices (4 rules)

### E1. `prometheus_counter_created_per_request`
- [ ] Detect `prometheus.NewCounter(prometheus.CounterOpts{...})` inside handler functions.
- **Why**: Metrics must be registered once at init time. Creating per request panics on duplicate registration or wastes memory if using `promauto`. ~5μs per construction.
- **Use this**: Package-level `var reqCounter = prometheus.NewCounter(opts)` + `init() { prometheus.MustRegister(reqCounter) }`
- **Instead of**: `prometheus.NewCounter(opts)` per request — panic or leak

### E2. `prometheus_high_cardinality_labels`
- [ ] Detect `counter.WithLabelValues(userId)` or `histogram.WithLabelValues(requestPath)` where label values appear to come from user IDs, request paths, or other high-cardinality sources.
- **Why**: Each unique label combination creates a new time series. 1M users = 1M time series = ~4GB Prometheus memory. This is the #1 cause of Prometheus OOM.
- **Use this**: Bounded labels: `status_code`, `method`, `endpoint_group`
- **Instead of**: `counter.WithLabelValues(userID)` — unbounded cardinality, OOM

### E3. `prometheus_observe_without_timer`
- [ ] Detect manual `time.Since(start).Seconds()` + `histogram.Observe(duration)` patterns when `prometheus.NewTimer` would be safer.
- **Why**: `prometheus.NewTimer(histogram)` + `defer timer.ObserveDuration()` ensures the observation happens even on panic. Manual timing misses panics and early returns.
- **Use this**: `timer := prometheus.NewTimer(histogram); defer timer.ObserveDuration()`
- **Instead of**: Manual `time.Since(start)` + `Observe` — misses panics/early returns

### E4. `prometheus_unregistered_metric`
- [ ] Detect metrics created with `prometheus.NewCounter`/`NewHistogram` that are never registered with `prometheus.MustRegister` or `promauto`.
- **Why**: Unregistered metrics are never scraped. The application silently drops all observations. This is a silent monitoring failure.
- **Use this**: `promauto.NewCounter(opts)` — auto-registers
- **Instead of**: `prometheus.NewCounter(opts)` without `MustRegister` — silently unused

---

## Section F — AWS SDK And Cloud Client Worst Practices (5 rules)

### F1. `aws_session_per_request`
- [ ] Detect `session.NewSession()` or `config.LoadDefaultConfig(ctx)` inside handler functions.
- **Why**: AWS session setup involves reading config files, resolving credentials, and HTTP client setup (~5-50ms). Should be done once at startup.
- **Use this**: Process-level session created at startup
- **Instead of**: `session.NewSession()` per request — ~5-50ms setup each time

### F2. `s3_getobject_without_range`
- [ ] Detect `s3.GetObject` downloading full objects when only partial data is needed (inferred from subsequent `io.LimitReader` or partial reads).
- **Why**: Downloading a 100MB S3 object to read the first 1KB wastes ~99.999% bandwidth. Range requests download only what's needed.
- **Use this**: `s3.GetObject(&s3.GetObjectInput{Range: aws.String("bytes=0-1023")})`
- **Instead of**: Full `GetObject` + `io.LimitReader` — downloads entire object

### F3. `aws_credential_hardcoded`
- [ ] Detect `credentials.NewStaticCredentials("AKID...", "secret...", "")` with literal access keys.
- **Risk**: Hardcoded AWS credentials in source code. If the repo is public or binary is decompiled, the credentials are exposed. AWS key scanning bots exploit leaked keys within minutes.
- **Use this**: `credentials.NewEnvCredentials()` or IAM roles
- **Instead of**: `credentials.NewStaticCredentials("AKID...", "secret", "")` — keys in source

### F4. `s3_listobjects_without_pagination`
- [ ] Detect `s3.ListObjectsV2` without `MaxKeys` or pagination in handler code.
- **Why**: `ListObjectsV2` returns up to 1000 objects by default, which is fine. But without pagination logic, only the first page is processed. For buckets with millions of objects, this silently drops results.
- **Use this**: `s3.ListObjectsV2Pages` or manual pagination loop
- **Instead of**: Single `ListObjectsV2` call — misses objects beyond first page

### F5. `dynamodb_scan_in_handler`
- [ ] Detect `dynamodb.Scan` in request handler functions.
- **Why**: DynamoDB Scan reads every item in the table — it's O(n) and costs read capacity units for the entire table. For a 1M-row table: ~125K RCU consumed per scan. Use Query with partition key.
- **Use this**: `dynamodb.Query` with partition key filter — reads only matching items
- **Instead of**: `dynamodb.Scan` — reads entire table, O(n) cost

---

## Shared Implementation Checklist

- [ ] Implement each rule family as a function in `src/heuristics/go/advanceplan4/` extending the existing pattern.
- [ ] Use `import_aliases_for()` to resolve library aliases (`go-redis/redis`, `google.golang.org/grpc`, `go.uber.org/zap`, `github.com/spf13/viper`, etc.).
- [ ] Default to `Warning` for performance rules; `Error` for security rules that represent direct vulnerabilities.
- [ ] Skip test files and generated files.
- [ ] Add one positive and one clean fixture per section before enabling.
- [ ] Validate against at least one real-world Go application using each library family.

## Acceptance Criteria

- [ ] Every shipped rule explains the specific cost or risk with approximate numbers.
- [ ] Clean fixtures for correct library usage patterns stay quiet.
- [ ] Rules correctly resolve popular library import paths via alias resolution.
- [ ] No rule fires on test code or mocks.
