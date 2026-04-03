# Plan 2 — Security Worst Practices And Vulnerability Patterns (Go)

Date: 2026-04-03

## Status

- [x] Implemented on 2026-04-03.
- [x] All 52 plan2 security rules are now shipped in `src/heuristics/go/advanceplan4/security.rs`.
- [x] Grouped positive and clean fixture coverage ships in `tests/fixtures/go/advanceplan4_security_{positive,clean}.txt`.
- [x] Integration verification ships in `tests/integration_scan/go_advanceplan4.rs`.
- [x] The detailed rule bullets below remain as the drafting inventory; the shipped status above is the source of truth.

## Already Covered And Excluded From This Plan

- [x] `weak_crypto` (md5, sha1, des, rc4) — security.rs
- [x] `hardcoded_secret` — security.rs
- [x] `sql_string_concat` — security.rs
- [x] `http_client_without_timeout` — advanceplan2/plan2
- [x] `http_server_without_timeouts` — advanceplan2/plan2
- [x] `http_response_body_not_closed` — advanceplan2/plan2

## Objective

Build a comprehensive security worst-practice detection pack (52 rules) that targets common vulnerability patterns in Go applications. Focus on patterns that are statically detectable, frequently appear in generated or hastily-written code, and have well-documented security consequences.

## Phase Completion

- [x] Section A shipped all 10 crypto/secret rules.
- [x] Section B shipped all 10 injection/input-validation rules.
- [x] Section C shipped all 8 auth/session rules.
- [x] Section D shipped all 7 concurrency-security rules.
- [x] Section E shipped all 8 network/TLS rules.
- [x] Section F shipped all 9 data-exposure/logging rules.

---

## Section A — Cryptographic Misuse And Secrets (10 rules)

### A1. `insecure_random_for_security`
- [ ] Detect `math/rand` usage (any of `rand.Int`, `rand.Intn`, `rand.Read`, `rand.New`) in functions whose names suggest security use (token generation, key generation, password, nonce, salt, session).
- **Risk**: `math/rand` is deterministic and seedable. An attacker who knows the seed can predict all outputs. CVSS: High.
- **Use this**: `crypto/rand.Read()` — cryptographically secure
- **Instead of**: `math/rand.Intn()` — deterministic, predictable

### A2. `hardcoded_tls_skip_verify`
- [ ] Detect `tls.Config{InsecureSkipVerify: true}` in non-test code.
- **Risk**: Disables TLS certificate validation, enabling MITM attacks. Any network hop between client and server can intercept/modify traffic.
- **Use this**: Proper CA certificate configuration or system cert pool
- **Instead of**: `InsecureSkipVerify: true` — disables all TLS security

### A3. `hardcoded_tls_min_version_too_low`
- [ ] Detect `tls.Config{MinVersion: tls.VersionTLS10}` or `tls.VersionTLS11` or `tls.VersionSSL30`.
- **Risk**: TLS 1.0 and 1.1 have known vulnerabilities (BEAST, POODLE). PCI DSS requires TLS 1.2 minimum.
- **Use this**: `MinVersion: tls.VersionTLS12` or `tls.VersionTLS13`
- **Instead of**: `tls.VersionTLS10` / `tls.VersionTLS11` — known vulnerable protocols

### A4. `weak_hash_for_integrity`
- [ ] Detect `md5.New()`, `sha1.New()`, `md5.Sum()`, `sha1.Sum()` used for integrity checks, checksums, or MAC operations (not just `weak_crypto` import-level detection).
- **Risk**: MD5 and SHA-1 are collision-broken. An attacker can forge files/messages with the same hash. MD5 collision: ~2^18 operations; SHA-1: ~2^63.
- **Use this**: `sha256.New()` or `sha512.New()` — collision-resistant
- **Instead of**: `md5.Sum()` / `sha1.Sum()` — collision-broken

### A5. `constant_encryption_key`
- [ ] Detect `[]byte("...")` used directly as arguments to `cipher.NewGCM`, `aes.NewCipher`, or similar encryption constructor calls.
- **Risk**: Hardcoded keys in source code are trivially extractable from compiled binaries. Any reader of the binary can decrypt all data.
- **Use this**: Key derivation from environment/vault/KMS at runtime
- **Instead of**: `aes.NewCipher([]byte("mysecretkey12345"))` — key in binary

