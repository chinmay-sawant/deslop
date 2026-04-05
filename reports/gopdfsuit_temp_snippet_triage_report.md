# Temp.txt Snippet-Only Triage Report

- Source: `scripts/temp.txt`
- Scope: snippet-only triage; no other project files were used.
- Total findings covered: 1445
- Legend:
  - Likely real from snippet = the visible line itself supports the rule.
  - Likely false positive/dismissible = the visible line does not support the rule or obviously points at the wrong thing.
  - Needs more context = the rule may be valid, but the visible snippet alone is not enough to decide.

## Summary
- Likely Real From Snippet: 346 findings
- Likely False Positive Or Dismissible From Snippet: 134 findings
- Needs More Context Than The Snippet Shows: 965 findings

## Likely Real From Snippet

- Rule: Broad except Exception: style handlers that still obscure failure shape even when not fully swallowed.
  Findings: 1337, 1342, 1348, 1357
  Why: The broad `except Exception` handler is directly visible.

- Rule: Calls that load an entire payload into memory instead of streaming.
  Findings: 107, 116, 122, 134, 143, 179, 184, 189, 194, 201, 204, 609, 762-765, 1088-1090, 1232, 1238-1239, 1248, 1285, 1361, 1373-1374, 1376-1377, 1379-1381, 1383, 1385, 1387-1388, 1390-1391, 1393-1394, 1396-1397, 1399-1400
  Why: The `os.ReadFile` / `io.ReadAll` call is directly visible.

- Rule: Direct recursion in traversal-style helpers that may be safer as iterative walks for deep inputs.
  Findings: 14-16, 1351-1352
  Why: The recursive self-call is visible.

- Rule: Direct use of weak standard-library crypto packages such as crypto/md5, crypto/sha1, crypto/des, and crypto/rc4.
  Findings: 531, 533, 537, 539, 541, 545, 547, 806, 808, 1027, 1029, 1031, 1033, 1035, 1038, 1040
  Why: The weak crypto primitive is directly visible.

- Rule: Explicit return None in simple code paths where Python would already return None implicitly.
  Findings: 13, 83, 1344-1346, 1349
  Why: The explicit `return None` is directly visible.

- Rule: Exported functions or methods that expose raw boolean mode switches in their signatures.
  Findings: 571, 849, 1013
  Why: The boolean mode switch is visible in the signature.

- Rule: Function names that are overly generic without stronger contextual signals.
  Findings: 1184
  Why: The generic function name is visible.

- Rule: Gin handlers that open uploaded form files and then materialize them with io.ReadAll(...).
  Findings: 121, 142, 178, 183, 188, 193, 200
  Why: The `io.ReadAll(...)` on an uploaded file handle is visible.

- Rule: Handlers that load files into memory and then write them through gin.Context.Data(...) instead of using file helpers or streaming.
  Findings: 118
  Why: The `c.Data(...)` write of in-memory bytes is visible.

- Rule: Module-scope file reads, writes, or directory scans that happen during import.
  Findings: 1240
  Why: The module-scope `with open(...)` is visible.

- Rule: Public Python APIs that expose Any in parameter or return contracts.
  Findings: 31, 34, 38, 41, 44, 47, 51, 54, 58, 66
  Why: The public `Any`-based contract is visible.

- Rule: Public Python functions that expose *args or **kwargs instead of a clearer interface.
  Findings: 3, 5
  Why: The `*args` exposure is visible in the signature.

- Rule: Public functions or model fields that expose Any, object, or similarly wide contracts.
  Findings: 30, 33, 37, 40, 43, 46, 50, 53, 57, 65
  Why: The public `Any`-based contract is visible.

- Rule: Raw go statements without an obvious context or WaitGroup-like coordination signal.
  Findings: 101, 103
  Why: A raw `go` statement is visible with no coordination signal in the snippet.

- Rule: Request-path handlers that read files directly instead of using startup caching or dedicated file-serving paths.
  Findings: 115
  Why: The handler-side `os.ReadFile(...)` call is visible.

- Rule: Signatures that rely on any or empty interface types.
  Findings: 12, 18, 20, 22-24, 26-28, 32, 35, 39, 42, 45, 48, 52, 55, 59-60, 62, 64, 67, 205, 1161, 1363-1364
  Why: The wide `Any`/`Dict[str, Any]` contract is visible in the signature.

