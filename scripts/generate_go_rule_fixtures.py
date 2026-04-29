#!/usr/bin/env python3

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


def exported_name(rule_id: str, polarity: str) -> str:
    words = re.findall(r"[a-zA-Z0-9]+", rule_id)
    stem = "".join(word[:1].upper() + word[1:] for word in words)
    return f"{polarity}{stem}"[:180]


def comment(rule: dict, polarity: str) -> str:
    description = str(rule["description"]).replace("\n", " ").strip()
    label = "Positive" if polarity == "positive" else "Negative"
    intent = "shows the risky shape" if polarity == "positive" else "shows the preferred shape"
    return f"// {label} scenario for {rule['id']}: {intent} for this rule.\n// Rule intent: {description}\n"


def imports(*paths: str) -> str:
    unique = sorted(dict.fromkeys(paths))
    if not unique:
        return ""
    body = "\n".join(f'    "{path}"' for path in unique)
    return f"import (\n{body}\n)\n\n"


def package_for(rule: dict) -> str:
    rid = str(rule["id"])
    fam = str(rule["family"])
    if fam == "style":
        return "alpha"
    if "middleware" in rid:
        return "middleware"
    if "repository" in rid or "gorm" in rid or "sql_" in rid or "table_" in rid or "column_" in rid:
        return "repository"
    if "service" in rid:
        return "service"
    if "model" in rid or "entity" in rid:
        return "models"
    if "router" in rid or "route" in rid:
        return "router"
    if "cmd" in rid or "main" in rid or "bootstrap" in rid:
        return "main"
    if "handler" in rid or "transport" in rid or "dto" in rid or "api_" in rid:
        return "handler"
    return "rulecoverage"


def header(rule: dict, polarity: str, *import_paths: str) -> str:
    return f"package {package_for(rule)}\n\n" + imports(*import_paths) + comment(rule, polarity)


def generic_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "context", "net/http", "time")
        + f"type {name}Config struct {{\n    Timeout time.Duration\n}}\n\n"
        + f"func {name}(ctx context.Context, client *http.Client, cfg {name}Config) (*http.Response, error) {{\n"
        + "    req, err := http.NewRequestWithContext(ctx, http.MethodGet, \"https://example.test/status\", nil)\n"
        + "    if err != nil {\n        return nil, err\n    }\n"
        + "    if cfg.Timeout <= 0 {\n        cfg.Timeout = 2 * time.Second\n    }\n"
        + "    return client.Do(req)\n}\n"
    )


def architecture_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "database/sql", "errors", "fmt", "net/http", "os", "github.com/gin-gonic/gin", "gorm.io/gorm")
        + "type UserModel struct {\n"
        + "    ID string `json:\"id\" gorm:\"column:id\" binding:\"required\"`\n"
        + "    Status string `json:\"status,omitempty\" gorm:\"column:status\" form:\"status\"`\n"
        + "}\n\n"
        + "type APIError struct { Error string `json:\"error\"` }\n"
        + "type UserRepository struct { db *gorm.DB; sql *sql.DB }\n"
        + "type UserService struct { db *gorm.DB; repo *UserRepository; handler *UserHandler; cfg map[string]any }\n"
        + "type UserHandler struct { service *UserService; repo *UserRepository; ctx *gin.Context }\n\n"
        + f"func {name}(c *gin.Context, db *gorm.DB, sqlDB *sql.DB) {{\n"
        + "    var model UserModel\n"
        + "    if err := c.ShouldBindJSON(&model); err != nil {\n        c.JSON(http.StatusInternalServerError, gin.H{\"error\": err.Error()})\n        return\n    }\n"
        + "    tenantID := c.GetHeader(\"X-Tenant-ID\")\n"
        + "    userID := c.Param(\"id\")\n"
        + "    tx := db.Begin()\n"
        + "    tx.Where(\"tenant_id = ?\", tenantID).Where(\"id = ?\", userID).Find(&model)\n"
        + "    sqlDB.Query(\"SELECT * FROM users WHERE id = \" + userID)\n"
        + "    c.JSON(http.StatusOK, model)\n"
        + "    go func() { _ = tx.Commit(); fmt.Println(os.Getenv(\"FEATURE_FLAG\")) }()\n"
        + "}\n\n"
        + f"func (s *UserService) {name}Service(c *gin.Context, dto UserModel, model UserModel) (gin.H, int, *gorm.DB) {{\n"
        + "    if dto.Status == \"\" { dto.Status = \"active\" }\n"
        + "    if dto.Status == \"banned\" { return gin.H{\"error\": \"bad status\"}, http.StatusBadRequest, s.db }\n"
        + "    return gin.H{\"user\": model, \"error\": nil}, http.StatusOK, s.db\n}\n\n"
        + "func (r *UserRepository) Find(c *gin.Context, request UserModel) (*gorm.DB, error) {\n"
        + "    return r.db.Where(\"status = ?\", request.Status), errors.New(\"repository mapped http 500\")\n}\n"
    )