### A6. `constant_iv_or_nonce`
- [ ] Detect constant or zero-valued byte slices used as IV/nonce arguments to `cipher.NewCBCEncrypter`, `gcm.Seal`, or similar.
- **Risk**: Reusing a nonce with AES-GCM completely breaks authenticity and can leak the key. Using a zero IV with CBC enables known-plaintext attacks.
- **Use this**: `crypto/rand.Read(nonce)` for each encryption operation
- **Instead of**: `nonce := make([]byte, 12)` or `var iv [16]byte{}` — nonce reuse

### A7. `ecb_mode_cipher`
- [ ] Detect direct use of `cipher.Block.Encrypt` / `cipher.Block.Decrypt` without a block mode wrapper (CBC, CTR, GCM).
- **Risk**: Direct block cipher usage is ECB mode, which leaks plaintext patterns. Identical plaintext blocks produce identical ciphertext blocks.
- **Use this**: `cipher.NewGCM(block)` or `cipher.NewCTR(block, iv)`
- **Instead of**: `block.Encrypt(dst, src)` directly — ECB mode, leaks patterns

### A8. `jwt_none_algorithm_risk`
- [ ] Detect JWT verification code that accepts `"none"` or `alg: ""` as valid signing methods, or uses `jwt.Parse` without `WithValidMethods`.
- **Risk**: The "none" algorithm bypass is a classic JWT vulnerability. An attacker can forge tokens by setting `alg: "none"` and removing the signature.
- **Use this**: `jwt.Parse(token, keyFunc, jwt.WithValidMethods([]string{"RS256"}))`
- **Instead of**: `jwt.Parse(token, keyFunc)` without method validation

### A9. `bcrypt_cost_too_low`
- [ ] Detect `bcrypt.GenerateFromPassword(pw, cost)` where `cost` is literally `< 10` or `bcrypt.MinCost`.
- **Risk**: Low bcrypt cost allows faster brute-force attacks. At cost=4: ~0.1ms per hash; at cost=12: ~250ms per hash. The difference is ~2500× more attacker effort.
- **Use this**: `bcrypt.GenerateFromPassword(pw, 12)` — ~250ms per hash
- **Instead of**: `bcrypt.GenerateFromPassword(pw, 4)` — ~0.1ms, trivially brute-forceable

### A10. `rsa_key_size_too_small`
- [ ] Detect `rsa.GenerateKey(rand, bits)` where `bits` is literally `< 2048`.
- **Risk**: RSA-1024 is considered breakable by nation-state actors. NIST recommends 2048-bit minimum, 3072-bit for post-2030 protection.
- **Use this**: `rsa.GenerateKey(rand, 2048)` minimum — NIST recommended
- **Instead of**: `rsa.GenerateKey(rand, 1024)` — breakable, non-compliant

---

## Section B — Injection And Input Validation (10 rules)

### B1. `os_exec_command_with_user_input`
- [ ] Detect `exec.Command(userInput)` or `exec.Command("sh", "-c", variable)` where the command string appears to come from a function parameter or request binding.
- **Risk**: OS command injection allows arbitrary code execution. An attacker can chain commands with `;`, `|`, `&&`, backticks.
- **Use this**: `exec.Command(fixedBinary, userArg)` with no shell interpretation
- **Instead of**: `exec.Command("sh", "-c", userInput)` — full shell injection

### B2. `template_html_unescaped`
- [ ] Detect `template.HTML(userInput)` or `template.JS(userInput)` type conversions on data from request parameters.
- **Risk**: `template.HTML` marks content as safe, bypassing `html/template`'s auto-escaping. This enables stored/reflected XSS attacks.
- **Use this**: Let `html/template` auto-escape: `{{.UserContent}}`
- **Instead of**: `template.HTML(r.FormValue("content"))` — disables XSS protection

### B3. `text_template_for_html`
- [ ] Detect `text/template` used to generate HTML content (check for HTML tags in template literals or `.html` file extensions in `ParseFiles`).
- **Risk**: `text/template` does not auto-escape HTML. Any user data in the template creates XSS vulnerabilities. `html/template` provides context-aware escaping.
- **Use this**: `html/template` for any HTML output — auto-escapes by context
- **Instead of**: `text/template` for HTML — no escaping, XSS vulnerable