- Rule: String joins that materialize an unnecessary list comprehension instead of using a generator or direct iterable.
  Findings: 1265-1266
  Why: The list-comprehension join pattern is directly visible.

- Rule: Very long identifiers with too many descriptive tokens.
  Findings: 517, 553, 555, 559, 570, 572, 691, 821, 832, 834, 1095, 1136, 1237, 1333, 1371, 1401
  Why: The long identifier is visible.

- Rule: `binary.Read(r, order, &singleField)` for reading a single integer
  Findings: 610-611, 614, 616, 618, 620-621, 623, 625, 627, 629, 631, 633, 635, 637-638, 641-642, 644, 646-649, 651-658, 660, 662-663, 665-666, 668, 670, 672, 674, 676, 678-680, 682, 684, 686-687
  Why: The single-field `binary.Read` call is visible.

- Rule: `builder.WriteString(a + b)` where `a` and `b` are separate bindings
  Findings: 820, 981, 1442-1445
  Why: The concatenation inside `WriteString` is visible.

- Rule: `c.JSON(500, gin.H{"error": err.Error()})` or `http.Error(w, err.Error(), 500)` returning internal error details to the client
  Findings: 117, 120, 123-125, 127-130, 132, 135, 139-140, 144-146, 149, 151, 154, 157, 161, 164, 167, 171, 180-181, 185-186, 190-191, 195-196, 202-203
  Why: The response string concatenates `err.Error()` directly.

- Rule: `dst = append(dst, src...)` when `dst` is known empty and `len(src)` is known
  Findings: 845-846, 1106, 1133
  Why: The `append([]T(nil), src...)` pattern is visible.

- Rule: `fmt.Errorf("context: %v", err)` instead of `%w`
  Findings: 842, 844, 1016, 1018, 1053, 1056
  Why: The `%v` formatting of `err` is directly visible.

- Rule: `fmt.Sprintf("%d", n)` where `n` is clearly an integer type
  Findings: 771, 825, 829, 1252, 1296, 1300, 1302
  Why: The `%d` formatting call is visible.

- Rule: `fmt.Sprintf("%s:%s", a, b)` where only `%s` verbs are used
  Findings: 518-530, 549, 775, 802, 804, 810, 848, 1014, 1019, 1080-1081, 1155-1158
  Why: The formatting pattern is visible.

- Rule: `if strings.HasPrefix(s, p) { s = strings.TrimPrefix(s, p) }`
  Findings: 850, 1075
  Why: The prefix check plus trim pattern is visible.

- Rule: `if strings.HasSuffix(s, p) { s = strings.TrimSuffix(s, p) }`
  Findings: 1074, 1076, 1183
  Why: The suffix check plus trim pattern is visible.

- Rule: `md5.New()`, `sha1.New()`, `md5.Sum()`, `sha1.Sum()` used for integrity checks, checksums, or MAC operations (not just `weak_crypto` import-level detection)
  Findings: 532, 534, 538, 540, 542, 546, 548, 807, 809, 1028, 1030, 1032, 1034, 1036, 1039
  Why: The `md5`/`sha1` call is directly visible.

- Rule: `os.Create("/tmp/myapp-data.txt")` or `os.OpenFile("/tmp/" + fixedName, ...)` with predictable filenames
  Findings: 100
  Why: The fixed `/tmp/...` path is shown explicitly.

- Rule: `strings.ReplaceAll(s, "x", "y")` where both old and new are single characters
  Findings: 242, 371, 690, 777, 1023, 1025, 1165, 1175, 1182, 1185, 1187
  Why: The single-character `ReplaceAll` call is visible.

- Rule: `v := i.(T)` without the comma-ok form in non-panic-safe code
  Findings: 550-551, 767-768, 840
  Why: The unchecked type assertion is directly visible.

- Rule: chains like `strings.TrimSpace(strings.ToLower(strings.TrimPrefix(s, ...)))` that scan the string multiple times
  Findings: 1045-1046, 1097, 1105
  Why: The multi-scan normalization chain is visible.

- Rule: exported struct fields containing sensitive data (Password, Secret, Token, APIKey, PrivateKey) without `json:"-"` tags in API response structs
  Findings: 207-210
  Why: The sensitive field and JSON tag are directly visible.