def architecture_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "context", "database/sql", "net/http")
        + "type User struct {\n    ID string\n    Status string\n}\n\n"
        + "type UserRequest struct {\n    Status *string `json:\"status\"`\n}\n\n"
        + "type UserResponse struct {\n    ID string `json:\"id\"`\n    Status string `json:\"status\"`\n}\n\n"
        + "type UserRepository interface {\n    FindByID(ctx context.Context, id string) (User, error)\n}\n\n"
        + "type UserService struct { repo UserRepository }\n\n"
        + f"func (s UserService) {name}(ctx context.Context, id string, req UserRequest) (UserResponse, error) {{\n"
        + "    user, err := s.repo.FindByID(ctx, id)\n    if err != nil {\n        return UserResponse{}, err\n    }\n"
        + "    if req.Status != nil {\n        user.Status = *req.Status\n    }\n"
        + "    return UserResponse{ID: user.ID, Status: user.Status}, nil\n}\n\n"
        + "func writeUser(w http.ResponseWriter, response UserResponse) {\n    w.Header().Set(\"Content-Type\", \"application/json\")\n    w.WriteHeader(http.StatusOK)\n}\n\n"
        + "func queryUser(ctx context.Context, db *sql.DB, id string) (*sql.Rows, error) {\n    return db.QueryContext(ctx, \"SELECT id, status FROM users WHERE id = ?\", id)\n}\n"
    )


def data_access_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "context", "database/sql", "fmt", "net/http", "github.com/gin-gonic/gin", "gorm.io/gorm")
        + "type Account struct { ID int; TenantID string; Status string }\n\n"
        + f"func {name}(c *gin.Context, db *gorm.DB, sqlDB *sql.DB, ids []int) ([]Account, error) {{\n"
        + "    ctx := context.Background()\n    var accounts []Account\n    var total int64\n"
        + "    sql.Open(\"postgres\", c.Query(\"dsn\"))\n    sqlDB.PingContext(ctx)\n"
        + "    db.AutoMigrate(&Account{})\n    db.Model(&Account{}).Where(\"tenant_id = ?\", c.Query(\"tenant\")).Count(&total)\n"
        + "    db.Where(\"tenant_id = ?\", c.Query(\"tenant\")).Preload(\"Orders\").Find(&accounts)\n"
        + "    for _, id := range ids {\n        db.Exec(\"UPDATE accounts SET status = ? WHERE id = ?\", \"active\", id)\n        db.Create(&Account{ID: id})\n        db.Delete(&Account{}, id)\n        sqlDB.QueryContext(ctx, fmt.Sprintf(\"SELECT * FROM accounts WHERE id = %d\", id))\n    }\n"
        + "    page := accounts[:10]\n    return page, nil\n}\n"
    )


def data_access_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "context", "database/sql")
        + "type Account struct { ID int; TenantID string; Status string }\n\n"
        + f"func {name}(ctx context.Context, db *sql.DB, tenantID string, limit int) (*sql.Rows, error) {{\n"
        + "    if limit <= 0 || limit > 100 {\n        limit = 100\n    }\n"
        + "    return db.QueryContext(ctx, \"SELECT id, tenant_id, status FROM accounts WHERE tenant_id = ? ORDER BY id LIMIT ?\", tenantID, limit)\n}\n"
    )