### B4. `filepath_join_with_user_path`
- [ ] Detect `filepath.Join(baseDir, userInput)` without subsequent `filepath.Rel` or path-containment validation.
- **Risk**: Path traversal via `../../etc/passwd`. `filepath.Join` resolves `..` segments. Without validation that the result stays within `baseDir`, attackers can read/write arbitrary files.
- **Use this**: `filepath.Rel(baseDir, joined)` and verify no `..` prefix
- **Instead of**: `filepath.Join(base, userInput)` without containment check — path traversal

### B5. `url_redirect_without_validation`
- [ ] Detect `http.Redirect(w, r, r.FormValue("redirect_url"), 302)` or `c.Redirect(302, c.Query("url"))` without URL validation.
- **Risk**: Open redirect allows attackers to redirect users to malicious sites while the URL appears to come from the trusted domain. Phishing vector.
- **Use this**: Validate redirect URL against an allowlist or ensure it's a relative path
- **Instead of**: `http.Redirect(w, r, userProvidedURL, 302)` — open redirect

### B6. `ssrf_via_user_controlled_url`
- [ ] Detect `http.Get(userInput)` or `http.NewRequest("GET", userInput, nil)` where the URL comes from request parameters.
- **Risk**: Server-Side Request Forgery (SSRF) allows attackers to make the server access internal resources, cloud metadata endpoints (169.254.169.254), or internal services.
- **Use this**: Validate URL against an allowlist; block private/loopback IPs
- **Instead of**: `http.Get(r.FormValue("url"))` — unlimited internal access