- Rule: fmt.Errorf calls that reference err without %w.
  Findings: 841, 843, 1015, 1017, 1052, 1055
  Why: The `%v` formatting of `err` is directly visible.

- Rule: print() calls left in non-test Python functions that do not look like obvious main-entrypoint output.
  Findings: 1273-1274, 1327-1328, 1369-1370
  Why: The `print(...)` call is directly visible.

## Likely False Positive Or Dismissible From Snippet

- Rule: Functions that mix HTTP, persistence, and filesystem-style concerns in one body.
  Findings: 1326, 1355
  Why: The visible snippet is only a function signature; it does not show mixed concerns.

- Rule: Handlers that manually marshal JSON and then write it through gin.Context.Data(...).
  Findings: 197
  Why: The visible snippet writes PDF bytes, not manually marshaled JSON.

- Rule: Large gin.H payloads built as transient dynamic maps right before JSON rendering.
  Findings: 113, 119, 126, 131, 141, 155, 165, 177, 182, 187, 192, 199
  Why: The visible payloads are tiny one-field error maps, not large transient payloads.

- Rule: Large list or export handlers that materialize everything before writing rather than using chunked or streaming output.
  Findings: 198
  Why: The visible snippet is a single PDF response, not a large list/export payload.

- Rule: Package-level variables that are mutated from function bodies instead of kept immutable.
  Findings: 566-567, 1201, 1205, 1292-1293
  Why: The visible snippet is only a package-level declaration, not a mutation site.

- Rule: Query result handles that appear locally owned but have no observed rows.Close() call.
  Findings: 112
  Why: The snippet shows `c.Query(...)`, not a query result handle.

- Rule: Read-style, transformation-style, or utility-style names that still perform mutation or own multiple infrastructure concerns.
  Findings: 1339
  Why: The visible snippet is just `import json`, which does not support the rule at all.

- Rule: Repeated branch-shaping numeric or string literals that likely want an explicit constant or policy name.
  Findings: 1317, 1332, 1354
  Why: The visible snippets are function signatures, not repeated branch literals.

- Rule: Tests that look skipped, TODO-shaped, or otherwise placeholder-like.
  Findings: 573-574
  Why: The visible test names are concrete tests, not TODO/placeholder names.

- Rule: UUID or hash formatting observed inside loops only for log output.
  Findings: 994, 1009
  Why: The visible formatting is used to build output content, not just log output.

- Rule: `filepath.Join(baseDir, userInput)` without subsequent `filepath.Rel` or path-containment validation
  Findings: 111, 114
  Why: The visible snippets do not show an unsafe join of raw user input; one uses fixed path segments and the other already normalizes with `filepath.Base(...)`.

- Rule: `if _, ok := m[k]; ok { v := m[k] }` -- two map lookups for the same key
  Findings: 174, 176, 712, 1103
  Why: The snippet shows only the first lookup, not the claimed second lookup.

- Rule: `log.Error(err); return err` or `logger.Error("failed", zap.Error(err)); return fmt.Errorf("failed: %w", err)` -- logging the error then returning it
  Findings: 569, 612-613, 615, 617, 622, 624, 626, 630, 632, 639, 643, 645, 650, 659, 661, 664, 667, 669, 671, 673, 675, 681, 683, 747
  Why: The visible line shows only a returned error, not a prior log-and-return pair.

- Rule: `os.OpenFile(path, flag, 0666)` or `os.WriteFile(path, data, 0777)` with world-readable/writable permissions
  Findings: 1284, 1287-1289, 1315, 1362
  Why: The visible permission is `0644`, not the world-writable values named by the rule.

- Rule: `sub := original[a:b]` followed by `sub = append(sub, ...)` with no capacity bound
  Findings: 217-223, 232, 251, 260, 279, 284, 309, 314, 360, 365, 380, 389, 408, 413, 428, 452, 456, 458, 461, 466, 469, 512-516, 737, 778, 789, 792, 797, 799, 801, 803, 805, 811-812, 819, 847, 884, 887, 890-891, 899, 918, 1037, 1130, 1163-1164
  Why: The visible snippets show reset/reuse lines like `buf = buf[:0]`, not the claimed `sub := original[a:b]` pattern.

