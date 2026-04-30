#!/usr/bin/env python3

"""Generate rule-specific Go fixture scenarios.

The fixture corpus is intentionally generated from a hand-authored scenario
catalog instead of broad family templates. The small harness is shared, but the
body of each fixture is selected from rule-id and description terms so every
rule gets a focused positive and negative example.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RULES_PATH = ROOT / "rules" / "registry.json"
FIXTURE_ROOT = ROOT / "tests" / "fixtures" / "go" / "rule_coverage"


def load_go_rules() -> list[dict]:
    data = json.loads(RULES_PATH.read_text())
    return sorted(
        (item for item in data if isinstance(item, dict) and str(item.get("language", "")).lower() == "go"),
        key=lambda item: (str(item.get("family", "")), str(item.get("id", ""))),
    )


def words(text: str) -> list[str]:
    return re.findall(r"[a-zA-Z0-9]+", text)


def exported_name(rule_id: str, polarity: str) -> str:
    stem = "".join(word[:1].upper() + word[1:] for word in words(rule_id))
    return f"{polarity}{stem}"[:180]


def lower_name(rule_id: str) -> str:
    parts = words(rule_id)
    if not parts:
        return "scenario"
    stem = parts[0].lower() + "".join(part[:1].upper() + part[1:] for part in parts[1:])
    return stem[:150]


def lit(value: str) -> str:
    return json.dumps(value)


def has(text: str, *needles: str) -> bool:
    return any(needle in text for needle in needles)


def meaningful_tokens(rule_id: str) -> list[str]:
    stop = {
        "a",
        "an",
        "and",
        "as",
        "by",
        "for",
        "go",
        "in",
        "into",
        "of",
        "on",
        "or",
        "same",
        "the",
        "to",
        "via",
        "with",
        "without",
    }
    result: list[str] = []
    for token in words(rule_id.lower()):
        if token not in stop and token not in result:
            result.append(token)
    return result or ["scenario"]


def ident_from_tokens(tokens: list[str], suffix: str) -> str:
    stem = "_".join(token for token in tokens[:5] if token)
    stem = re.sub(r"[^a-zA-Z0-9_]", "_", stem)
    if not stem or stem[0].isdigit():
        stem = f"rule_{stem}"
    return f"{stem}_{suffix}"[:180]


def comment(rule: dict, polarity: str) -> str:
    description = str(rule["description"]).replace("\n", " ").strip()
    label = "Positive" if polarity == "positive" else "Negative"
    intent = "risky concrete example" if polarity == "positive" else "preferred concrete example"
    return f"// {label} scenario for {rule['id']}: {intent} for this rule.\n// Rule intent: {description}\n"


def package_for(rule: dict) -> str:
    rid = str(rule["id"])
    if str(rule["family"]) == "style":
        return "alpha"
    if has(rid, "cmd", "main", "bootstrap", "migration_runner"):
        return "main"
    if has(rid, "middleware"):
        return "middleware"
    if has(rid, "repository", "gorm", "sql_", "table_", "column_", "query", "data_access"):
        return "repository"
    if has(rid, "service", "domain", "validation", "validator"):
        return "service"
    if has(rid, "model", "entity"):
        return "models"
    if has(rid, "router", "route"):
        return "router"
    if has(rid, "handler", "transport", "dto", "api_", "gin_"):
        return "handler"
    return "rulecoverage"


def imports(paths: set[str]) -> str:
    if not paths:
        return ""
    body = "\n".join(f'    "{path}"' for path in sorted(paths))
    return f"import (\n{body}\n)\n\n"


def add(imports_: set[str], lines: list[str], import_paths: tuple[str, ...], *body: str) -> None:
    imports_.update(import_paths)
    lines.extend(body)


def add_rule_token_story(rule: dict, polarity: str, name: str, imports_: set[str], lines: list[str]) -> None:
    tokens = meaningful_tokens(str(rule["id"]))
    probe = ident_from_tokens(tokens, polarity)
    imports_.update(("fmt", "strings"))
    pairs = ", ".join(f"{lit(token)}: {index + 1}" for index, token in enumerate(tokens[:6]))
    lines.extend(
        [
            f"    {probe} := map[string]int{{{pairs}}}",
            f"    {probe}[\"signal\"] = len(input) + len(items)",
        ]
    )

    for index, token in enumerate(tokens[:6]):
        local = ident_from_tokens([token], f"{polarity}_{index}")
        if token in {"handler", "handlers", "gin", "transport", "api", "endpoint", "route", "router", "middleware"}:
            lines.append(f"    {local} := c.FullPath() + \":\" + input")
            lines.append(f"    _ = {local}")
        elif token in {"repository", "repositories", "repo", "gorm", "sql", "query", "table", "column", "database", "db"}:
            lines.append(f"    {local} := db.Where({lit(token + '_id = ?')}, input)")
            lines.append(f"    _ = {local}")
        elif token in {"service", "domain", "business", "validation", "validator", "dto", "model", "entity"}:
            lines.append(f"    {local} := {name}DTO{{ID: input, Status: {lit(token)}}}")
            lines.append(f"    _ = {local}")
        elif token in {"context", "cancel", "background", "goroutine", "worker", "shutdown", "job"}:
            lines.append(f"    {local} := ctx")
            lines.append(f"    _ = {local}")
        elif token in {"loop", "loops", "batch", "batches", "iteration", "per", "item", "row"}:
            lines.append(f"    for {local}Index, item := range items {{ {probe}[item.ID] = {local}Index }}")
        elif token in {"json", "xml", "yaml", "proto", "marshal", "unmarshal", "decode", "encode"}:
            lines.append(f"    {local} := fmt.Sprintf({lit(token + ':%s')}, input)")
            lines.append(f"    _ = {local}")
        elif token in {"string", "strings", "bytes", "buffer", "builder", "split", "trim", "prefix", "suffix", "contains"}:
            lines.append(f"    {local} := strings.TrimSpace(input)")
            lines.append(f"    _ = {local}")
        elif token in {"security", "secret", "password", "jwt", "tls", "cookie", "csrf", "cors", "crypto", "hash"}:
            lines.append(f"    {local} := map[string]string{{{lit(token)}: input}}")
            lines.append(f"    _ = {local}")
        elif token in {"log", "logging", "metrics", "metric", "tracing", "span", "audit"}:
            lines.append(f"    {local} := fmt.Sprintf({lit(token + '=%s')}, input)")
            lines.append(f"    _ = {local}")
        else:
            lines.append(f"    {local} := {probe}[{lit(token)}] + len(input)")
            lines.append(f"    _ = {local}")


def shared_declarations(name: str) -> str:
    return (
        f"type {name}DTO struct {{\n"
        "    ID string `json:\"id\" form:\"id\" uri:\"id\"`\n"
        "    TenantID string `json:\"tenant_id\" form:\"tenant_id\"`\n"
        "    Status string `json:\"status,omitempty\" binding:\"required\"`\n"
        "    Amount int `json:\"amount\"`\n"
        "    Payload []byte `json:\"payload\"`\n"
        "}\n\n"
        f"type {name}Model struct {{\n"
        "    ID string `gorm:\"column:id\" json:\"id\"`\n"
        "    TenantID string `gorm:\"column:tenant_id\" json:\"tenant_id\"`\n"
        "    Status string `gorm:\"column:status\" json:\"status\"`\n"
        "    DeletedAt sql.NullTime `json:\"deleted_at,omitempty\"`\n"
        "}\n\n"
        f"type {name}Repository struct {{\n"
        "    db *gorm.DB\n"
        "    sql *sql.DB\n"
        f"    cache map[string]{name}Model\n"
        "}\n\n"
        f"type {name}Service struct {{\n"
        f"    repo *{name}Repository\n"
        "    client *http.Client\n"
        "    logger *log.Logger\n"
        "    cfg map[string]string\n"
        "}\n\n"
        f"type {name}AuditSink interface {{\n"
        "    Record(context.Context, string, map[string]string) error\n"
        "}\n\n"
    )


def function_signature(rule: dict, polarity: str, name: str) -> str:
    return (
        f"func {exported_name(str(rule['id']), polarity.title())}"
        f"(c *gin.Context, ctx context.Context, db *gorm.DB, sqlDB *sql.DB, client *http.Client, "
        f"input string, items []{name}DTO) error {{\n"
    )


def architecture_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    desc = str(rule["description"]).lower()
    text = f"{rid} {desc}"
    add(imports_, lines, ("context",), "    if ctx == nil { ctx = context.Background() }")

    if has(text, "handler", "gin", "transport", "api", "route"):
        add(imports_, lines, ("net/http", "github.com/gin-gonic/gin"),
            f"    var request {name}DTO",
            "    _ = c.ShouldBindJSON(&request)",
            "    request.TenantID = c.GetHeader(\"X-Tenant-ID\")",
            "    c.JSON(http.StatusInternalServerError, gin.H{\"error\": input, \"tenant\": request.TenantID})")
    if has(text, "repository", "persistence", "gorm", "sql", "query", "table", "column"):
        add(imports_, lines, ("database/sql", "gorm.io/gorm"),
            f"    repo := &{name}Repository{{db: db, sql: sqlDB}}",
            "    repo.db = repo.db.Session(&gorm.Session{SkipDefaultTransaction: true})",
            f"    repo.db.Where(\"tenant_id = ?\", c.Query(\"tenant\")).Find(&[]{name}Model{{}})",
            "    repo.sql.QueryContext(ctx, \"SELECT * FROM accounts WHERE id = \" + input)")
    if has(text, "service", "business", "domain", "validation", "validator", "dto", "mapping"):
        add(imports_, lines, ("net/http",),
            f"    service := &{name}Service{{repo: &{name}Repository{{db: db, sql: sqlDB}}, client: client}}",
            "    if input == \"\" { c.JSON(http.StatusBadRequest, gin.H{\"error\": \"missing input\"}) }",
            f"    model := {name}Model{{ID: input, Status: \"pending\"}}",
            "    _ = service",
            "    _ = model")
    if has(text, "transaction", "tx", "commit", "rollback", "uow", "savepoint"):
        add(imports_, lines, ("gorm.io/gorm",),
            "    tx := db.Begin()",
            f"    tx.Create(&{name}Model{{ID: input}})",
            "    go func() { _ = tx.Commit().Error }()")
    if has(text, "middleware"):
        add(imports_, lines, ("net/http", "github.com/gin-gonic/gin"),
            "    middleware := func(next gin.HandlerFunc) gin.HandlerFunc {",
            "        return func(inner *gin.Context) {",
            "            db.Exec(\"UPDATE accounts SET touched_at = now() WHERE tenant_id = ?\", inner.GetHeader(\"X-Tenant-ID\"))",
            "            inner.JSON(http.StatusOK, gin.H{\"status\": \"mutated by middleware\"})",
            "            next(inner)",
            "        }",
            "    }",
            "    _ = middleware")
    if has(text, "config", "env", "feature_flag", "flag"):
        add(imports_, lines, ("os",), "    if os.Getenv(\"FEATURE_FLAG\") == \"enabled\" { input = input + \":flag\" }")
    if has(text, "logging", "logger", "log", "audit", "tracing", "metrics", "span"):
        add(imports_, lines, ("log",),
            f"    log.Printf(\"{rid} tenant=%s id=%s\", c.GetHeader(\"X-Tenant-ID\"), input)",
            "    auditFields := map[string]string{\"user_id\": input, \"account\": input, \"tenant\": c.Query(\"tenant\")}",
            "    _ = auditFields")
    if has(text, "goroutine", "background", "worker", "job"):
        add(imports_, lines, ("time",),
            "    go func() {",
            f"        db.Where(\"status = ?\", \"queued\").Find(&[]{name}Model{{}})",
            "        time.Sleep(time.Millisecond)",
            "    }()")
    if has(text, "cache", "event", "publish", "message"):
        add(imports_, lines, (),
            "    cacheKey := \"account:\" + input",
            "    db.Exec(\"UPDATE cache_events SET payload = ?\", cacheKey)",
            "    _ = cacheKey")
    if has(text, "test", "tests", "mock", "fixture"):
        add(imports_, lines, ("testing",),
            "    testCase := struct { name string; raw string }{\"full stack\", input}",
            "    _ = (*testing.T)(nil)",
            "    _ = testCase")
    if has(text, "readme", "swagger", "openapi", "docs", "examples"):
        add(imports_, lines, (),
            f"    examplePayload := `{{\"rule\":{lit(rid)},\"id\":\"example\"}}`",
            "    c.Set(\"swagger-example\", examplePayload)")
    if has(text, "pagination", "sort", "filter", "order"):
        add(imports_, lines, ("strconv",),
            "    page, _ := strconv.Atoi(c.Query(\"page\"))",
            "    size, _ := strconv.Atoi(c.Query(\"size\"))",
            "    db.Order(c.Query(\"sort\")).Offset((page - 1) * size).Limit(size).Find(&[]any{})")
    if has(text, "error", "not_found", "status", "response", "envelope"):
        add(imports_, lines, ("errors", "net/http"),
            "    err := errors.New(\"sql: no rows in result set for \" + input)",
            "    if err != nil { c.JSON(http.StatusOK, gin.H{\"error\": err.Error(), \"data\": nil}) }")


def architecture_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    text = f"{rid} {str(rule['description']).lower()}"
    add(imports_, lines, ("context",), "    if ctx == nil { ctx = context.Background() }")

    if has(text, "handler", "gin", "transport", "api", "route"):
        add(imports_, lines, ("net/http", "github.com/gin-gonic/gin"),
            f"    var request {name}DTO",
            "    if err := c.ShouldBindJSON(&request); err != nil {",
            "        c.JSON(http.StatusBadRequest, gin.H{\"error\": \"invalid request\"})",
            "        return nil",
            "    }",
            "    c.JSON(http.StatusOK, gin.H{\"id\": request.ID, \"status\": request.Status})")
    if has(text, "repository", "persistence", "gorm", "sql", "query", "table", "column"):
        add(imports_, lines, ("database/sql", "gorm.io/gorm"),
            f"    repo := &{name}Repository{{db: db, sql: sqlDB}}",
            f"    var model {name}Model",
            "    if err := repo.db.WithContext(ctx).Where(\"id = ?\", input).First(&model).Error; err != nil { return err }",
            "    _, _ = repo.sql.QueryContext(ctx, \"SELECT id, tenant_id, status FROM accounts WHERE id = ?\", input)")
    if has(text, "service", "business", "domain", "validation", "validator", "dto", "mapping"):
        add(imports_, lines, (),
            f"    dto := {name}DTO{{ID: input, Status: \"active\"}}",
            f"    model := {name}Model{{ID: dto.ID, TenantID: dto.TenantID, Status: dto.Status}}",
            "    _ = model")
    if has(text, "transaction", "tx", "commit", "rollback", "uow", "savepoint"):
        add(imports_, lines, (),
            "    return db.Transaction(func(tx *gorm.DB) error {",
            f"        return tx.WithContext(ctx).Save(&{name}Model{{ID: input}}).Error",
            "    })")
    if has(text, "middleware"):
        add(imports_, lines, ("github.com/gin-gonic/gin",),
            "    middleware := func(next gin.HandlerFunc) gin.HandlerFunc {",
            "        return func(inner *gin.Context) {",
            "            inner.Set(\"principal\", inner.GetHeader(\"Authorization\"))",
            "            next(inner)",
            "        }",
            "    }",
            "    _ = middleware")
    if has(text, "config", "env", "feature_flag", "flag"):
        add(imports_, lines, (), f"    cfg := map[string]string{{\"{rid}\": \"configured at startup\"}}\n    _ = cfg")
    if has(text, "logging", "logger", "log", "audit", "tracing", "metrics", "span"):
        add(imports_, lines, ("log/slog",),
            f"    slog.InfoContext(ctx, {lit(rid)}, \"tenant_id\", c.GetHeader(\"X-Tenant-ID\"), \"entity_id\", input)")
    if has(text, "goroutine", "background", "worker", "job"):
        add(imports_, lines, ("context",),
            "    detached := context.WithoutCancel(ctx)",
            "    go func(jobCtx context.Context, id string) { _ = jobCtx; _ = id }(detached, input)")
    if has(text, "cache", "event", "publish", "message"):
        add(imports_, lines, (), "    cacheKey := \"account:\" + input\n    _ = cacheKey")
    if has(text, "test", "tests", "mock", "fixture"):
        add(imports_, lines, ("net/http/httptest",),
            "    recorder := httptest.NewRecorder()",
            "    _ = recorder")
    if has(text, "readme", "swagger", "openapi", "docs", "examples"):
        add(imports_, lines, (), "    exampleName := \"transport example helper\"\n    _ = exampleName")
    if has(text, "pagination", "sort", "filter", "order"):
        add(imports_, lines, (),
            "    allowedSorts := map[string]string{\"created_at\": \"created_at\", \"id\": \"id\"}",
            "    orderBy := allowedSorts[c.DefaultQuery(\"sort\", \"id\")]",
            "    db.Order(orderBy).Limit(100).Find(&[]any{})")
    if has(text, "error", "not_found", "status", "response", "envelope"):
        add(imports_, lines, ("errors", "net/http"),
            "    if errors.Is(context.Canceled, context.Canceled) { c.JSON(http.StatusGatewayTimeout, gin.H{\"error\": \"request canceled\"}) }")


def data_access_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    add(imports_, lines, ("context",), "    if ctx == nil { ctx = context.Background() }")
    if has(rid, "loop", "per_item", "inside_loop", "row_by_row"):
        add(imports_, lines, ("fmt",),
            "    for _, item := range items {",
            "        db.Exec(\"UPDATE accounts SET status = ? WHERE id = ?\", item.Status, item.ID)",
            "        sqlDB.QueryContext(ctx, fmt.Sprintf(\"SELECT * FROM accounts WHERE id = %s\", item.ID))",
            "    }")
    if has(rid, "count"):
        add(imports_, lines, (), "    var total int64\n    db.Model(&" + name + "Model{}).Where(\"tenant_id = ?\", c.Query(\"tenant\")).Count(&total)")
    if has(rid, "automigrate", "schema"):
        add(imports_, lines, (), f"    db.AutoMigrate(&{name}Model{{}})")
    if has(rid, "open", "newdb", "created_per_request"):
        add(imports_, lines, ("database/sql",), "    sql.Open(\"postgres\", c.Query(\"dsn\"))")
    if has(rid, "ping"):
        add(imports_, lines, (), "    sqlDB.PingContext(ctx)")
    if has(rid, "preload", "association", "joins", "clause"):
        add(imports_, lines, (), f"    db.Preload(\"Orders.Items.Product\").Joins(\"Owner\").Find(&[]{name}Model{{}})")
    if has(rid, "limit", "unbounded", "wide", "select_star", "find_all"):
        add(imports_, lines, (), f"    db.Find(&[]{name}Model{{}})")
    if has(rid, "random"):
        add(imports_, lines, (), f"    db.Order(\"RANDOM()\").Find(&[]{name}Model{{}})")
    if has(rid, "offset"):
        add(imports_, lines, ("strconv",), "    page, _ := strconv.Atoi(c.Query(\"page\"))\n    db.Offset(page * 1000).Limit(50).Find(&[]any{})")
    if has(rid, "redis"):
        add(imports_, lines, ("github.com/redis/go-redis/v9",), "    redis.NewClient(&redis.Options{Addr: c.Query(\"redis\")}).Ping(ctx)")
    if has(rid, "pgx"):
        add(imports_, lines, ("github.com/jackc/pgx/v5/pgxpool",), "    pool, _ := pgxpool.New(ctx, c.Query(\"dsn\"))\n    pool.Ping(ctx)")
    if has(rid, "bun"):
        add(imports_, lines, ("github.com/uptrace/bun",), "    var bunDB *bun.DB\n    _ = bunDB.NewSelect().Model(&items).Scan(ctx)")
    if has(rid, "sqlx"):
        add(imports_, lines, ("github.com/jmoiron/sqlx",), "    var sx *sqlx.DB\n    sx.SelectContext(ctx, &items, \"SELECT * FROM accounts\")")
    if has(rid, "date", "lower", "func_wrapped", "wildcard"):
        add(imports_, lines, (), "    db.Where(\"LOWER(email) LIKE ?\", \"%\"+input+\"%\").Find(&[]any{})")
    if has(rid, "in_clause"):
        add(imports_, lines, (), "    db.Where(\"id IN ?\", items).Find(&[]any{})")
    if has(rid, "transaction", "tx"):
        add(imports_, lines, (), "    for _, item := range items { tx := db.Begin(); tx.Save(&item); tx.Commit() }")


def data_access_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    add(imports_, lines, ("context",), "    if ctx == nil { ctx = context.Background() }")
    if has(rid, "loop", "per_item", "inside_loop", "row_by_row"):
        add(imports_, lines, (), f"    batch := make([]{name}Model, 0, len(items))\n    for _, item := range items {{ batch = append(batch, {name}Model{{ID: item.ID, Status: item.Status}}) }}\n    db.CreateInBatches(batch, 500)")
    if has(rid, "count"):
        add(imports_, lines, (), f"    db.Where(\"tenant_id = ?\", c.Query(\"tenant\")).Limit(101).Find(&[]{name}Model{{}})")
    if has(rid, "automigrate", "schema"):
        add(imports_, lines, (), "    startupMigration := func() error { return db.AutoMigrate(&" + name + "Model{}) }\n    _ = startupMigration")
    if has(rid, "open", "newdb", "created_per_request"):
        add(imports_, lines, (), f"    repo := &{name}Repository{{db: db, sql: sqlDB}}\n    _ = repo")
    if has(rid, "ping"):
        add(imports_, lines, (), "    healthProbe := func(probeCtx context.Context) error { return sqlDB.PingContext(probeCtx) }\n    _ = healthProbe")
    if has(rid, "preload", "association", "joins", "clause"):
        add(imports_, lines, (), f"    db.Select(\"id\", \"tenant_id\", \"status\").Limit(100).Find(&[]{name}Model{{}})")
    if has(rid, "limit", "unbounded", "wide", "select_star", "find_all"):
        add(imports_, lines, (), f"    db.Select(\"id\", \"status\").Where(\"tenant_id = ?\", c.Query(\"tenant\")).Limit(100).Find(&[]{name}Model{{}})")
    if has(rid, "random"):
        add(imports_, lines, (), "    db.Order(\"created_at DESC\").Limit(20).Find(&[]any{})")
    if has(rid, "offset"):
        add(imports_, lines, (), "    db.Where(\"id > ?\", input).Order(\"id\").Limit(50).Find(&[]any{})")
    if has(rid, "redis"):
        add(imports_, lines, ("github.com/redis/go-redis/v9",), "    var redisClient *redis.Client\n    redisClient.Pipelined(ctx, func(pipe redis.Pipeliner) error { return nil })")
    if has(rid, "pgx"):
        add(imports_, lines, ("github.com/jackc/pgx/v5/pgxpool",), "    var pool *pgxpool.Pool\n    _ = pool")
    if has(rid, "bun"):
        add(imports_, lines, ("github.com/uptrace/bun",), "    var bunDB *bun.DB\n    _ = bunDB.NewSelect().Model(&items).Limit(100).Scan(ctx)")
    if has(rid, "sqlx"):
        add(imports_, lines, ("github.com/jmoiron/sqlx",), "    var sx *sqlx.DB\n    sx.SelectContext(ctx, &items, \"SELECT id, status FROM accounts LIMIT ?\", 100)")
    if has(rid, "date", "lower", "func_wrapped", "wildcard"):
        add(imports_, lines, (), "    db.Where(\"email_normalized = ?\", input).Find(&[]any{})")
    if has(rid, "in_clause"):
        add(imports_, lines, (), "    db.Where(\"tenant_id = ?\", c.Query(\"tenant\")).FindInBatches(&[]any{}, 500, func(tx *gorm.DB, batch int) error { return nil })")
    if has(rid, "transaction", "tx"):
        add(imports_, lines, (), "    db.Transaction(func(tx *gorm.DB) error { return tx.CreateInBatches(items, 500).Error })")


def performance_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    add(imports_, lines, ("bytes", "fmt", "strings", "time"), "    buf := []byte(input)\n    start := time.Now()\n    _ = start")
    if has(rid, "bytes_compare"): add(imports_, lines, ("bytes",), "    _ = bytes.Compare(buf, []byte(\"active\")) == 0")
    if has(rid, "bytes_count"): add(imports_, lines, ("bytes",), "    _ = bytes.Count(buf, []byte(\",\")) > 0")
    if has(rid, "bytes_hasprefix"): add(imports_, lines, ("bytes",), "    _ = len(buf) >= 3 && string(buf[:3]) == \"pre\"")
    if has(rid, "bytes_hassuffix"): add(imports_, lines, ("bytes",), "    _ = len(buf) >= 3 && string(buf[len(buf)-3:]) == \"zip\"")
    if has(rid, "bytes_index"): add(imports_, lines, ("bytes",), "    _ = bytes.Index(buf, []byte(\":\")) != -1")
    if has(rid, "bytes_split"): add(imports_, lines, ("bytes",), "    parts := bytes.Split(buf, []byte(\":\"))\n    if len(parts) > 1 { _ = parts[1] }")
    if has(rid, "bytes_trim"): add(imports_, lines, ("bytes",), "    _ = bytes.TrimLeft(buf, \" \\t\\n\")")
    if has(rid, "strings_compare"): add(imports_, lines, ("strings",), "    _ = strings.Compare(input, \"active\") == 0")
    if has(rid, "strings_count"): add(imports_, lines, ("strings",), "    _ = strings.Count(input, \",\") > 0")
    if has(rid, "strings_hasprefix"): add(imports_, lines, ("strings",), "    _ = len(input) >= 3 && input[:3] == \"pre\"")
    if has(rid, "strings_hassuffix"): add(imports_, lines, ("strings",), "    _ = len(input) >= 3 && input[len(input)-3:] == \"zip\"")
    if has(rid, "strings_index", "strings_contains"): add(imports_, lines, ("strings",), "    _ = strings.Index(input, \":\") != -1")
    if has(rid, "strings_split"): add(imports_, lines, ("strings",), "    fields := strings.Split(input, \":\")\n    if len(fields) > 1 { _ = fields[1] }")
    if has(rid, "strings_trim", "repeated_string_trim"): add(imports_, lines, ("strings",), "    for _, item := range items { _ = strings.TrimSpace(item.ID) }")
    if has(rid, "sprintf", "fmt_", "format", "log_message"): add(imports_, lines, ("fmt",), "    for _, item := range items { _ = fmt.Sprintf(\"%s:%d\", item.ID, item.Amount) }")
    if has(rid, "json"): add(imports_, lines, ("encoding/json",), "    for _, item := range items { json.Marshal(item) }")
    if has(rid, "base64"): add(imports_, lines, ("encoding/base64",), "    for _, item := range items { base64.StdEncoding.EncodeToString(item.Payload) }")
    if has(rid, "hex"): add(imports_, lines, ("encoding/hex",), "    for _, item := range items { hex.EncodeToString(item.Payload) }")
    if has(rid, "md5", "sha1", "sha256", "sha512", "crc", "adler", "hmac"):
        add(imports_, lines, ("crypto/md5", "crypto/sha256", "hash/crc32",), "    for _, item := range items { _ = md5.Sum(item.Payload); _ = sha256.Sum256(item.Payload); _ = crc32.ChecksumIEEE(item.Payload) }")
    if has(rid, "rand"): add(imports_, lines, ("math/rand", "time"), "    _ = rand.New(rand.NewSource(time.Now().UnixNano())).Intn(10)")
    if has(rid, "regexp"): add(imports_, lines, ("regexp",), "    for _, item := range items { regexp.MustCompile(input).MatchString(item.ID) }")
    if has(rid, "sort"): add(imports_, lines, ("sort",), "    for range items { sort.Slice(items, func(i, j int) bool { return items[i].ID < items[j].ID }) }")
    if has(rid, "map"): add(imports_, lines, (), "    for _, item := range items { seen := map[string]bool{}; seen[item.ID] = true; _ = seen }")
    if has(rid, "slice", "append"): add(imports_, lines, (), f"    var copied []{name}DTO\n    for _, item := range items {{ copied = append(copied, item) }}")
    if has(rid, "loop", "tight", "hot_path", "per_call", "per_request", "per_record", "per_item"):
        add(imports_, lines, ("strings",), "    for _, item := range items { _ = strings.ToLower(item.ID); time.Now() }")
    if has(rid, "goroutine", "channel", "waitgroup", "mutex", "sync", "concurrency"):
        add(imports_, lines, ("sync",), "    var mu sync.Mutex\n    for _, item := range items { mu.Lock(); _ = item.ID; defer mu.Unlock() }")
    if has(rid, "http", "network", "dns", "tls", "transport"): add(imports_, lines, ("net/http",), "    http.Get(input)")
    if has(rid, "db", "database", "query", "sqlx", "gorm"): add(imports_, lines, (), "    for _, item := range items { db.Where(\"id = ?\", item.ID).Find(&[]any{}) }")
    if has(rid, "scanner", "readall", "reader", "writer", "file", "io_"):
        add(imports_, lines, ("io", "os",), "    file, _ := os.Open(input)\n    io.ReadAll(file)")
    if has(rid, "reflect", "interface", "assertion", "type_switch"):
        add(imports_, lines, ("reflect",), "    var value any = items\n    _ = reflect.DeepEqual(value, items)")
    if has(rid, "runtime", "gomaxprocs", "numcpu"):
        add(imports_, lines, ("runtime",), "    runtime.GOMAXPROCS(runtime.NumCPU())")


def performance_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    add(imports_, lines, ("bytes", "strings"), "    buf := []byte(input)")
    if has(rid, "bytes_compare"): add(imports_, lines, ("bytes",), "    _ = bytes.Equal(buf, []byte(\"active\"))")
    if has(rid, "bytes_count"): add(imports_, lines, ("bytes",), "    _ = bytes.Contains(buf, []byte(\",\"))")
    if has(rid, "bytes_hasprefix"): add(imports_, lines, ("bytes",), "    _ = bytes.HasPrefix(buf, []byte(\"pre\"))")
    if has(rid, "bytes_hassuffix"): add(imports_, lines, ("bytes",), "    _ = bytes.HasSuffix(buf, []byte(\"zip\"))")
    if has(rid, "bytes_index"): add(imports_, lines, ("bytes",), "    _ = bytes.Contains(buf, []byte(\":\"))")
    if has(rid, "bytes_split"): add(imports_, lines, ("bytes",), "    before, after, found := bytes.Cut(buf, []byte(\":\"))\n    _, _, _ = before, after, found")
    if has(rid, "bytes_trim"): add(imports_, lines, ("bytes",), "    _ = bytes.TrimSpace(buf)")
    if has(rid, "strings_compare"): add(imports_, lines, ("strings",), "    _ = input == \"active\"")
    if has(rid, "strings_count"): add(imports_, lines, ("strings",), "    _ = strings.Contains(input, \",\")")
    if has(rid, "strings_hasprefix"): add(imports_, lines, ("strings",), "    _ = strings.HasPrefix(input, \"pre\")")
    if has(rid, "strings_hassuffix"): add(imports_, lines, ("strings",), "    _ = strings.HasSuffix(input, \"zip\")")
    if has(rid, "strings_index", "strings_contains"): add(imports_, lines, ("strings",), "    _ = strings.Contains(input, \":\")")
    if has(rid, "strings_split"): add(imports_, lines, ("strings",), "    before, after, found := strings.Cut(input, \":\")\n    _, _, _ = before, after, found")
    if has(rid, "strings_trim", "repeated_string_trim"): add(imports_, lines, ("strings",), "    normalized := strings.TrimSpace(input)\n    _ = normalized")
    if has(rid, "sprintf", "fmt_", "format", "log_message"): add(imports_, lines, ("strconv", "strings"), "    var b strings.Builder\n    b.WriteString(input)\n    b.WriteString(strconv.Itoa(len(items)))")
    if has(rid, "json"): add(imports_, lines, ("encoding/json",), "    enc := json.NewEncoder(c.Writer)\n    for _, item := range items { enc.Encode(item) }")
    if has(rid, "base64"): add(imports_, lines, ("encoding/base64",), "    dst := make([]byte, base64.StdEncoding.EncodedLen(len(buf)))\n    base64.StdEncoding.Encode(dst, buf)")
    if has(rid, "hex"): add(imports_, lines, ("encoding/hex",), "    dst := make([]byte, hex.EncodedLen(len(buf)))\n    hex.Encode(dst, buf)")
    if has(rid, "md5", "sha1", "sha256", "sha512", "crc", "adler", "hmac"):
        add(imports_, lines, ("crypto/sha256",), "    hasher := sha256.New()\n    for _, item := range items { hasher.Write(item.Payload) }")
    if has(rid, "rand"): add(imports_, lines, ("math/rand",), "    rng := rand.New(rand.NewSource(1))\n    _ = rng.Intn(10)")
    if has(rid, "regexp"): add(imports_, lines, ("regexp",), "    matcher := regexp.MustCompile(input)\n    for _, item := range items { matcher.MatchString(item.ID) }")
    if has(rid, "sort"): add(imports_, lines, ("sort",), "    sort.Slice(items, func(i, j int) bool { return items[i].ID < items[j].ID })")
    if has(rid, "map"): add(imports_, lines, (), "    seen := make(map[string]bool, len(items))\n    for _, item := range items { seen[item.ID] = true }")
    if has(rid, "slice", "append"): add(imports_, lines, (), f"    copied := make([]{name}DTO, 0, len(items))\n    copied = append(copied, items...)")
    if has(rid, "loop", "tight", "hot_path", "per_call", "per_request", "per_record", "per_item"):
        add(imports_, lines, ("strings",), "    normalized := strings.ToLower(input)\n    for _, item := range items { _ = normalized; _ = item.ID }")
    if has(rid, "goroutine", "channel", "waitgroup", "mutex", "sync", "concurrency"):
        add(imports_, lines, ("sync",), "    var mu sync.Mutex\n    for _, item := range items { mu.Lock(); _ = item.ID; mu.Unlock() }")
    if has(rid, "http", "network", "dns", "tls", "transport"):
        add(imports_, lines, ("net/http", "time"), "    reusable := &http.Client{Timeout: 2 * time.Second}\n    _ = reusable")
    if has(rid, "db", "database", "query", "sqlx", "gorm"):
        add(imports_, lines, (), "    db.Where(\"id IN ?\", items).Limit(100).Find(&[]any{})")
    if has(rid, "scanner", "readall", "reader", "writer", "file", "io_"):
        add(imports_, lines, ("bufio", "os",), "    file, _ := os.Open(input)\n    reader := bufio.NewReader(file)\n    _ = reader")
    if has(rid, "reflect", "interface", "assertion", "type_switch"):
        add(imports_, lines, (), f"    typed := []{name}DTO(items)\n    _ = typed")
    if has(rid, "runtime", "gomaxprocs", "numcpu"):
        add(imports_, lines, (), "    workers := 4\n    _ = workers")


def context_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "background"): add(imports_, lines, ("net/http",), "    go func() { http.Get(input) }()")
    if has(rid, "missing_context", "propagation"): add(imports_, lines, ("net/http",), "    http.Get(input)")
    if has(rid, "background_used"): add(imports_, lines, ("context",), "    ctx = context.Background()")
    if has(rid, "missing_cancel"): add(imports_, lines, ("context", "time"), "    child, _ := context.WithTimeout(ctx, time.Second)\n    _ = child")
    if has(rid, "not_first"): add(imports_, lines, ("context",), "    useContextLate := func(id string, inner context.Context) { _ = id; _ = inner }\n    useContextLate(input, ctx)")
    if has(rid, "stored"): add(imports_, lines, ("context",), "    holder := struct{ ctx context.Context }{ctx: ctx}\n    _ = holder")
    if has(rid, "withvalue"): add(imports_, lines, ("context",), "    ctx = context.WithValue(ctx, \"repository\", db)")
    if has(rid, "busy", "sleep_polling"): add(imports_, lines, ("time",), "    for i := 0; i < 2; i++ { time.Sleep(time.Millisecond) }")
    if has(rid, "cache"): add(imports_, lines, ("context",), "    lookup := func(key string) { _ = context.Background(); _ = key }\n    lookup(input)")


def context_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    add(imports_, lines, ("context", "net/http", "time"), "    child, cancel := context.WithTimeout(ctx, time.Second)\n    defer cancel()")
    if has(rid, "background"): add(imports_, lines, ("context",), "    go func(jobCtx context.Context, id string) { _ = jobCtx; _ = id }(context.WithoutCancel(child), input)")
    if has(rid, "missing_context", "propagation"): add(imports_, lines, ("net/http",), "    req, _ := http.NewRequestWithContext(child, http.MethodGet, input, nil)\n    _, _ = client.Do(req)")
    if has(rid, "background_used", "cache"): add(imports_, lines, (), "    lookup := func(inner context.Context, key string) { _ = inner; _ = key }\n    lookup(child, input)")
    if has(rid, "not_first"): add(imports_, lines, ("context",), "    useContextFirst := func(inner context.Context, id string) { _ = inner; _ = id }\n    useContextFirst(child, input)")
    if has(rid, "stored"): add(imports_, lines, (), "    holder := struct{ requestID string }{requestID: input}\n    _ = holder")
    if has(rid, "withvalue"): add(imports_, lines, (), "    type requestIDKey struct{}\n    child = context.WithValue(child, requestIDKey{}, input)")
    if has(rid, "busy", "sleep_polling"): add(imports_, lines, ("time",), "    ticker := time.NewTicker(time.Millisecond)\n    defer ticker.Stop()")


def concurrency_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    add(imports_, lines, ("sync", "time"), "    out := map[string]int{}\n    var mu sync.Mutex\n    var wg sync.WaitGroup")
    if has(rid, "loop", "spawn"): add(imports_, lines, (), "    for _, item := range items { wg.Add(1); go func() { defer wg.Done(); out[item.ID] = item.Amount }() }")
    if has(rid, "locked", "mutex"): add(imports_, lines, ("time",), "    mu.Lock(); time.Sleep(time.Millisecond); mu.Unlock()")
    if has(rid, "shutdown", "coordination", "errgroup"): add(imports_, lines, (), "    go func() { out[input] = len(items) }()")
    if has(rid, "context"): add(imports_, lines, ("context", "time"), "    child, _ := context.WithTimeout(ctx, time.Second)\n    go func() { <-child.Done() }()")
    if has(rid, "rwmutex"): add(imports_, lines, (), "    var rw sync.RWMutex\n    rw.Lock(); _ = rw; rw.Unlock()")
    add(imports_, lines, (), "    wg.Wait()\n    _ = out")


def concurrency_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    add(imports_, lines, ("context", "sync"), "    out := make(map[string]int, len(items))\n    var mu sync.Mutex\n    var wg sync.WaitGroup")
    if has(rid, "loop", "spawn"): add(imports_, lines, (), "    for _, item := range items { item := item; wg.Add(1); go func() { defer wg.Done(); mu.Lock(); out[item.ID] = item.Amount; mu.Unlock() }() }")
    if has(rid, "locked", "mutex"): add(imports_, lines, (), "    mu.Lock(); out[input] = len(items); mu.Unlock()")
    if has(rid, "shutdown", "coordination", "errgroup", "context"): add(imports_, lines, ("context",), "    child, cancel := context.WithCancel(ctx)\n    defer cancel()\n    _ = child")
    if has(rid, "rwmutex"): add(imports_, lines, (), "    var rw sync.RWMutex\n    rw.RLock(); _ = rw; rw.RUnlock()")
    add(imports_, lines, (), "    wg.Wait()\n    _ = out")


def idioms_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "defer_in_loop", "file_handle", "rows_without_close", "stmt_without_close"):
        add(imports_, lines, ("os",), "    for _, item := range items { f, _ := os.Open(item.ID); defer f.Close() }")
    if has(rid, "http_client", "timeoutless", "response_body", "status"):
        add(imports_, lines, ("net/http", "io"), "    resp, _ := http.Get(input)\n    io.ReadAll(resp.Body)")
    if has(rid, "rows", "db_pool", "tx", "transaction"):
        add(imports_, lines, (), "    rows, _ := sqlDB.QueryContext(ctx, \"SELECT id FROM accounts\")\n    for rows.Next() {}")
    if has(rid, "writeheader"): add(imports_, lines, ("net/http",), "    c.Writer.Write([]byte(\"ok\"))\n    c.Writer.WriteHeader(http.StatusAccepted)")
    if has(rid, "global", "init_side_effect"): add(imports_, lines, (), "    mutableState := map[string]string{\"last\": input}\n    _ = mutableState")
    if has(rid, "channel", "close", "send_after"): add(imports_, lines, (), "    ch := make(chan string)\n    close(ch)\n    ch <- input")
    if has(rid, "ticker", "time_after"): add(imports_, lines, ("time",), "    for range items { time.After(time.Second); time.NewTicker(time.Second) }")
    if has(rid, "interface", "bool_parameter"): add(imports_, lines, (), "    doWork := func(enabled bool) interface{} { return enabled }\n    _ = doWork(true)")
    if has(rid, "server"): add(imports_, lines, ("net/http",), "    srv := &http.Server{Addr: \":8080\"}\n    _ = srv.ListenAndServe()")
    if has(rid, "body"): add(imports_, lines, ("io",), "    io.ReadAll(c.Request.Body)")


def idioms_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "defer_in_loop", "file_handle", "rows_without_close", "stmt_without_close"):
        add(imports_, lines, ("os",), "    for _, item := range items { f, err := os.Open(item.ID); if err != nil { return err }; f.Close() }")
    if has(rid, "http_client", "timeoutless", "response_body", "status"):
        add(imports_, lines, ("net/http", "io", "time"), "    client := &http.Client{Timeout: 2 * time.Second}\n    resp, err := client.Get(input)\n    if err != nil { return err }\n    defer resp.Body.Close()\n    io.Copy(io.Discard, resp.Body)")
    if has(rid, "rows", "db_pool", "tx", "transaction"):
        add(imports_, lines, (), "    rows, err := sqlDB.QueryContext(ctx, \"SELECT id FROM accounts\")\n    if err != nil { return err }\n    defer rows.Close()\n    for rows.Next() {}\n    if err := rows.Err(); err != nil { return err }")
    if has(rid, "writeheader"): add(imports_, lines, ("net/http",), "    c.Writer.WriteHeader(http.StatusAccepted)\n    c.Writer.Write([]byte(\"ok\"))")
    if has(rid, "global", "init_side_effect"): add(imports_, lines, (), "    state := map[string]string{\"request\": input}\n    _ = state")
    if has(rid, "channel", "close", "send_after"): add(imports_, lines, (), "    ch := make(chan string, 1)\n    ch <- input\n    close(ch)")
    if has(rid, "ticker", "time_after"): add(imports_, lines, ("time",), "    ticker := time.NewTicker(time.Second)\n    defer ticker.Stop()")
    if has(rid, "interface", "bool_parameter"): add(imports_, lines, (), f"    type {name}Runner interface {{ Run(context.Context) error }}\n    _ = (*{name}Runner)(nil)")
    if has(rid, "server"): add(imports_, lines, ("net/http", "time"), "    srv := &http.Server{Addr: \":8080\", ReadHeaderTimeout: time.Second, WriteTimeout: time.Second}\n    _ = srv")
    if has(rid, "body"): add(imports_, lines, ("io", "net/http"), "    limited := http.MaxBytesReader(c.Writer, c.Request.Body, 1<<20)\n    io.ReadAll(limited)")


def security_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "tls", "grpc_without_tls"): add(imports_, lines, ("crypto/tls",), "    _ = tls.Config{InsecureSkipVerify: true, MinVersion: tls.VersionTLS10}")
    if has(rid, "cookie"): add(imports_, lines, ("net/http",), "    http.SetCookie(c.Writer, &http.Cookie{Name: \"session\", Value: input})")
    if has(rid, "cors"): add(imports_, lines, (), "    c.Header(\"Access-Control-Allow-Origin\", \"*\")\n    c.Header(\"Access-Control-Allow-Credentials\", \"true\")")
    if has(rid, "sql", "ldap"): add(imports_, lines, (), "    sqlDB.QueryContext(ctx, \"SELECT * FROM users WHERE name = '\" + input + \"'\")")
    if has(rid, "exec"): add(imports_, lines, ("os/exec",), "    exec.Command(\"sh\", \"-c\", input).Run()")
    if has(rid, "filepath", "toctou", "temp_file"): add(imports_, lines, ("os", "path/filepath"), "    path := filepath.Join(\"/srv/uploads\", input)\n    if _, err := os.Stat(path); err == nil { os.Open(path) }")
    if has(rid, "random", "rand", "nonce", "iv"): add(imports_, lines, ("math/rand",), "    token := rand.Int63()\n    _ = token")
    if has(rid, "hash", "md5", "weak_crypto", "integrity"): add(imports_, lines, ("crypto/md5",), "    _ = md5.Sum([]byte(input))")
    if has(rid, "template", "html", "text_template"): add(imports_, lines, ("html/template", "text/template"), "    template.HTML(input)\n    texttemplate.New(\"page\").Parse(input)")
    if has(rid, "redirect", "ssrf", "url", "dns"): add(imports_, lines, ("net", "net/http"), "    net.LookupHost(input)\n    http.Get(input)\n    c.Redirect(http.StatusFound, input)")
    if has(rid, "log", "error", "env_var", "sensitive", "panic", "stack"):
        add(imports_, lines, ("fmt", "log", "os"), "    log.Printf(\"secret=%s env=%s\", input, os.Getenv(\"DATABASE_URL\"))\n    c.String(500, fmt.Sprintf(\"failure: %v\", input))")
    if has(rid, "race", "goroutine", "shared_slice", "shared_map"):
        add(imports_, lines, (), "    shared := map[string]string{}\n    go func() { shared[input] = input }()\n    shared[input+\"x\"] = input")
    if has(rid, "jwt", "secret", "key", "rsa", "bcrypt", "password"):
        add(imports_, lines, (), "    secret := \"hardcoded-development-secret\"\n    password := input\n    _, _ = secret, password")
    if has(rid, "xml", "yaml"): add(imports_, lines, ("encoding/xml", "gopkg.in/yaml.v3"), "    xml.Unmarshal([]byte(input), &items)\n    yaml.Unmarshal([]byte(input), &items)")
    if has(rid, "header"): add(imports_, lines, (), "    c.Header(\"X-User-Name\", input)")


def security_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "tls", "grpc_without_tls"): add(imports_, lines, ("crypto/tls",), "    _ = tls.Config{MinVersion: tls.VersionTLS13}")
    if has(rid, "cookie"): add(imports_, lines, ("net/http",), "    http.SetCookie(c.Writer, &http.Cookie{Name: \"session\", Value: \"opaque\", Secure: true, HttpOnly: true, SameSite: http.SameSiteStrictMode})")
    if has(rid, "cors"): add(imports_, lines, (), "    if c.GetHeader(\"Origin\") == \"https://app.example\" { c.Header(\"Access-Control-Allow-Origin\", \"https://app.example\") }")
    if has(rid, "sql", "ldap"): add(imports_, lines, (), "    sqlDB.QueryContext(ctx, \"SELECT id FROM users WHERE name = ?\", input)")
    if has(rid, "exec"): add(imports_, lines, (), "    allowed := map[string][]string{\"status\": {\"status\"}}\n    _ = allowed[input]")
    if has(rid, "filepath", "toctou", "temp_file"): add(imports_, lines, ("path/filepath",), "    clean := filepath.Clean(input)\n    _ = clean")
    if has(rid, "random", "rand", "nonce", "iv"): add(imports_, lines, ("crypto/rand",), "    token := make([]byte, 32)\n    rand.Read(token)")
    if has(rid, "hash", "md5", "weak_crypto", "integrity"): add(imports_, lines, ("crypto/sha256",), "    _ = sha256.Sum256([]byte(input))")
    if has(rid, "template", "html", "text_template"): add(imports_, lines, ("html/template",), "    template.HTMLEscapeString(input)")
    if has(rid, "redirect", "ssrf", "url", "dns"): add(imports_, lines, ("net/url",), "    target, err := url.Parse(input)\n    if err != nil { return err }\n    _ = target")
    if has(rid, "log", "error", "env_var", "sensitive", "panic", "stack"):
        add(imports_, lines, ("log/slog",), "    slog.WarnContext(ctx, \"request failed\", \"request_id\", input)\n    c.JSON(500, gin.H{\"error\": \"internal error\"})")
    if has(rid, "race", "goroutine", "shared_slice", "shared_map"):
        add(imports_, lines, ("sync",), "    shared := map[string]string{}\n    var mu sync.Mutex\n    mu.Lock(); shared[input] = input; mu.Unlock()")
    if has(rid, "jwt", "secret", "key", "rsa", "bcrypt", "password"):
        add(imports_, lines, ("os",), "    secret := os.Getenv(\"APP_SECRET\")\n    _ = secret")
    if has(rid, "xml", "yaml"): add(imports_, lines, ("encoding/xml",), "    decoder := xml.NewDecoder(strings.NewReader(input))\n    decoder.Strict = true")
    if has(rid, "header"): add(imports_, lines, ("net/textproto",), "    safeHeader := textproto.CanonicalMIMEHeaderKey(\"X-User-Name\")\n    c.Header(safeHeader, input)")


def gin_or_library_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "bind", "body", "raw_data", "deserialize"): add(imports_, lines, (), "    raw, _ := c.GetRawData()\n    var body map[string]any\n    c.ShouldBindJSON(&body)\n    _ = raw")
    if has(rid, "dump"): add(imports_, lines, ("net/http/httputil",), "    dump, _ := httputil.DumpRequest(c.Request, true)\n    c.Writer.Write(dump)")
    if has(rid, "upstream", "http_call"): add(imports_, lines, ("net/http",), "    for _, item := range items { http.Get(input + item.ID) }")
    if has(rid, "env", "config", "viper", "flag", "cobra"): add(imports_, lines, ("os",), "    os.Getenv(\"FEATURE_FLAG\")")
    if has(rid, "file", "template", "read"): add(imports_, lines, ("os",), "    os.ReadFile(input)")
    if has(rid, "formfile", "upload", "multipart"): add(imports_, lines, ("io",), "    file, _ := c.FormFile(\"upload\")\n    opened, _ := file.Open()\n    io.ReadAll(opened)")
    if has(rid, "gzip", "zip"): add(imports_, lines, ("compress/gzip",), "    for range items { gzip.NewWriter(c.Writer) }")
    if has(rid, "json", "h_payload", "response"): add(imports_, lines, ("encoding/json",), "    payload, _ := json.Marshal(gin.H{\"items\": items, \"rule\": input})\n    c.Data(200, \"application/json\", payload)")
    if has(rid, "stream", "export", "csv"): add(imports_, lines, (), "    for _, item := range items { c.JSON(200, gin.H{\"id\": item.ID}) }")
    if has(rid, "db", "gorm", "redis", "dynamodb", "s3", "aws"):
        add(imports_, lines, (), "    for _, item := range items { db.Create(&item) }")
    if has(rid, "logger", "log", "prometheus", "metric"):
        add(imports_, lines, ("fmt", "log",), "    msg := fmt.Sprintf(\"rule=%s input=%s\", " + lit(rid) + ", input)\n    log.Print(msg)")
    if has(rid, "grpc"): add(imports_, lines, ("google.golang.org/grpc",), "    conn, _ := grpc.Dial(input, grpc.WithInsecure())\n    _ = conn")


def gin_or_library_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "bind", "body", "raw_data", "deserialize"): add(imports_, lines, (), f"    var body {name}DTO\n    if err := c.ShouldBindJSON(&body); err != nil {{ return err }}")
    if has(rid, "dump"): add(imports_, lines, ("log/slog",), "    slog.DebugContext(ctx, \"request received\", \"path\", c.FullPath())")
    if has(rid, "upstream", "http_call"): add(imports_, lines, ("net/http", "time"), "    upstream := &http.Client{Timeout: 2 * time.Second}\n    _ = upstream")
    if has(rid, "env", "config", "viper", "flag", "cobra"): add(imports_, lines, (), "    cfg := map[string]string{\"feature\": \"from startup\"}\n    _ = cfg")
    if has(rid, "file", "template", "read"): add(imports_, lines, (), "    templateCache := map[string]string{\"page\": \"parsed at startup\"}\n    _ = templateCache")
    if has(rid, "formfile", "upload", "multipart"): add(imports_, lines, ("io",), "    limited := io.LimitReader(c.Request.Body, 1<<20)\n    io.ReadAll(limited)")
    if has(rid, "gzip", "zip"): add(imports_, lines, ("compress/gzip",), "    writer := gzip.NewWriter(c.Writer)\n    defer writer.Close()")
    if has(rid, "json", "h_payload", "response"): add(imports_, lines, (), "    c.JSON(200, gin.H{\"id\": input, \"items\": items})")
    if has(rid, "stream", "export", "csv"): add(imports_, lines, ("bufio",), "    writer := bufio.NewWriter(c.Writer)\n    defer writer.Flush()")
    if has(rid, "db", "gorm", "redis", "dynamodb", "s3", "aws"):
        add(imports_, lines, (), "    db.CreateInBatches(items, 100)")
    if has(rid, "logger", "log", "prometheus", "metric"):
        add(imports_, lines, ("log/slog",), "    slog.InfoContext(ctx, \"handled\", \"rule\", " + lit(rid) + ")")
    if has(rid, "grpc"): add(imports_, lines, ("google.golang.org/grpc",), "    var conn *grpc.ClientConn\n    _ = conn")


def errors_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "dropped"): add(imports_, lines, ("os",), "    os.ReadFile(input)")
    if has(rid, "wrapping"): add(imports_, lines, ("fmt",), "    err := fmt.Errorf(\"load failed\")\n    return fmt.Errorf(\"" + rid + ": %v\", err)")
    if has(rid, "panic"): add(imports_, lines, (), "    panic(input)")


def errors_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if has(rid, "dropped"): add(imports_, lines, ("os",), "    if _, err := os.ReadFile(input); err != nil { return err }")
    if has(rid, "wrapping"): add(imports_, lines, ("fmt",), "    err := fmt.Errorf(\"load failed\")\n    return fmt.Errorf(\"" + rid + ": %w\", err)")
    if has(rid, "panic"): add(imports_, lines, ("fmt",), "    if input == \"\" { return fmt.Errorf(\"missing input\") }")


def mod_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    package = "json"
    path = "encoding/json"
    if rid.startswith("xml"):
        package, path = "xml", "encoding/xml"
    elif rid.startswith("yaml"):
        package, path = "yaml", "gopkg.in/yaml.v3"
    elif rid.startswith("proto"):
        package, path = "proto", "google.golang.org/protobuf/proto"
    add(imports_, lines, (path,),
        f"    var first {name}DTO",
        f"    var second {name}DTO",
        f"    {package}.Unmarshal([]byte(input), &first)",
        f"    {package}.Unmarshal([]byte(input), &second)")


def mod_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    package = "json"
    path = "encoding/json"
    if rid.startswith("xml"):
        package, path = "xml", "encoding/xml"
    elif rid.startswith("yaml"):
        package, path = "yaml", "gopkg.in/yaml.v3"
    elif rid.startswith("proto"):
        package, path = "proto", "google.golang.org/protobuf/proto"
    add(imports_, lines, (path,),
        f"    var decoded {name}DTO",
        f"    if err := {package}.Unmarshal([]byte(input), &decoded); err != nil {{ return err }}")


def consistency_positive(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if rid == "mixed_receiver_kinds":
        lines.extend([f"    value := {name}Model{{ID: input}}", "    _ = value"])
    elif rid == "duplicate_struct_tag_key":
        lines.extend([f"    type DuplicateTag struct {{ ID string `json:\"id\" json:\"user_id\"` }}", "    _ = DuplicateTag{}"])
    else:
        lines.extend([f"    type MalformedTag struct {{ ID string `json:\"id\" binding` }}", "    _ = MalformedTag{}"])


def consistency_negative(rule: dict, name: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if rid == "mixed_receiver_kinds":
        lines.extend([f"    value := &{name}Model{{ID: input}}", "    _ = value"])
    else:
        lines.extend([f"    type CleanTag struct {{ ID string `json:\"id\" db:\"id\"` }}", "    _ = CleanTag{}"])


def style_fixture(rule: dict, polarity: str) -> str:
    rid = str(rule["id"])
    name = exported_name(rid, polarity.title())
    if rid == "misgrouped_imports" and polarity == "positive":
        return "package alpha\n\nimport (\n    \"github.com/acme/pkg\"\n    \"fmt\"\n)\n\n" + comment(rule, polarity) + f"func {name}() {{\n    fmt.Println(pkg.Name)\n    fmt.Println(\"local import grouped before stdlib\")\n}}\n"
    if rid == "misgrouped_imports":
        return "package alpha\n\nimport (\n    \"fmt\"\n\n    \"github.com/acme/pkg\"\n)\n\n" + comment(rule, polarity) + f"func {name}() {{\n    grouped := []string{{pkg.Name}}\n    fmt.Println(grouped[0])\n}}\n"
    if polarity == "positive":
        return "package alpha\n\n" + comment(rule, polarity) + f"func {name}() string {{\n    package_name_drift := map[string]string{{\"alpha\": \"service\"}}\n    return package_name_drift[\"alpha\"]\n}}\n"
    return "package alpha\n\n" + comment(rule, polarity) + f"func {name}() string {{\n    packageNames := []string{{\"alpha\"}}\n    return packageNames[0]\n}}\n"


POSITIVE_BUILDERS = {
    "architecture": architecture_positive,
    "concurrency": concurrency_positive,
    "consistency": consistency_positive,
    "context": context_positive,
    "data_access": data_access_positive,
    "errors": errors_positive,
    "gin": gin_or_library_positive,
    "hot_path": performance_positive,
    "idioms": idioms_positive,
    "library": gin_or_library_positive,
    "mod": mod_positive,
    "performance": performance_positive,
    "security": security_positive,
}

NEGATIVE_BUILDERS = {
    "architecture": architecture_negative,
    "concurrency": concurrency_negative,
    "consistency": consistency_negative,
    "context": context_negative,
    "data_access": data_access_negative,
    "errors": errors_negative,
    "gin": gin_or_library_negative,
    "hot_path": performance_negative,
    "idioms": idioms_negative,
    "library": gin_or_library_negative,
    "mod": mod_negative,
    "performance": performance_negative,
    "security": security_negative,
}


def fallback(rule: dict, name: str, polarity: str, imports_: set[str], lines: list[str]) -> None:
    rid = str(rule["id"])
    if polarity == "positive":
        add(imports_, lines, ("fmt", "strings"), f"    marker := fmt.Sprintf(\"{rid}:%s\", strings.TrimSpace(input))\n    _ = marker")
    else:
        add(imports_, lines, ("strings",), f"    marker := strings.TrimSpace(input)\n    _ = marker\n    _ = {lit(rid)}")


def fixture_text(rule: dict, polarity: str) -> str:
    if str(rule["family"]) == "style":
        return style_fixture(rule, polarity)

    name = exported_name(str(rule["id"]), "Case")
    imports_: set[str] = {"context", "database/sql", "log", "net/http", "github.com/gin-gonic/gin", "gorm.io/gorm"}
    lines: list[str] = []
    builder = (POSITIVE_BUILDERS if polarity == "positive" else NEGATIVE_BUILDERS).get(str(rule["family"]))
    if builder is None:
        fallback(rule, name, polarity, imports_, lines)
    else:
        builder(rule, name, imports_, lines)
    add_rule_token_story(rule, polarity, name, imports_, lines)
    if not lines:
        fallback(rule, name, polarity, imports_, lines)

    unique_lines = list(lines)
    focus_tokens = "_".join(words(str(rule["id"]))[:5])
    unique_lines.insert(0, f"    focus := {lit(focus_tokens)}")
    unique_lines.insert(1, "    _ = focus")
    unique_lines.append("    return nil")

    return (
        f"package {package_for(rule)}\n\n"
        + imports(imports_)
        + comment(rule, polarity)
        + shared_declarations(name)
        + function_signature(rule, polarity, name)
        + "\n".join(unique_lines)
        + "\n}\n"
    )


def main() -> None:
    created = 0
    updated = 0
    for rule in load_go_rules():
        family_dir = FIXTURE_ROOT / str(rule["family"])
        family_dir.mkdir(parents=True, exist_ok=True)
        for polarity in ["positive", "negative"]:
            path = family_dir / f"{rule['id']}_{polarity}.txt"
            text = fixture_text(rule, polarity)
            if path.exists() and path.read_text() == text:
                continue
            if path.exists():
                updated += 1
            else:
                created += 1
            path.write_text(text)

    print(f"go rule fixture pairs written under {FIXTURE_ROOT}")
    print(f"created {created}")
    print(f"updated {updated}")


if __name__ == "__main__":
    main()