### B7. `ldap_injection_via_string_concat`
- [ ] Detect string concatenation or `fmt.Sprintf` building LDAP filter strings with user input.
- **Risk**: LDAP injection can bypass authentication or extract directory data. Special characters like `*`, `(`, `)`, `\`, NUL can modify query semantics.
- **Use this**: LDAP filter escaping via `ldap.EscapeFilter(userInput)`
- **Instead of**: `fmt.Sprintf("(uid=%s)", userInput)` — LDAP injection

### B8. `header_injection_via_user_input`
- [ ] Detect `w.Header().Set(name, userInput)` or `w.Header().Add(name, userInput)` where the value contains unvalidated user input that could contain `\r\n`.
- **Risk**: HTTP header injection / response splitting. Injecting `\r\n` allows attackers to forge headers, set cookies, or inject response bodies.
- **Use this**: Validate/sanitize header values; reject `\r` and `\n` characters
- **Instead of**: `w.Header().Set("X-Custom", r.FormValue("val"))` — header injection

### B9. `xml_decoder_without_entity_limit`
- [ ] Detect `xml.NewDecoder(r)` processing untrusted XML without setting `d.Entity = nil` and without input size limits.
- **Risk**: XML External Entity (XXE) attack and Billion Laughs DoS. Malicious XML can reference external DTDs or expand entities exponentially.
- **Use this**: `d.Entity = map[string]string{}` and wrap with `io.LimitReader`
- **Instead of**: Default `xml.NewDecoder` on untrusted input — XXE/DoS vulnerable

### B10. `yaml_unmarshal_untrusted_input`
- [ ] Detect `yaml.Unmarshal(untrustedInput, &target)` using `gopkg.in/yaml.v2` without size limits.
- **Risk**: YAML v2 can create arbitrary Go objects via `!!go/object` tags. Malicious YAML can execute code. `yaml.v3` with `KnownFields(true)` is safer.
- **Use this**: `yaml.v3` with strict mode, or validate/limit input before unmarshal
- **Instead of**: `yaml.v2` unmarshal on untrusted input — potential code execution

---

## Section C — Authentication, Session, And Access Control (8 rules)

### C1. `cookie_without_secure_flag`
- [ ] Detect `http.Cookie{...}` literals without `Secure: true` for session or authentication cookies.
- **Risk**: Without `Secure`, cookies are sent over plain HTTP. Any network observer can steal session cookies. OWASP Top 10 Sensitive Data Exposure.
- **Use this**: `http.Cookie{Name: "session", Secure: true, HttpOnly: true, SameSite: http.SameSiteStrictMode}`
- **Instead of**: `http.Cookie{Name: "session"}` — transmitted over HTTP, stealable

### C2. `cookie_without_httponly`
- [ ] Detect `http.Cookie{...}` for session/auth cookies without `HttpOnly: true`.
- **Risk**: Without `HttpOnly`, JavaScript can access the cookie via `document.cookie`. Any XSS vulnerability can steal session cookies.
- **Use this**: `HttpOnly: true` for all session cookies
- **Instead of**: Omitting HttpOnly — allows JavaScript cookie theft via XSS

### C3. `cookie_without_samesite`
- [ ] Detect `http.Cookie{...}` without `SameSite` set, particularly for auth/session cookies.
- **Risk**: Without SameSite, cookies are sent on cross-site requests, enabling CSRF attacks. `SameSite=Lax` prevents most CSRF vectors.
- **Use this**: `SameSite: http.SameSiteLaxMode` minimum
- **Instead of**: Default SameSite (browser-dependent) — CSRF vulnerable

### C4. `cors_allow_all_origins`
- [ ] Detect `Access-Control-Allow-Origin: *` combined with `Access-Control-Allow-Credentials: true`, or CORS middleware configured with `AllowAllOrigins: true` in Gin/Echo/Chi.
- **Risk**: Wildcard CORS with credentials allows any website to make authenticated requests to the API. Enables data theft from authenticated users.
- **Use this**: Explicit origin allowlist: `AllowOrigins: []string{"https://app.example.com"}`
- **Instead of**: `AllowAllOrigins: true` with credentials — any site can read responses

### C5. `jwt_secret_in_source`
- [ ] Detect `jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString([]byte("hardcoded"))` where the signing key is a string literal.
- **Risk**: JWT signing secret in source code allows anyone with binary/source access to forge valid JWTs and impersonate any user.
- **Use this**: Load signing key from environment variable, vault, or KMS
- **Instead of**: `[]byte("my-secret-key")` inline — forging tokens trivial

### C6. `timing_attack_on_token_comparison`
- [ ] Detect `token == expectedToken` or `bytes.Equal(token, expected)` for comparing authentication tokens, API keys, or HMAC values.
- **Risk**: Standard string/byte comparison short-circuits on the first different byte, leaking timing information. An attacker can brute-force one byte at a time. HMAC comparison should take constant time.
- **Use this**: `hmac.Equal(a, b)` or `subtle.ConstantTimeCompare(a, b)` — constant-time
- **Instead of**: `token == expected` — timing side-channel, byte-by-byte brute-force

### C7. `missing_rate_limiting_on_auth_endpoint`
- [ ] Detect login/authentication handler functions (name contains `Login`, `Authenticate`, `SignIn`) that don't reference rate limiting, throttling, or brute-force protection mechanisms.
- **Risk**: Without rate limiting, attackers can brute-force credentials at full speed. A login endpoint processing 1000 req/sec allows ~86M attempts/day.
- **Use this**: Rate limiter middleware (e.g., `golang.org/x/time/rate`, `tollbooth`)
- **Instead of**: Unprotected auth endpoints — unlimited brute-force attempts

### C8. `password_stored_as_plaintext`
- [ ] Detect struct fields named `Password`, `Passwd`, or `Pwd` stored as `string` in database model structs without evidence of hashing.
- **Risk**: Plaintext password storage is the #1 worst practice. Any database leak exposes all user credentials directly.
- **Use this**: `bcrypt.GenerateFromPassword(password, cost)` before storage
- **Instead of**: `user.Password = plaintext` — database leak = full credential exposure

---

## Section D — Concurrency Security And Race Conditions (7 rules)

### D1. `race_on_shared_map`
- [ ] Detect map reads/writes from multiple goroutines without mutex or `sync.Map` protection (detect goroutine launches + shared map access patterns).
- **Risk**: Concurrent map access causes runtime panics in Go. `fatal error: concurrent map writes` crashes the process. In security-critical contexts, this is a DoS vector.
- **Use this**: `sync.Map` or `sync.RWMutex`-protected map access
- **Instead of**: Unprotected `map[K]V` accessed from goroutines — runtime panic

### D2. `toctou_file_check_then_open`
- [ ] Detect `os.Stat(path)` or file existence check followed by `os.Open(path)` or `os.Create(path)` without atomic operations.
- **Risk**: Time-of-check-to-time-of-use (TOCTOU) race. Between the stat and open, an attacker can swap the file (symlink race). Particularly dangerous in `setuid` programs or containerized environments.
- **Use this**: `os.OpenFile` with appropriate flags; handle errors directly
- **Instead of**: `if _, err := os.Stat(p); err == nil { os.Open(p) }` — TOCTOU race

### D3. `shared_slice_append_race`
- [ ] Detect goroutines appending to a shared slice without synchronization.
- **Risk**: Concurrent `append` to the same slice causes data races. Unlike maps, Go doesn't crash on concurrent slice access — it silently corrupts data, which is worse for security (corrupted auth tokens, permissions, etc.).
- **Use this**: Channel-based collection or mutex-protected append
- **Instead of**: `go func() { results = append(results, v) }()` — silent data corruption

### D4. `goroutine_captures_loop_variable`
- [ ] Detect `for _, v := range items { go func() { use(v) }() }` without rebinding `v` inside the loop body (pre-Go 1.22).
- **Risk**: All goroutines capture the same variable and see its final value. This can cause authentication bypass if goroutines process the wrong user's data.
- **Use this**: `go func(v T) { use(v) }(v)` or use Go 1.22+ loop variable semantics
- **Instead of**: `go func() { use(v) }()` — all goroutines see the last value

### D5. `unsafe_pointer_cast`
- [ ] Detect `unsafe.Pointer` casts between incompatible types, particularly `uintptr` arithmetic followed by cast back to `unsafe.Pointer`.
- **Risk**: `unsafe.Pointer` bypasses Go's type safety. Invalid casts can read/write arbitrary memory (buffer overflow, info leak). `uintptr` values are not tracked by GC and can become dangling pointers.
- **Use this**: Type-safe alternatives; if `unsafe` is needed, follow the 6 valid patterns from `unsafe.Pointer` docs
- **Instead of**: Ad-hoc `unsafe.Pointer(uintptr(...) + offset)` — memory corruption

### D6. `cgo_string_lifetime`
- [ ] Detect `C.CString(goString)` without a corresponding `C.free` in the same function, or deferred `C.free`.
- **Risk**: `C.CString` allocates C memory that the Go GC doesn't track. Leaking this memory is a classic DoS vector. In long-running servers, leaked CStrings can exhaust memory.
- **Use this**: `cs := C.CString(s); defer C.free(unsafe.Pointer(cs))`
- **Instead of**: `C.CString(s)` without `C.free` — memory leak

### D7. `global_rand_source_contention`
- [ ] Detect `math/rand.Intn()`, `rand.Float64()`, etc. (global source) in hot handler or goroutine paths.
- **Risk**: The global `math/rand` source has a mutex. Under high concurrency, this becomes a contention bottleneck. In Go 1.20+, `rand.New(rand.NewSource(...))` per goroutine avoids the lock. Not a security issue per se, but causes DoS under load.
- **Use this**: `rand.New(rand.NewSource(seed))` per goroutine (Go < 1.22) or `math/rand/v2` (Go 1.22+)
- **Instead of**: Global `rand.Intn()` — mutex contention under load

---

## Section E — Network And TLS Security (8 rules)

### E1. `http_handler_without_csrf_protection`
- [ ] Detect POST/PUT/DELETE handler registration without evidence of CSRF token middleware.
- **Risk**: Without CSRF protection, malicious sites can submit forms to your application on behalf of authenticated users. OWASP Top 10: Broken Access Control.
- **Use this**: CSRF middleware (e.g., `gorilla/csrf`, `nosurf`, Gin CSRF middleware)
- **Instead of**: Unprotected state-changing endpoints — vulnerable to cross-site form submission

### E2. `http_handler_missing_security_headers`
- [ ] Detect HTTP handler functions that write responses without setting `X-Content-Type-Options`, `X-Frame-Options`, or `Content-Security-Policy` headers (or without security header middleware).
- **Risk**: Missing security headers enable MIME sniffing, clickjacking, and XSS attacks. Modern browsers rely on these headers for protection.
- **Use this**: Security header middleware setting `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, CSP
- **Instead of**: No security headers — clickjacking, MIME sniffing vulnerable