- Rule: `token == expectedToken` or `bytes.Equal(token, expected)` for comparing authentication tokens, API keys, or HMAC values
  Findings: 1026
  Why: The visible snippet does not show any token comparison.

- Rule: json.dumps(...) is repeated for the same object instead of caching the serialized value.
  Findings: 10-11, 84, 1334
  Why: The snippet shows only a single `json.dumps(...)` call, not the claimed repetition.

- Rule: login/authentication handler functions (name contains `Login`, `Authenticate`, `SignIn`) that don't reference rate limiting, throttling, or brute-force protection mechanisms
  Findings: 206
  Why: The flagged function name is `LogAuthInfo`, which is not a login/authentication entrypoint.

- Rule: maps.Clone or equivalent map-copy calls observed inside loops.
  Findings: 942, 1170
  Why: The visible snippets do not show a map-clone operation.

- Rule: slices.Clone(...) or similar whole-slice cloning observed inside loops.
  Findings: 721, 726, 731
  Why: The visible snippets do not show `slices.Clone(...)` or another obvious whole-slice clone.

## Needs More Context Than The Snippet Shows

- Rule: An invariant string template is formatted repeatedly in a loop instead of being partially precomputed.
  Findings: 1256, 1258, 1262, 1269, 1316, 1319, 1324-1325, 1335, 1338
  Why: The snippet does not show the repeated invariant formatting; only loop context is visible.

- Rule: Append calls inside nested loops without visible preallocation on the outer slice.
  Findings: 860, 881, 892, 977, 1437
  Why: The append is visible, but the nested-loop/preallocation context is not.

- Rule: Broad exception handlers like except: or except Exception: that immediately suppress the error with pass, continue, break, or return.
  Findings: 1244
  Why: The `except Exception:` line is visible, but the suppression action is not.

- Rule: Byte-to-string or string-to-byte conversion observed inside loops in short-lived lookup or append paths.
  Findings: 720, 725, 730, 740, 744, 749, 1063, 1069, 1126, 1218
  Why: The conversions are visible, but the repeated loop path is not.

- Rule: Ceremonial wrapper-style or tiny data-container classes that add little beyond storing constructor state.
  Findings: 1331
  Why: Only the class header is shown.

- Rule: Dataclass or TypedDict models that accumulate many optional fields and boolean switches.
  Findings: 17, 19, 21, 25, 29, 36, 49, 56, 61, 63
  Why: Only the class header is shown; the field list is missing.

- Rule: External JSON or tabular data is parsed without visible schema validation.
  Findings: 1257
  Why: The file-open line is visible, but the parse/validation logic is not.

- Rule: Gin handlers that call c.JSON(...) or c.PureJSON(...) from inside loops.
  Findings: 133, 136, 150, 152
  Why: The `c.JSON(...)` call is visible, but the loop and surrounding control flow are not.

- Rule: Loop shapes that look like obvious sum, any, or all candidates.
  Findings: 1261, 1268, 1318, 1323
  Why: The snippet shows only a loop header, not the full accumulation logic.

- Rule: Loop-local list, dict, or set construction that likely adds avoidable allocation churn.
  Findings: 1321
  Why: The allocation is visible, but the surrounding loop is not.

- Rule: Looping goroutine literals without an obvious ctx.Done() or done-channel shutdown path.
  Findings: 1278, 1306
  Why: The goroutine start is visible, but any shutdown path is not.

- Rule: Map insertions inside loops without a visible size hint on the initial make call.
  Findings: 586, 591, 595, 600, 688-689, 705-706, 711, 716, 753, 770, 859, 864, 878, 903, 905, 915, 921, 943, 973, 1061, 1068, 1083, 1113, 1147, 1167, 1174
  Why: The insertion is visible, but the map construction and size hint are not.

- Rule: Obvious make, new, or buffer-construction calls inside loops.
  Findings: 535, 543, 575, 578, 580, 589, 596, 940, 944, 1093, 1121, 1168, 1413
  Why: The allocation is visible, but the enclosing loop is not.

- Rule: Public Python functions that omit complete parameter or return annotations.
  Findings: 1-2, 4, 68-71, 1241-1243, 1255, 1259-1260, 1267, 1271-1272, 1275, 1290-1291, 1329-1330, 1336, 1341, 1347, 1350, 1356, 1365-1368
  Why: The missing annotation is visible, but many flagged cases are fixtures, hooks, or `main()` helpers, so snippet-only triage cannot cleanly separate actionable API issues from acceptable script/test code.