def performance_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "bytes", "crypto/hmac", "crypto/sha256", "encoding/json", "fmt", "io", "math/rand", "reflect", "regexp", "runtime", "sort", "strings", "sync", "time")
        + "type record struct { ID int; Name string; Payload []byte }\n\n"
        + f"func {name}(items []record, needle string, w io.Writer) string {{\n"
        + "    var out string\n    var mu sync.Mutex\n    seen := map[string]bool{}\n"
        + "    for i, item := range items {\n"
        + "        out += fmt.Sprintf(\"%s/%d\", item.Name, i)\n"
        + "        if strings.Index(item.Name, needle) != -1 { seen[item.Name] = true }\n"
        + "        _ = bytes.Compare(item.Payload, []byte(needle)) == 0\n"
        + "        _ = len(bytes.NewBuffer(item.Payload).String())\n"
        + "        _ = regexp.MustCompile(needle).MatchString(item.Name)\n"
        + "        _ = reflect.DeepEqual(item, record{})\n"
        + "        json.Marshal(item)\n"
        + "        hmac.New(sha256.New, []byte(needle))\n"
        + "        rand.New(rand.NewSource(time.Now().UnixNano()))\n"
        + "        runtime.GOMAXPROCS(runtime.NumCPU())\n"
        + "        mu.Lock(); defer mu.Unlock()\n"
        + "        sort.Slice(items, func(i, j int) bool { return items[i].Name < items[j].Name })\n"
        + "        w.Write([]byte(\"x\"))\n"
        + "    }\n"
        + "    return out\n}\n"
    )


def performance_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "bytes", "io", "sort", "strconv", "strings")
        + "type record struct { ID int; Name string; Payload []byte }\n\n"
        + f"func {name}(items []record, needle string, w io.Writer) string {{\n"
        + "    sort.Slice(items, func(i, j int) bool { return items[i].Name < items[j].Name })\n"
        + "    var b strings.Builder\n    b.Grow(len(items) * 8)\n    scratch := bytes.NewBuffer(make([]byte, 0, 64))\n"
        + "    for i, item := range items {\n"
        + "        if strings.Contains(item.Name, needle) {\n            b.WriteString(item.Name)\n            b.WriteByte('/')\n            b.WriteString(strconv.Itoa(i))\n        }\n"
        + "        scratch.Reset()\n        scratch.Write(item.Payload)\n    }\n"
        + "    _, _ = io.WriteString(w, b.String())\n    return b.String()\n}\n"
    )


def security_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "crypto/md5", "crypto/tls", "database/sql", "fmt", "html/template", "math/rand", "net/http", "os/exec", "path/filepath", "github.com/gin-gonic/gin")
        + f"func {name}(c *gin.Context, db *sql.DB, userInput string) {{\n"
        + "    _ = tls.Config{InsecureSkipVerify: true, MinVersion: tls.VersionTLS10}\n"
        + "    _ = md5.Sum([]byte(userInput))\n"
        + "    _ = rand.Intn(999999)\n"
        + "    db.Query(\"SELECT * FROM users WHERE name = '\" + userInput + \"'\")\n"
        + "    exec.Command(\"sh\", \"-c\", userInput).Run()\n"
        + "    http.SetCookie(c.Writer, &http.Cookie{Name: \"session\", Value: userInput})\n"
        + "    c.Header(\"Access-Control-Allow-Origin\", \"*\")\n"
        + "    c.Header(\"Access-Control-Allow-Credentials\", \"true\")\n"
        + "    c.JSON(http.StatusInternalServerError, gin.H{\"error\": fmt.Errorf(\"secret %s\", userInput).Error()})\n"
        + "    template.HTML(userInput)\n"
        + "    filepath.Join(\"/srv/uploads\", userInput)\n}\n"
    )