### E3. `http_listen_non_tls`
- [ ] Detect `http.ListenAndServe` (non-TLS) usage in production-like code (not test files, not localhost bindings).
- **Risk**: All traffic is unencrypted. Credentials, session tokens, and sensitive data are visible to any network observer.
- **Use this**: `http.ListenAndServeTLS` or reverse proxy with TLS termination
- **Instead of**: `http.ListenAndServe(":8080", nil)` — all traffic plaintext

### E4. `dns_lookup_for_access_control`
- [ ] Detect `net.LookupHost` or `net.LookupAddr` results used in access control decisions.
- **Risk**: DNS responses can be spoofed (DNS rebinding). Basing access control on hostname resolution allows attackers to bypass IP-based restrictions.
- **Use this**: IP-based access control; validate IPs directly
- **Instead of**: `net.LookupHost(hostname)` for auth decisions — DNS spoofable

### E5. `websocket_without_origin_check`
- [ ] Detect `websocket.Upgrader{CheckOrigin: func(r *http.Request) bool { return true }}` or missing `CheckOrigin`.
- **Risk**: Without origin validation, any website can establish a WebSocket connection to your server, enabling cross-site WebSocket hijacking.
- **Use this**: `CheckOrigin` validating against allowed origins
- **Instead of**: `CheckOrigin: func(r *http.Request) bool { return true }` — open to all origins