- Rule: Repeated Lock or RLock acquisition inside loops.
  Findings: 105, 1253
  Why: The lock call is visible, but the enclosing loop shape is not.

- Rule: Repeated exception-handling block shapes in one file.
  Findings: 1322, 1343
  Why: The snippet shows only a single `except` block, not repetition across the file.

- Rule: Repeated string concatenation inside loops (O(n^2) risk).
  Findings: 109, 445, 558, 957, 959, 1066, 1070, 1129, 1186, 1202-1204, 1215, 1263-1264, 1270
  Why: The string concatenation is visible, but the repeated loop context is not.

- Rule: Repeated validation guard pipelines across functions in one file.
  Findings: 7, 9
  Why: The repeated guard pipeline is not shown in the snippet.

- Rule: Repository-local interfaces with one obvious implementation and a very small consumer surface.
  Findings: 1044
  Why: The interface declaration is visible, but the implementation/consumer count is not.

- Rule: Request, sync, or job-style Python functions that call HTTP boundaries with no obvious timeout or retry policy.
  Findings: 1340
  Why: Only the function signature is shown; the HTTP call site is not.

- Rule: Same collection traversed multiple times for filter, count, and process steps.
  Findings: 224, 359, 587, 603, 715, 769, 822, 831, 833, 835, 927, 946, 985, 1135, 1139, 1149, 1311, 1425
  Why: The snippet shows only one traversal point, not the repeated passes.

- Rule: Slice append followed by reslice each iteration instead of batching.
  Findings: 225, 361, 722, 779, 922
  Why: The append is visible, but the claimed reslice/batching pattern is not fully shown.

- Rule: Slice append inside a range loop without visible preallocation when the bound is locally known.
  Findings: 137, 226, 228, 230, 233, 235, 237, 239, 243, 245, 247, 249, 252, 254, 256, 258, 261, 263, 265, 267, 269, 271, 273, 275, 277, 280, 282, 285, 287, 289, 291, 293, 295, 297, 299, 301, 303, 305, 307, 310, 312, 315, 317, 319, 321, 323, 325, 327, 329, 331, 333, 335, 337, 339, 341, 343, 345, 347, 349, 351, 353, 355, 357, 362-364, 366-369, 372, 374, 376, 378, 381, 383, 385, 387, 390, 392, 394, 396, 398, 400, 402, 404, 406, 409, 411, 414, 416, 418, 420, 422, 424, 426, 429, 431, 433, 435, 437, 439, 441, 443, 446, 448, 450, 453-455, 457, 459-460, 462-465, 467-468, 470-472, 474, 476, 478, 480, 482, 484, 486, 488, 490, 492, 494, 496, 498, 500, 502, 504, 506, 508, 510, 556, 560, 563, 592, 601, 605, 702, 709, 713, 732, 772, 780-781, 783, 785, 787, 790, 793, 798, 800, 851, 854, 857, 861, 868, 870, 872, 874, 876, 879, 882, 885, 888, 893, 895, 897, 906, 908, 910, 913, 916, 919, 923, 925, 928, 930, 933, 935, 947, 949, 951, 953, 955, 961, 963, 971, 978, 987, 989, 991, 1047, 1064, 1072, 1086, 1098, 1100, 1107, 1109, 1111, 1116, 1119, 1123, 1127, 1141, 1143, 1159, 1207, 1209, 1211, 1213, 1216, 1220, 1222, 1224, 1226, 1228, 1249, 1282, 1313, 1415, 1417, 1419, 1421, 1423, 1426, 1430, 1432, 1434, 1438
  Why: The append is visible, but the known loop bound and allocation history are not.

- Rule: Stable value normalization (ToLower, TrimSpace, etc.) repeated inside inner loops.
  Findings: 108, 173, 175, 241, 370, 776, 1102, 1132
  Why: The normalization call is visible, but the loop context is not.

- Rule: Standard-library context-aware calls from functions that do not accept context.Context.
  Findings: 215, 568, 1051, 1054
  Why: The `exec.Command(...)` call is visible, but the enclosing function signature is not.