def security_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "context", "crypto/rand", "crypto/sha256", "crypto/tls", "database/sql", "net/http", "path/filepath")
        + f"func {name}(ctx context.Context, db *sql.DB, w http.ResponseWriter, userInput string) error {{\n"
        + "    _ = tls.Config{MinVersion: tls.VersionTLS13}\n    token := make([]byte, 32)\n"
        + "    if _, err := rand.Read(token); err != nil {\n        return err\n    }\n"
        + "    _ = sha256.Sum256([]byte(userInput))\n"
        + "    if _, err := db.QueryContext(ctx, \"SELECT id FROM users WHERE name = ?\", userInput); err != nil {\n        return err\n    }\n"
        + "    clean, err := filepath.Rel(\"/srv/uploads\", filepath.Join(\"/srv/uploads\", filepath.Clean(userInput)))\n    if err != nil || clean == \"..\" {\n        return err\n    }\n"
        + "    http.SetCookie(w, &http.Cookie{Name: \"session\", Value: \"opaque\", Secure: true, HttpOnly: true, SameSite: http.SameSiteStrictMode})\n"
        + "    return nil\n}\n"
    )


def context_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "context", "net/http", "os/exec", "time")
        + "type Worker struct { ctx context.Context }\n\n"
        + f"func {name}(ctx context.Context, url string) {{\n"
        + "    child, _ := context.WithTimeout(ctx, time.Second)\n"
        + "    _ = child\n    http.Get(url)\n    exec.Command(\"curl\", url).Run()\n"
        + "    go func() { http.Get(url) }()\n"
        + "    for {\n        select { default: time.Sleep(time.Millisecond) }\n    }\n}\n"
    )


def context_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "context", "net/http", "time")
        + f"func {name}(ctx context.Context, client *http.Client, url string) (*http.Response, error) {{\n"
        + "    child, cancel := context.WithTimeout(ctx, time.Second)\n    defer cancel()\n"
        + "    req, err := http.NewRequestWithContext(child, http.MethodGet, url, nil)\n    if err != nil { return nil, err }\n"
        + "    return client.Do(req)\n}\n"
    )


def concurrency_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "sync", "time")
        + f"func {name}(items []int) map[int]int {{\n"
        + "    out := map[int]int{}\n    var mu sync.Mutex\n    var wg sync.WaitGroup\n"
        + "    for _, item := range items {\n        mu.Lock()\n        time.Sleep(time.Millisecond)\n        out[item] = item\n        mu.Unlock()\n        wg.Add(1)\n        go func() { defer wg.Done(); out[item] = item * 2 }()\n    }\n"
        + "    wg.Wait()\n    return out\n}\n"
    )


def concurrency_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "context", "sync")
        + f"func {name}(ctx context.Context, items []int) map[int]int {{\n"
        + "    out := make(map[int]int, len(items))\n    var mu sync.Mutex\n    var wg sync.WaitGroup\n"
        + "    for _, item := range items {\n        item := item\n        wg.Add(1)\n        go func() {\n            defer wg.Done()\n            select {\n            case <-ctx.Done():\n                return\n            default:\n                mu.Lock(); out[item] = item * 2; mu.Unlock()\n            }\n        }()\n    }\n"
        + "    wg.Wait()\n    return out\n}\n"
    )


def errors_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "fmt", "log", "os")
        + f"func {name}() error {{\n"
        + "    _, _ = os.Open(\"missing.txt\")\n"
        + "    if _, err := os.ReadFile(\"config.json\"); err != nil {\n        log.Fatal(err)\n    }\n"
        + "    err := fmt.Errorf(\"disk failed\")\n    return fmt.Errorf(\"load failed: %v\", err)\n}\n"
    )


def errors_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "fmt", "os")
        + f"func {name}() error {{\n"
        + "    file, err := os.Open(\"config.json\")\n    if err != nil {\n        return fmt.Errorf(\"open config: %w\", err)\n    }\n"
        + "    defer file.Close()\n    return nil\n}\n"
    )