### E6. `grpc_without_tls_credentials`
- [ ] Detect `grpc.Dial(addr, grpc.WithInsecure())` or `grpc.WithTransportCredentials(insecure.NewCredentials())` in non-test code.
- **Risk**: gRPC without TLS sends all data (including auth tokens) in plaintext. Any network observer can intercept RPCs.
- **Use this**: `grpc.WithTransportCredentials(credentials.NewTLS(tlsConfig))`
- **Instead of**: `grpc.WithInsecure()` — all RPC traffic plaintext

### E7. `ssh_host_key_callback_insecure`
- [ ] Detect `ssh.ClientConfig{HostKeyCallback: ssh.InsecureIgnoreHostKey()}` in non-test code.
- **Risk**: Disables SSH host key verification, enabling MITM attacks on SSH connections. Any network position can impersonate the server.
- **Use this**: `ssh.FixedHostKey(expectedKey)` or custom `HostKeyCallback`
- **Instead of**: `ssh.InsecureIgnoreHostKey()` — MITM vulnerable

### E8. `smtp_plaintext_auth`
- [ ] Detect `smtp.PlainAuth` used without TLS (`smtp.SendMail` to non-TLS endpoints).
- **Risk**: SMTP PLAIN auth sends credentials in base64 (trivially decoded) over plaintext. Any network observer captures email credentials.
- **Use this**: `smtp.PlainAuth` only over TLS connections; use STARTTLS
- **Instead of**: `smtp.PlainAuth(...)` over plaintext — credentials exposed

---

## Section F — Data Exposure And Logging Security (9 rules)

### F1. `sensitive_data_in_log`
- [ ] Detect `log.Printf`, `slog.Info`, `zap.String`, `logrus.WithField` calls that include variables named `password`, `secret`, `token`, `apiKey`, `creditCard`, `ssn`, or similar.
- **Risk**: Sensitive data in logs is accessible to anyone with log access, often persisted indefinitely, and frequently sent to third-party log aggregation services.
- **Use this**: Redact sensitive fields: `log.Printf("user=%s login attempt", user)` — no password
- **Instead of**: `log.Printf("user=%s password=%s", user, password)` — password in logs

### F2. `error_detail_leaked_to_client`
- [ ] Detect `c.JSON(500, gin.H{"error": err.Error()})` or `http.Error(w, err.Error(), 500)` returning internal error details to the client.
- **Risk**: Internal error messages can leak stack traces, database schemas, file paths, and internal service names, aiding reconnaissance.
- **Use this**: Generic error response + internal logging: `c.JSON(500, gin.H{"error": "internal server error"})`
- **Instead of**: `c.JSON(500, gin.H{"error": err.Error()})` — leaks internals

### F3. `debug_endpoint_in_production`
- [ ] Detect `net/http/pprof` import or `http.Handle("/debug/pprof/", ...)` registration without access control.
- **Risk**: pprof endpoints expose heap dumps, goroutine stacks, CPU profiles, and can reveal secrets in memory. Publicly accessible pprof is a critical information disclosure vulnerability.
- **Use this**: Separate debug listener on internal-only port with auth
- **Instead of**: `import _ "net/http/pprof"` on the public server — heap dump access