- Rule: Tests that assert success expectations without any obvious negative-path signal.
  Findings: 72-82, 85-99
  Why: Only the test names are shown; the assertions/body are missing.

- Rule: Tests that exercise production code without an obvious assertion or failure signal.
  Findings: 1358, 1372, 1375, 1378, 1382, 1384, 1386, 1389, 1392, 1395, 1398, 1402
  Why: Only test names are visible; the assertion body is omitted.

- Rule: The same string binding is converted with strconv parsing helpers multiple times in one function.
  Findings: 750, 853, 865, 969, 976, 1071, 1177-1178, 1188, 1206
  Why: One conversion is visible, not the claimed repeated conversions.

- Rule: Tutorial-style documentation that narrates obvious implementation steps.
  Findings: 6, 8, 1020, 1104, 1235-1236
  Why: The docstring body is not shown.

- Rule: Very large Python functions with high control-flow and call-surface concentration.
  Findings: 1320, 1353
  Why: Only the function signature is shown.

- Rule: `copy := append([]T(nil), original...)` when `copy` is only read, never mutated
  Findings: 1134
  Why: The copy pattern is visible, but later mutation/read behavior is not.

- Rule: `errors.New("some error")` called repeatedly in hot paths instead of a package-level sentinel
  Findings: 619, 628, 634, 636, 640, 677, 685, 1021-1022, 1024, 1049, 1062, 1091-1092, 1096
  Why: The `errors.New(...)` call is visible, but repetition/hot-path context is not.

- Rule: `for i := 0; i < len(s); i++ { c := s[i] }` on strings that should iterate runes
  Findings: 839, 1079
  Why: The index-based loop is visible, but the snippet does not prove the iterated value is a string rather than bytes.

- Rule: `for { wg.Add(1); go func() { ... wg.Done() }() }` where `wg.Add` could be called once before the loop with the count
  Findings: 1230, 1276, 1305
  Why: The `wg.Add(1)` is visible, but the surrounding loop shape is not.

- Rule: `go func() { result <- compute() }()` followed by `<-result` where the goroutine is immediately awaited
  Findings: 102, 1277
  Why: The snippet shows the goroutine start but not the immediate await.

- Rule: `json.Unmarshal` into `map[string]any` for numeric data without `UseNumber()`
  Findings: 1359
  Why: The `json.Unmarshal(...)` call is visible, but the target map type is not.

- Rule: `log.Printf` (stdlib) usage in web service handler code
  Findings: 153, 156, 158-160, 162-163, 166, 168-170, 172
  Why: The logging call is visible, but the snippet alone does not fully show the handler context.

- Rule: `m[k] = append(m[k], v)` in loops without pre-allocating inner slices
  Findings: 1114, 1148
  Why: The map-append expression is visible, but the loop/preallocation context is not.

- Rule: `map[int]bool` or `map[int]struct{}` used as a set for small dense integer ranges
  Findings: 856, 863, 866, 904, 912, 932, 937, 939, 970
  Why: The map-as-set is visible, but the "small dense range" assumption is not.

- Rule: `math/rand.Intn()`, `rand.Float64()`, etc. (global source) in hot handler or goroutine paths
  Findings: 1308
  Why: The random call is visible, but the hot-path/handler context is not.

- Rule: `net/http/pprof` import or `http.Handle("/debug/pprof/", ...)` registration without access control
  Findings: 110
  Why: The pprof route is visible, but any surrounding access control is not.

- Rule: `time.Now()` called on every iteration of a tight inner loop
  Findings: 1280, 1286, 1309-1310
  Why: The `time.Now()` call is visible, but the inner-loop tightness is not.