def idioms_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "database/sql", "io", "net/http", "os", "time")
        + "var mutableConfig = map[string]string{}\n\n"
        + f"func {name}(w http.ResponseWriter, r *http.Request, db *sql.DB, paths []string) {{\n"
        + "    client := http.Client{}\n    resp, _ := client.Get(\"https://example.test\")\n    _ = resp\n"
        + "    rows, _ := db.Query(\"SELECT id FROM users\")\n    for rows.Next() {}\n"
        + "    for _, path := range paths {\n        f, _ := os.Open(path)\n        defer f.Close()\n        time.After(time.Second)\n    }\n"
        + "    io.ReadAll(r.Body)\n    w.Write([]byte(\"ok\"))\n    w.WriteHeader(http.StatusAccepted)\n    mutableConfig[\"last\"] = r.URL.Path\n}\n"
    )


def idioms_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "context", "database/sql", "io", "net/http", "time")
        + "var sharedClient = &http.Client{Timeout: 2 * time.Second}\n\n"
        + f"func {name}(ctx context.Context, w http.ResponseWriter, r *http.Request, db *sql.DB) error {{\n"
        + "    limited := http.MaxBytesReader(w, r.Body, 1<<20)\n    defer limited.Close()\n"
        + "    if _, err := io.ReadAll(limited); err != nil { return err }\n"
        + "    rows, err := db.QueryContext(ctx, \"SELECT id FROM users LIMIT 100\")\n    if err != nil { return err }\n    defer rows.Close()\n"
        + "    for rows.Next() {}\n    if err := rows.Err(); err != nil { return err }\n"
        + "    w.WriteHeader(http.StatusAccepted)\n    _, _ = w.Write([]byte(\"ok\"))\n    _ = sharedClient\n    return nil\n}\n"
    )


def gin_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    return (
        header(rule, "positive", "bytes", "compress/gzip", "encoding/json", "io", "net/http", "net/http/httputil", "os", "github.com/gin-gonic/gin")
        + f"func {name}(c *gin.Context) {{\n"
        + "    raw, _ := c.GetRawData()\n    var body map[string]any\n    c.ShouldBindJSON(&body)\n"
        + "    dump, _ := httputil.DumpRequest(c.Request, true)\n    c.Writer.Write(dump)\n"
        + "    client := &http.Client{}\n    client.Get(c.Query(\"upstream\"))\n    client.Get(c.Query(\"upstream\"))\n"
        + "    os.Getenv(\"FEATURE_FLAG\")\n    file, _ := c.FormFile(\"upload\")\n    opened, _ := file.Open()\n    io.ReadAll(opened)\n"
        + "    gz := gzip.NewWriter(c.Writer)\n    for _, chunk := range bytes.Split(raw, []byte(\",\")) { gz.Write(chunk) }\n"
        + "    payload, _ := json.Marshal(gin.H{\"payload\": body, \"raw\": string(raw)})\n    c.Data(200, \"application/json\", payload)\n}\n"
    )


def gin_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return (
        header(rule, "negative", "net/http", "github.com/gin-gonic/gin")
        + "type requestBody struct { ID string `json:\"id\"` }\n\n"
        + f"func {name}(c *gin.Context, client *http.Client) {{\n"
        + "    var body requestBody\n    if err := c.ShouldBindJSON(&body); err != nil {\n        c.JSON(http.StatusBadRequest, gin.H{\"error\": \"invalid request\"})\n        return\n    }\n"
        + "    c.JSON(http.StatusOK, gin.H{\"id\": body.ID})\n    _ = client\n}\n"
    )


def hot_path_positive(rule: dict) -> str:
    return performance_positive(rule)


def hot_path_negative(rule: dict) -> str:
    return performance_negative(rule)