### F4. `struct_field_exposed_in_json`
- [ ] Detect exported struct fields containing sensitive data (Password, Secret, Token, APIKey, PrivateKey) without `json:"-"` tags in API response structs.
- **Risk**: These fields are automatically included in JSON serialization. A single `json.Marshal` call exposes them in API responses.
- **Use this**: `Password string \`json:"-"\`` — excluded from serialization
- **Instead of**: `Password string \`json:"password"\`` — included in every response

### F5. `temp_file_predictable_name`
- [ ] Detect `os.Create("/tmp/myapp-data.txt")` or `os.OpenFile("/tmp/" + fixedName, ...)` with predictable filenames.
- **Risk**: Predictable temp file names enable symlink attacks. An attacker creates a symlink at the expected path pointing to a sensitive file, and the application writes to/reads from the wrong file.
- **Use this**: `os.CreateTemp("", "myapp-*")` — random suffix, secure
- **Instead of**: `os.Create("/tmp/myapp-data.txt")` — predictable, symlink attackable

### F6. `world_readable_file_permissions`
- [ ] Detect `os.OpenFile(path, flag, 0666)` or `os.WriteFile(path, data, 0777)` with world-readable/writable permissions.
- **Risk**: World-readable files can be read by any user on the system. For credentials, keys, or sensitive config: any local user can steal them.
- **Use this**: `os.OpenFile(path, flag, 0600)` — owner-only access
- **Instead of**: `os.OpenFile(path, flag, 0666)` — world-readable

### F7. `fmt_print_of_sensitive_struct`
- [ ] Detect `fmt.Sprintf("%+v", user)` or `fmt.Printf("%v", config)` on structs that contain password/secret/token fields.
- **Risk**: `%+v` prints all field names and values, including passwords, tokens, and secrets. Often used in error messages or logs.
- **Use this**: Custom `String()` method that redacts sensitive fields
- **Instead of**: `fmt.Sprintf("%+v", user)` — prints all fields including secrets

### F8. `panic_stack_trace_to_client`
- [ ] Detect `recover()` in HTTP middleware that sends the panic message/stack to the response writer.
- **Risk**: Stack traces contain file paths, function names, argument values, and goroutine states — all useful for attacker reconnaissance.
- **Use this**: Log the panic internally; return generic 500 to the client
- **Instead of**: `w.Write([]byte(fmt.Sprintf("%v", r)))` — stack trace to client

### F9. `env_var_in_error_message`
- [ ] Detect `fmt.Errorf("... %s", os.Getenv("SECRET_KEY"))` or similar patterns that embed environment variable values in errors.
- **Risk**: Environment variables often contain secrets (API keys, DB passwords). Embedding them in error messages that may be logged or returned to clients exposes them.
- **Use this**: Log the env var name, not its value: `fmt.Errorf("SECRET_KEY is invalid")`
- **Instead of**: `fmt.Errorf("key %s is invalid", os.Getenv("SECRET_KEY"))` — secret in error

---

## Shared Implementation Checklist

- [x] Implement each rule family as a function in `src/heuristics/go/advanceplan4/` using the existing pattern.
- [x] Use `import_aliases_for()` to resolve package aliases for `crypto/*`, `net/http`, `os`, `database/sql`, `encoding/xml`, etc.
- [x] Use `body_lines()` with pattern matching for composite literal and function argument analysis.
- [x] Default to `Warning` severity for security rules and `Error` for the direct-exploit families.
- [x] Suppress findings in test code via `is_test_file` / `is_test_function` gating in the rule-pack entrypoint.
- [x] Add one positive and one clean fixture for every rule section before enabling.
- [x] Validate the shipped rule pack with `cargo test go_advanceplan4 -- --nocapture`.

## Acceptance Criteria

- [x] Every shipped rule explains the specific attack vector or vulnerability class.
- [x] Clean fixtures demonstrating secure code patterns stay quiet.
- [x] Rules remain function-local and parser-driven without requiring type checking.
- [x] Security severity is proportional to exploitability and impact.
- [x] Rules reference relevant standards heuristically in the explanatory text where applicable.