- Rule: `var result []T` followed by `append` in a loop where the iteration count is visible from a `len()` or range source
  Findings: 138, 216, 227, 229, 231, 234, 236, 238, 240, 244, 246, 248, 250, 253, 255, 257, 259, 262, 264, 266, 268, 270, 272, 274, 276, 278, 281, 283, 286, 288, 290, 292, 294, 296, 298, 300, 302, 304, 306, 308, 311, 313, 316, 318, 320, 322, 324, 326, 328, 330, 332, 334, 336, 338, 340, 342, 344, 346, 348, 350, 352, 354, 356, 358, 373, 375, 377, 379, 382, 384, 386, 388, 391, 393, 395, 397, 399, 401, 403, 405, 407, 410, 412, 415, 417, 419, 421, 423, 425, 427, 430, 432, 434, 436, 438, 440, 442, 444, 447, 449, 451, 473, 475, 477, 479, 481, 483, 485, 487, 489, 491, 493, 495, 497, 499, 501, 503, 505, 507, 509, 511, 557, 561, 564, 577, 593, 602, 606, 608, 703, 710, 714, 733, 773, 782, 784, 786, 788, 852, 855, 858, 862, 869, 871, 873, 875, 877, 880, 883, 886, 889, 894, 896, 898, 900, 902, 907, 909, 911, 914, 917, 920, 924, 926, 929, 931, 934, 936, 948, 950, 952, 954, 956, 962, 964, 972, 979, 988, 990, 992, 1048, 1057, 1065, 1073, 1087, 1099, 1101, 1108, 1110, 1112, 1117, 1120, 1124, 1128, 1142, 1144, 1160, 1208, 1210, 1212, 1214, 1217, 1219, 1221, 1223, 1225, 1227, 1229, 1245, 1250, 1283, 1314, 1403-1412, 1416, 1418, 1420, 1422, 1424, 1427-1429, 1431, 1433, 1435-1436, 1439-1441
  Why: The append is visible, but the earlier zero-value declaration and bound are not.

- Rule: bytes.Buffer used without Grow when approximate output size is locally visible.
  Findings: 147, 588, 594, 598-599, 604, 607, 708, 746, 759, 766, 901, 938, 967, 1067, 1078, 1082, 1166
  Why: The buffer is visible, but the supposedly visible size estimate is not.

- Rule: defer statements inside loops that can accumulate resources until function exit.
  Findings: 1231, 1279, 1307
  Why: The `defer` is visible, but the resource-lifetime issue is not shown clearly in the snippet.

- Rule: err != nil branches that jump straight to panic or log.Fatal style exits.
  Findings: 104, 211-213, 1162, 1233-1234
  Why: The snippet shows only the `if err != nil` branch, not the panic/log.Fatal line.

- Rule: fmt formatting calls such as Sprintf inside loops.
  Findings: 148, 214, 582, 584-585, 699, 704, 717, 723, 734, 736, 738-739, 743, 748, 751, 794-796, 813-818, 823-824, 826-828, 830, 838, 958, 960, 966, 968, 982-983, 993, 995-1003, 1005-1006, 1008, 1010-1012, 1043, 1050, 1060, 1084-1085, 1115, 1118, 1138, 1140, 1145-1146, 1152-1154, 1171-1173, 1176, 1179-1181, 1189-1200, 1246-1247, 1251, 1294-1295, 1297-1299, 1301, 1303-1304, 1360
  Why: The formatting call is visible, but the enclosing loop is not.

- Rule: goroutines appending to a shared slice without synchronization
  Findings: 106, 1254, 1281, 1312
  Why: The append is visible, but the synchronization context is not fully visible.

- Rule: make([]T, ...) scratch slices recreated inside loops instead of being reused.
  Findings: 536, 544, 576, 590, 597, 1094, 1414
  Why: The slice creation is visible, but the loop context is not.

- Rule: make(map[K]V, ...) scratch maps recreated inside loops instead of being reused or prebuilt.
  Findings: 579, 581, 941, 945, 1122, 1169
  Why: The map creation is visible, but the loop context is not.

- Rule: regexp.Compile or regexp.MustCompile observed inside obvious iterative paths.
  Findings: 692-698, 700-701, 718-719, 724, 727-729, 735, 752, 754-757, 836-837, 867, 965, 974-975, 1041-1042, 1058-1059, 1125
  Why: The regexp compilation is visible, but the iterative path is not.

- Rule: strings.Builder used without Grow when approximate output size is locally visible.
  Findings: 552, 554, 562, 565, 583, 742, 761, 774, 980, 984, 1004, 1007, 1077, 1131, 1137, 1151
  Why: The builder is visible, but the local size estimate is not.

- Rule: strings.Builder, bytes.Buffer, or bytes.NewBuffer(...) constructions observed inside loops instead of being reset or reused.
  Findings: 707, 741, 745, 758, 760, 791, 986, 1150
  Why: The construction is visible, but the loop is not.