def mod_positive(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Positive")
    rid = str(rule["id"])
    pkg = "json"
    if rid.startswith("xml"):
        pkg = "xml"
    elif rid.startswith("yaml"):
        pkg = "yaml"
    elif rid.startswith("proto"):
        pkg = "proto"
    import_path = {"json": "encoding/json", "xml": "encoding/xml", "yaml": "gopkg.in/yaml.v3", "proto": "google.golang.org/protobuf/proto"}[pkg]
    call = {"json": "json.Unmarshal(payload, &first)\n    json.Unmarshal(payload, &second)", "xml": "xml.Unmarshal(payload, &first)\n    xml.Unmarshal(payload, &second)", "yaml": "yaml.Unmarshal(payload, &first)\n    yaml.Unmarshal(payload, &second)", "proto": "proto.Unmarshal(payload, &first)\n    proto.Unmarshal(payload, &second)"}[pkg]
    return header(rule, "positive", import_path) + "type envelope struct { ID string }\n\n" + f"func {name}(payload []byte) {{\n    var first envelope\n    var second envelope\n    {call}\n}}\n"


def mod_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return header(rule, "negative", "encoding/json") + "type envelope struct { ID string }\n\n" + f"func {name}(payload []byte) (envelope, error) {{\n    var first envelope\n    err := json.Unmarshal(payload, &first)\n    return first, err\n}}\n"


def consistency_positive(rule: dict) -> str:
    rid = str(rule["id"])
    name = exported_name(rid, "Positive")
    if rid == "mixed_receiver_kinds":
        return header(rule, "positive") + f"type {name}User struct {{ ID string }}\n\nfunc (u {name}User) IDValue() string {{ return u.ID }}\nfunc (u *{name}User) Rename(id string) {{ u.ID = id }}\n"
    return header(rule, "positive") + f"type {name}User struct {{\n    ID string `json:\"id\" json:\"user_id\"`\n}}\n"


def consistency_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return header(rule, "negative") + f"type {name}User struct {{\n    ID string `json:\"id\" db:\"id\"`\n}}\n\nfunc (u *{name}User) IDValue() string {{ return u.ID }}\nfunc (u *{name}User) Rename(id string) {{ u.ID = id }}\n"


def style_positive(rule: dict) -> str:
    rid = str(rule["id"])
    name = exported_name(rid, "Positive")
    if rid == "misgrouped_imports":
        return "package alpha\n\nimport (\n    \"github.com/acme/pkg\"\n    \"fmt\"\n)\n\n" + comment(rule, "positive") + f"func {name}() {{ fmt.Println(pkg.Name) }}\n"
    return "package alpha\n\n" + comment(rule, "positive") + f"func {name}() string {{ return \"mixed package names in same directory\" }}\n"


def style_negative(rule: dict) -> str:
    name = exported_name(str(rule["id"]), "Negative")
    return "package alpha\n\nimport (\n    \"fmt\"\n\n    \"github.com/acme/pkg\"\n)\n\n" + comment(rule, "negative") + f"func {name}() {{ fmt.Println(pkg.Name) }}\n"


POSITIVE_BY_FAMILY = {
    "architecture": architecture_positive,
    "concurrency": concurrency_positive,
    "consistency": consistency_positive,
    "context": context_positive,
    "data_access": data_access_positive,
    "errors": errors_positive,
    "gin": gin_positive,
    "hot_path": hot_path_positive,
    "idioms": idioms_positive,
    "library": gin_positive,
    "mod": mod_positive,
    "performance": performance_positive,
    "security": security_positive,
    "style": style_positive,
}

NEGATIVE_BY_FAMILY = {
    "architecture": architecture_negative,
    "concurrency": concurrency_negative,
    "consistency": consistency_negative,
    "context": context_negative,
    "data_access": data_access_negative,
    "errors": errors_negative,
    "gin": gin_negative,
    "hot_path": hot_path_negative,
    "idioms": idioms_negative,
    "library": gin_negative,
    "mod": mod_negative,
    "performance": performance_negative,
    "security": security_negative,
    "style": style_negative,
}


def fixture_text(rule: dict, polarity: str) -> str:
    family = str(rule["family"])
    builder = (POSITIVE_BY_FAMILY if polarity == "positive" else NEGATIVE_BY_FAMILY).get(family)
    if builder is None:
        return generic_negative(rule)
    return builder(rule)


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
