use std::collections::BTreeSet;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::framework_patterns::{
    BodyLine, body_lines, import_aliases_for, is_request_path_function, split_assignment,
};

pub(crate) const BINDING_LOCATION: &str = file!();

#[derive(Clone, Copy)]
enum Scope {
    Any,
    Loop,
    Request,
    RequestOrLoop,
}

#[derive(Clone, Copy)]
struct AliasRuleSpec {
    id: &'static str,
    import_path: &'static str,
    scope: Scope,
    summary: &'static str,
    required: &'static [&'static str],
    excluded: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct PlainRuleSpec {
    id: &'static str,
    scope: Scope,
    summary: &'static str,
    required: &'static [&'static str],
    excluded: &'static [&'static str],
}

macro_rules! alias_rule {
    ($id:literal, $import:literal, $scope:ident, $summary:literal, [$($required:literal),* $(,)?], [$($excluded:literal),* $(,)?]) => {
        AliasRuleSpec {
            id: $id,
            import_path: $import,
            scope: Scope::$scope,
            summary: $summary,
            required: &[$($required),*],
            excluded: &[$($excluded),*],
        }
    };
}

macro_rules! plain_rule {
    ($id:literal, $scope:ident, $summary:literal, [$($required:literal),* $(,)?], [$($excluded:literal),* $(,)?]) => {
        PlainRuleSpec {
            id: $id,
            scope: Scope::$scope,
            summary: $summary,
            required: &[$($required),*],
            excluded: &[$($excluded),*],
        }
    };
}

const ALIAS_RULES: &[AliasRuleSpec] = &[
    alias_rule!(
        "bytes_compare_equal_zero",
        "bytes",
        Any,
        "`bytes.Compare(...) == 0` instead of `bytes.Equal(...)`.",
        ["{alias}.Compare(", "== 0"],
        []
    ),
    alias_rule!(
        "bytes_compare_not_equal_zero",
        "bytes",
        Any,
        "`bytes.Compare(...) != 0` instead of `!bytes.Equal(...)`.",
        ["{alias}.Compare(", "!= 0"],
        []
    ),
    alias_rule!(
        "bytes_index_not_minus_one_contains",
        "bytes",
        Any,
        "`bytes.Index(...)` presence checks instead of `bytes.Contains(...)`.",
        ["{alias}.Index(", "!= -1"],
        []
    ),
    alias_rule!(
        "bytes_index_any_not_minus_one_contains_any",
        "bytes",
        Any,
        "`bytes.IndexAny(...)` presence checks instead of `bytes.ContainsAny(...)`.",
        ["{alias}.IndexAny(", "!= -1"],
        []
    ),
    alias_rule!(
        "bytes_count_gt_zero_contains",
        "bytes",
        Any,
        "`bytes.Count(...) > 0` instead of `bytes.Contains(...)`.",
        ["{alias}.Count(", "> 0"],
        []
    ),
    alias_rule!(
        "strings_compare_equal_zero",
        "strings",
        Any,
        "`strings.Compare(...) == 0` instead of direct string equality.",
        ["{alias}.Compare(", "== 0"],
        []
    ),
    alias_rule!(
        "strings_compare_not_equal_zero",
        "strings",
        Any,
        "`strings.Compare(...) != 0` instead of direct string inequality.",
        ["{alias}.Compare(", "!= 0"],
        []
    ),
    alias_rule!(
        "strings_index_any_not_minus_one_contains_any",
        "strings",
        Any,
        "`strings.IndexAny(...)` presence checks instead of `strings.ContainsAny(...)`.",
        ["{alias}.IndexAny(", "!= -1"],
        []
    ),
    alias_rule!(
        "strings_count_gt_zero_contains",
        "strings",
        Any,
        "`strings.Count(...) > 0` instead of `strings.Contains(...)`.",
        ["{alias}.Count(", "> 0"],
        []
    ),
    alias_rule!(
        "strings_splitn_two_index_zero_cut",
        "strings",
        Any,
        "`strings.SplitN(..., 2)[0]` instead of `strings.Cut(...)`.",
        ["{alias}.SplitN(", ", 2)", "[0]"],
        []
    ),
    alias_rule!(
        "strings_splitn_two_index_one_cut",
        "strings",
        Any,
        "`strings.SplitN(..., 2)[1]` instead of `strings.Cut(...)`.",
        ["{alias}.SplitN(", ", 2)", "[1]"],
        []
    ),
    alias_rule!(
        "bytes_splitn_two_index_zero_cut",
        "bytes",
        Any,
        "`bytes.SplitN(..., 2)[0]` instead of `bytes.Cut(...)`.",
        ["{alias}.SplitN(", ", 2)", "[0]"],
        []
    ),
    alias_rule!(
        "bytes_splitn_two_index_one_cut",
        "bytes",
        Any,
        "`bytes.SplitN(..., 2)[1]` instead of `bytes.Cut(...)`.",
        ["{alias}.SplitN(", ", 2)", "[1]"],
        []
    ),
    alias_rule!(
        "strings_split_two_index_zero_cut",
        "strings",
        Any,
        "`strings.Split(...)[0]` when only the first segment is needed.",
        ["{alias}.Split(", "[0]"],
        ["SplitN("]
    ),
    alias_rule!(
        "strings_split_two_index_one_cut",
        "strings",
        Any,
        "`strings.Split(...)[1]` when only the second segment is needed.",
        ["{alias}.Split(", "[1]"],
        ["SplitN("]
    ),
    alias_rule!(
        "bytes_split_two_index_zero_cut",
        "bytes",
        Any,
        "`bytes.Split(...)[0]` when only the first segment is needed.",
        ["{alias}.Split(", "[0]"],
        ["SplitN("]
    ),
    alias_rule!(
        "bytes_split_two_index_one_cut",
        "bytes",
        Any,
        "`bytes.Split(...)[1]` when only the second segment is needed.",
        ["{alias}.Split(", "[1]"],
        ["SplitN("]
    ),
    alias_rule!(
        "strings_splitaftern_two_index_zero_cut",
        "strings",
        Any,
        "`strings.SplitAfterN(..., 2)[0]` instead of `strings.Cut(...)` plus delimiter handling.",
        ["{alias}.SplitAfterN(", ", 2)", "[0]"],
        []
    ),
    alias_rule!(
        "strings_splitaftern_two_index_one_cut",
        "strings",
        Any,
        "`strings.SplitAfterN(..., 2)[1]` instead of `strings.Cut(...)` plus delimiter handling.",
        ["{alias}.SplitAfterN(", ", 2)", "[1]"],
        []
    ),
    alias_rule!(
        "bytes_splitaftern_two_index_zero_cut",
        "bytes",
        Any,
        "`bytes.SplitAfterN(..., 2)[0]` instead of `bytes.Cut(...)` plus delimiter handling.",
        ["{alias}.SplitAfterN(", ", 2)", "[0]"],
        []
    ),
    alias_rule!(
        "bytes_splitaftern_two_index_one_cut",
        "bytes",
        Any,
        "`bytes.SplitAfterN(..., 2)[1]` instead of `bytes.Cut(...)` plus delimiter handling.",
        ["{alias}.SplitAfterN(", ", 2)", "[1]"],
        []
    ),
    alias_rule!(
        "strings_splitafter_two_index_zero_cut",
        "strings",
        Any,
        "`strings.SplitAfter(...)[0]` when only the first segment is needed.",
        ["{alias}.SplitAfter(", "[0]"],
        ["SplitAfterN("]
    ),
    alias_rule!(
        "strings_splitafter_two_index_one_cut",
        "strings",
        Any,
        "`strings.SplitAfter(...)[1]` when only the second segment is needed.",
        ["{alias}.SplitAfter(", "[1]"],
        ["SplitAfterN("]
    ),
    alias_rule!(
        "bytes_splitafter_two_index_zero_cut",
        "bytes",
        Any,
        "`bytes.SplitAfter(...)[0]` when only the first segment is needed.",
        ["{alias}.SplitAfter(", "[0]"],
        ["SplitAfterN("]
    ),
    alias_rule!(
        "bytes_splitafter_two_index_one_cut",
        "bytes",
        Any,
        "`bytes.SplitAfter(...)[1]` when only the second segment is needed.",
        ["{alias}.SplitAfter(", "[1]"],
        ["SplitAfterN("]
    ),
    alias_rule!(
        "strings_tolower_equalfold",
        "strings",
        Any,
        "`strings.ToLower(...) == strings.ToLower(...)` instead of `strings.EqualFold(...)`.",
        ["{alias}.ToLower(", "=="],
        []
    ),
    alias_rule!(
        "strings_toupper_equalfold",
        "strings",
        Any,
        "`strings.ToUpper(...) == strings.ToUpper(...)` instead of `strings.EqualFold(...)`.",
        ["{alias}.ToUpper(", "=="],
        []
    ),
    alias_rule!(
        "bytes_tolower_equalfold",
        "bytes",
        Any,
        "`bytes.Equal(bytes.ToLower(...), bytes.ToLower(...))` instead of `bytes.EqualFold(...)`.",
        ["{alias}.Equal(", "{alias}.ToLower("],
        []
    ),
    alias_rule!(
        "bytes_toupper_equalfold",
        "bytes",
        Any,
        "`bytes.Equal(bytes.ToUpper(...), bytes.ToUpper(...))` instead of `bytes.EqualFold(...)`.",
        ["{alias}.Equal(", "{alias}.ToUpper("],
        []
    ),
    alias_rule!(
        "bytes_newreader_on_string_conversion",
        "bytes",
        Any,
        "`bytes.NewReader([]byte(s))` instead of `strings.NewReader(s)`.",
        ["{alias}.NewReader([]byte("],
        []
    ),
    alias_rule!(
        "strings_newreader_on_byte_slice_conversion",
        "strings",
        Any,
        "`strings.NewReader(string(b))` instead of `bytes.NewReader(b)`.",
        ["{alias}.NewReader(string("],
        []
    ),
    alias_rule!(
        "bytes_newbufferstring_on_string_conversion",
        "bytes",
        Any,
        "`bytes.NewBufferString(string(b))` instead of using the byte slice directly.",
        ["{alias}.NewBufferString(string("],
        []
    ),
    alias_rule!(
        "strings_replace_neg_one_replaceall",
        "strings",
        Any,
        "`strings.Replace(..., -1)` instead of `strings.ReplaceAll(...)`.",
        ["{alias}.Replace(", ", -1)"],
        []
    ),
    alias_rule!(
        "bytes_replace_neg_one_replaceall",
        "bytes",
        Any,
        "`bytes.Replace(..., -1)` instead of `bytes.ReplaceAll(...)`.",
        ["{alias}.Replace(", ", -1)"],
        []
    ),
    alias_rule!(
        "strings_trimleft_space_trimspace",
        "strings",
        Any,
        "`strings.TrimLeft(..., whitespace)` instead of `strings.TrimSpace(...)`.",
        ["{alias}.TrimLeft(", "\" \\t\\r\\n\""],
        []
    ),
    alias_rule!(
        "strings_trimright_space_trimspace",
        "strings",
        Any,
        "`strings.TrimRight(..., whitespace)` instead of `strings.TrimSpace(...)`.",
        ["{alias}.TrimRight(", "\" \\t\\r\\n\""],
        []
    ),
    alias_rule!(
        "bytes_trimleft_space_trimspace",
        "bytes",
        Any,
        "`bytes.TrimLeft(..., whitespace)` instead of `bytes.TrimSpace(...)`.",
        ["{alias}.TrimLeft(", "\" \\t\\r\\n\""],
        []
    ),
    alias_rule!(
        "bytes_trimright_space_trimspace",
        "bytes",
        Any,
        "`bytes.TrimRight(..., whitespace)` instead of `bytes.TrimSpace(...)`.",
        ["{alias}.TrimRight(", "\" \\t\\r\\n\""],
        []
    ),
    alias_rule!(
        "filepath_split_base_only",
        "path/filepath",
        Any,
        "`filepath.Split(...)` when only the base name is used.",
        ["_,", "{alias}.Split("],
        []
    ),
    alias_rule!(
        "filepath_split_dir_only",
        "path/filepath",
        Any,
        "`filepath.Split(...)` when only the directory is used.",
        [", _ :=", "{alias}.Split("],
        []
    ),
    alias_rule!(
        "path_split_base_only",
        "path",
        Any,
        "`path.Split(...)` when only the base name is used.",
        ["_,", "{alias}.Split("],
        []
    ),
    alias_rule!(
        "path_split_dir_only",
        "path",
        Any,
        "`path.Split(...)` when only the directory is used.",
        [", _ :=", "{alias}.Split("],
        []
    ),
    alias_rule!(
        "bytes_hasprefix_manual_slice_after_len",
        "bytes",
        Any,
        "Manual `len` and slice prefix checks instead of `bytes.HasPrefix(...)`.",
        ["len(", "{alias}.Equal(", "[:len("],
        []
    ),
    alias_rule!(
        "bytes_hassuffix_manual_slice_after_len",
        "bytes",
        Any,
        "Manual `len` and slice suffix checks instead of `bytes.HasSuffix(...)`.",
        ["len(", "{alias}.Equal(", "-len(", ":]"],
        []
    ),
    alias_rule!(
        "fmt_sprintf_bool_to_string",
        "fmt",
        Any,
        "`fmt.Sprintf(\"%t\", ...)` instead of `strconv.FormatBool(...)`.",
        ["{alias}.Sprintf(\"%t\""],
        []
    ),
    alias_rule!(
        "fmt_sprintf_float_to_string",
        "fmt",
        Any,
        "`fmt.Sprintf(\"%f\", ...)` instead of `strconv.FormatFloat(...)` when only a float string is needed.",
        ["{alias}.Sprintf(\"%f\""],
        []
    ),
    alias_rule!(
        "fmt_sprintf_binary_to_string",
        "fmt",
        Any,
        "`fmt.Sprintf(\"%b\", ...)` instead of `strconv.FormatInt(...)` or `FormatUint(...)`.",
        ["{alias}.Sprintf(\"%b\""],
        []
    ),
    alias_rule!(
        "fmt_sprintf_octal_to_string",
        "fmt",
        Any,
        "`fmt.Sprintf(\"%o\", ...)` instead of `strconv.FormatInt(...)` or `FormatUint(...)`.",
        ["{alias}.Sprintf(\"%o\""],
        []
    ),
    alias_rule!(
        "fmt_sprintf_hex_to_string",
        "fmt",
        Any,
        "`fmt.Sprintf(\"%x\", ...)` instead of a direct hex formatter when only the string is needed.",
        ["{alias}.Sprintf(\"%x\""],
        []
    ),
    alias_rule!(
        "fmt_sprintf_quote_to_string",
        "fmt",
        Any,
        "`fmt.Sprintf(\"%q\", s)` instead of `strconv.Quote(s)`.",
        ["{alias}.Sprintf(\"%q\""],
        []
    ),
    alias_rule!(
        "fmt_sprintf_single_string_passthrough",
        "fmt",
        Any,
        "`fmt.Sprintf(\"%s\", s)` instead of returning or writing the string directly.",
        ["{alias}.Sprintf(\"%s\""],
        []
    ),
    alias_rule!(
        "strconv_formatint_int64_cast_itoa",
        "strconv",
        Any,
        "`strconv.FormatInt(int64(v), 10)` instead of `strconv.Itoa(v)`.",
        ["{alias}.FormatInt(int64(", ", 10)"],
        []
    ),
    alias_rule!(
        "time_tick_per_call",
        "time",
        Any,
        "`time.Tick(...)` in regular call paths instead of owning a reusable ticker.",
        ["{alias}.Tick("],
        []
    ),
    alias_rule!(
        "time_newtimer_inside_loop",
        "time",
        Loop,
        "`time.NewTimer(...)` inside loops.",
        ["{alias}.NewTimer("],
        []
    ),
    alias_rule!(
        "time_newticker_inside_loop",
        "time",
        Loop,
        "`time.NewTicker(...)` inside loops.",
        ["{alias}.NewTicker("],
        []
    ),
    alias_rule!(
        "rand_seed_per_call",
        "math/rand",
        Any,
        "`rand.Seed(...)` inside regular call paths instead of process startup.",
        ["{alias}.Seed("],
        []
    ),
    alias_rule!(
        "rand_newsource_per_call",
        "math/rand",
        Any,
        "`rand.NewSource(...)` inside regular call paths.",
        ["{alias}.NewSource("],
        []
    ),
    alias_rule!(
        "rand_newsource_with_time_now_per_call",
        "math/rand",
        Any,
        "`rand.NewSource(...)` seeded from wall clock on each call path.",
        ["{alias}.NewSource(", "UnixNano("],
        []
    ),
    alias_rule!(
        "rand_new_per_call",
        "math/rand",
        Any,
        "`rand.New(...)` inside regular call paths instead of reusing a source or generator.",
        ["{alias}.New("],
        []
    ),
    alias_rule!(
        "time_since_candidate_via_now_sub",
        "time",
        Any,
        "`time.Now().Sub(start)` instead of `time.Since(start)`.",
        ["{alias}.Now().Sub("],
        []
    ),
    alias_rule!(
        "time_until_candidate_via_deadline_sub_now",
        "time",
        Any,
        "`deadline.Sub(time.Now())` instead of `time.Until(deadline)`.",
        [".Sub(", "{alias}.Now()"],
        []
    ),
    alias_rule!(
        "runtime_numcpu_inside_loop",
        "runtime",
        Loop,
        "`runtime.NumCPU()` inside loops instead of caching once.",
        ["{alias}.NumCPU("],
        []
    ),
    alias_rule!(
        "runtime_gomaxprocs_per_request",
        "runtime",
        Request,
        "`runtime.GOMAXPROCS(...)` on request paths.",
        ["{alias}.GOMAXPROCS("],
        []
    ),
    alias_rule!(
        "time_loadlocation_per_call",
        "time",
        RequestOrLoop,
        "`time.LoadLocation(...)` inside request or loop paths instead of reusing the location.",
        ["{alias}.LoadLocation("],
        []
    ),
    alias_rule!(
        "time_fixedzone_per_call",
        "time",
        RequestOrLoop,
        "`time.FixedZone(...)` inside request or loop paths instead of reusing the location.",
        ["{alias}.FixedZone("],
        []
    ),
    alias_rule!(
        "context_withtimeout_inside_loop",
        "context",
        Loop,
        "`context.WithTimeout(...)` inside loops.",
        ["{alias}.WithTimeout("],
        []
    ),
    alias_rule!(
        "json_marshalindent_in_loop",
        "encoding/json",
        Loop,
        "`json.MarshalIndent(...)` inside loops.",
        ["{alias}.MarshalIndent("],
        []
    ),
    alias_rule!(
        "json_indent_in_loop",
        "encoding/json",
        Loop,
        "`json.Indent(...)` inside loops.",
        ["{alias}.Indent("],
        []
    ),
    alias_rule!(
        "base64_encode_to_string_in_loop",
        "encoding/base64",
        Loop,
        "Base64 encoding to string inside loops.",
        ["{alias}.", "EncodeToString("],
        []
    ),
    alias_rule!(
        "base64_decode_string_in_loop",
        "encoding/base64",
        Loop,
        "Base64 decode from string inside loops.",
        ["{alias}.", "DecodeString("],
        []
    ),
    alias_rule!(
        "hex_encode_to_string_in_loop",
        "encoding/hex",
        Loop,
        "`hex.EncodeToString(...)` inside loops.",
        ["{alias}.EncodeToString("],
        []
    ),
    alias_rule!(
        "hex_decode_string_in_loop",
        "encoding/hex",
        Loop,
        "`hex.DecodeString(...)` inside loops.",
        ["{alias}.DecodeString("],
        []
    ),
    alias_rule!(
        "sha1_sum_in_loop",
        "crypto/sha1",
        Loop,
        "`sha1.Sum(...)` inside loops.",
        ["{alias}.Sum("],
        []
    ),
    alias_rule!(
        "sha256_sum_in_loop",
        "crypto/sha256",
        Loop,
        "`sha256.Sum256(...)` inside loops.",
        ["{alias}.Sum256("],
        []
    ),
    alias_rule!(
        "sha512_sum_in_loop",
        "crypto/sha512",
        Loop,
        "`sha512.Sum512(...)` inside loops.",
        ["{alias}.Sum512("],
        []
    ),
    alias_rule!(
        "md5_sum_in_loop",
        "crypto/md5",
        Loop,
        "`md5.Sum(...)` inside loops.",
        ["{alias}.Sum("],
        []
    ),
    alias_rule!(
        "crc32_checksum_in_loop",
        "hash/crc32",
        Loop,
        "`crc32.Checksum...` inside loops.",
        ["{alias}.Checksum"],
        []
    ),
    alias_rule!(
        "crc64_checksum_in_loop",
        "hash/crc64",
        Loop,
        "`crc64.Checksum(...)` inside loops.",
        ["{alias}.Checksum("],
        []
    ),
    alias_rule!(
        "adler32_checksum_in_loop",
        "hash/adler32",
        Loop,
        "`adler32.Checksum(...)` inside loops.",
        ["{alias}.Checksum("],
        []
    ),
    alias_rule!(
        "hmac_new_in_loop",
        "crypto/hmac",
        Loop,
        "`hmac.New(...)` inside loops.",
        ["{alias}.New("],
        []
    ),
    alias_rule!(
        "strings_newreplacer_per_call",
        "strings",
        RequestOrLoop,
        "`strings.NewReplacer(...)` recreated on request or loop paths.",
        ["{alias}.NewReplacer("],
        []
    ),
];

const PLAIN_RULES: &[PlainRuleSpec] = &[
    plain_rule!(
        "writer_write_byte_slice_of_string",
        Any,
        "`writer.Write([]byte(s))` instead of `io.WriteString(writer, s)`.",
        [".Write([]byte("],
        []
    ),
    plain_rule!(
        "strings_hasprefix_manual_slice_after_len",
        Any,
        "Manual `len` and slice prefix checks instead of `strings.HasPrefix(...)`.",
        ["len(", "[:len(", "&&", "=="],
        []
    ),
    plain_rule!(
        "strings_hassuffix_manual_slice_after_len",
        Any,
        "Manual `len` and slice suffix checks instead of `strings.HasSuffix(...)`.",
        ["len(", "-len(", ":]", "&&", "=="],
        []
    ),
    plain_rule!(
        "duration_nanoseconds_zero_check",
        Any,
        "Duration zero checks written as `d.Nanoseconds() == 0` instead of `d == 0`.",
        [".Nanoseconds()", "== 0"],
        []
    ),
];

pub(crate) fn extra_performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if file.is_test_file || function.is_test_function {
        return Vec::new();
    }

    let lines = body_lines(function);
    let request_path = is_request_path_function(file, function);
    let builder_names = collect_builder_names(file, &lines);
    let buffer_names = collect_buffer_names(file, &lines);
    let once_names = collect_once_names(file, &lines);

    let mut findings = Vec::new();
    findings.extend(alias_rule_findings(file, function, &lines, request_path));
    findings.extend(plain_rule_findings(file, function, &lines, request_path));
    findings.extend(builder_buffer_findings(
        file,
        function,
        &lines,
        &builder_names,
        &buffer_names,
    ));
    findings.extend(sync_once_findings(file, function, &lines, &once_names));
    findings.extend(json_valid_then_unmarshal_findings(file, function, &lines));
    findings
}

fn alias_rule_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for spec in ALIAS_RULES {
        let mut matched_line = None;

        for alias in import_aliases_for(file, spec.import_path) {
            matched_line = lines
                .iter()
                .find(|line| {
                    scope_matches(spec.scope, line, request_path)
                        && alias_patterns_match(
                            line.text.as_str(),
                            &alias,
                            spec.required,
                            spec.excluded,
                        )
                })
                .map(|line| line.line);

            if matched_line.is_some() {
                break;
            }
        }

        if let Some(line) = matched_line {
            findings.push(performance_finding(
                file,
                function,
                spec.id,
                line,
                spec.summary,
            ));
        }
    }

    findings
}

fn plain_rule_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    request_path: bool,
) -> Vec<Finding> {
    PLAIN_RULES
        .iter()
        .filter_map(|spec| {
            lines
                .iter()
                .find(|line| {
                    scope_matches(spec.scope, line, request_path)
                        && plain_patterns_match(line.text.as_str(), spec.required, spec.excluded)
                })
                .map(|line| performance_finding(file, function, spec.id, line.line, spec.summary))
        })
        .collect()
}

fn builder_buffer_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    builder_names: &BTreeSet<String>,
    buffer_names: &BTreeSet<String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let fmt_aliases = import_aliases_for(file, "fmt");

    for line in lines {
        if let Some(receiver) = receiver_for_method(line.text.as_str(), "WriteString")
            && single_byte_string_literal(line.text.as_str())
        {
            if builder_names.contains(receiver.as_str()) {
                findings.push(performance_finding(
                    file,
                    function,
                    "builder_write_string_single_byte_literal",
                    line.line,
                    "`strings.Builder.WriteString(\"x\")` instead of `WriteByte('x')`.",
                ));
            }
            if buffer_names.contains(receiver.as_str()) {
                findings.push(performance_finding(
                    file,
                    function,
                    "buffer_write_string_single_byte_literal",
                    line.line,
                    "`bytes.Buffer.WriteString(\"x\")` instead of `WriteByte('x')`.",
                ));
            }
        }

        if let Some(receiver) = receiver_for_method(line.text.as_str(), "WriteRune")
            && ascii_rune_literal(line.text.as_str())
        {
            if builder_names.contains(receiver.as_str()) {
                findings.push(performance_finding(
                    file,
                    function,
                    "builder_write_rune_ascii_literal",
                    line.line,
                    "`strings.Builder.WriteRune('x')` for an ASCII literal instead of `WriteByte`.",
                ));
            }
            if buffer_names.contains(receiver.as_str()) {
                findings.push(performance_finding(
                    file,
                    function,
                    "buffer_write_rune_ascii_literal",
                    line.line,
                    "`bytes.Buffer.WriteRune('x')` for an ASCII literal instead of `WriteByte`.",
                ));
            }
        }

        for name in buffer_names {
            if line.text.contains(&format!("len({name}.String())")) {
                findings.push(performance_finding(
                    file,
                    function,
                    "bytes_buffer_string_len",
                    line.line,
                    "`len(buf.String())` instead of `buf.Len()`.",
                ));
            }

            if line.text.contains(&format!("{name}.Truncate(0)")) {
                findings.push(performance_finding(
                    file,
                    function,
                    "bytes_buffer_truncate_zero_reset",
                    line.line,
                    "`buf.Truncate(0)` instead of `buf.Reset()`.",
                ));
            }

            for alias in &fmt_aliases {
                if line.text.contains(&format!("{alias}.Fprint(&{name},")) {
                    findings.push(performance_finding(
                        file,
                        function,
                        "fmt_fprint_to_bytes_buffer",
                        line.line,
                        "`fmt.Fprint(&buf, ...)` instead of direct buffer writes.",
                    ));
                }
                if line.text.contains(&format!("{alias}.Fprintln(&{name},")) {
                    findings.push(performance_finding(
                        file,
                        function,
                        "fmt_fprintln_to_bytes_buffer",
                        line.line,
                        "`fmt.Fprintln(&buf, ...)` instead of direct buffer writes.",
                    ));
                }
                if line
                    .text
                    .contains(&format!("{alias}.Fprintf(&{name}, \"%s\""))
                {
                    findings.push(performance_finding(
                        file,
                        function,
                        "fmt_fprintf_single_string_to_bytes_buffer",
                        line.line,
                        "`fmt.Fprintf(&buf, \"%s\", s)` instead of `WriteString(s)`.",
                    ));
                }
            }
        }

        for name in builder_names {
            if line.text.contains(&format!("len({name}.String())")) {
                findings.push(performance_finding(
                    file,
                    function,
                    "strings_builder_string_len",
                    line.line,
                    "`len(builder.String())` instead of `builder.Len()`.",
                ));
            }

            for alias in &fmt_aliases {
                if line.text.contains(&format!("{alias}.Fprint(&{name},")) {
                    findings.push(performance_finding(
                        file,
                        function,
                        "fmt_fprint_to_strings_builder",
                        line.line,
                        "`fmt.Fprint(&builder, ...)` instead of direct builder writes.",
                    ));
                }
                if line.text.contains(&format!("{alias}.Fprintln(&{name},")) {
                    findings.push(performance_finding(
                        file,
                        function,
                        "fmt_fprintln_to_strings_builder",
                        line.line,
                        "`fmt.Fprintln(&builder, ...)` instead of direct builder writes.",
                    ));
                }
                if line
                    .text
                    .contains(&format!("{alias}.Fprintf(&{name}, \"%s\""))
                {
                    findings.push(performance_finding(
                        file,
                        function,
                        "fmt_fprintf_single_string_to_strings_builder",
                        line.line,
                        "`fmt.Fprintf(&builder, \"%s\", s)` instead of `WriteString(s)`.",
                    ));
                }
            }
        }
    }

    dedupe_findings(findings)
}

fn sync_once_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
    once_names: &BTreeSet<String>,
) -> Vec<Finding> {
    let has_once_signal = !once_names.is_empty()
        || function.body_text.contains("sync.Once")
        || function.body_text.contains("Once{}");
    if !has_once_signal {
        return Vec::new();
    }

    lines
        .iter()
        .find_map(|line| {
            line.text.contains(".Do(").then(|| {
                performance_finding(
                    file,
                    function,
                    "sync_once_do_inside_loop",
                    line.line,
                    "`sync.Once.Do(...)` inside loops.",
                )
            })
        })
        .into_iter()
        .collect()
}

fn json_valid_then_unmarshal_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    for alias in import_aliases_for(file, "encoding/json") {
        let valid_line = lines
            .iter()
            .find(|line| line.text.contains(&format!("{alias}.Valid(")))
            .map(|line| line.line);
        let unmarshal_line = lines
            .iter()
            .find(|line| line.text.contains(&format!("{alias}.Unmarshal(")))
            .map(|line| line.line);

        if let (Some(valid_line), Some(unmarshal_line)) = (valid_line, unmarshal_line) {
            return vec![Finding {
                rule_id: "json_valid_then_unmarshal".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: unmarshal_line,
                end_line: unmarshal_line,
                message: format!(
                    "function {} validates JSON and then unmarshals it again",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("{alias}.Valid(...) observed at line {valid_line}"),
                    format!("{alias}.Unmarshal(...) observed at line {unmarshal_line}"),
                    "double JSON parsing often wastes work on the same payload".to_string(),
                ],
            }];
        }
    }

    Vec::new()
}

fn performance_finding(
    file: &ParsedFile,
    function: &ParsedFunction,
    rule_id: &str,
    line: usize,
    summary: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!("function {} uses {}", function.fingerprint.name, summary),
        evidence: vec![summary.to_string()],
    }
}

fn scope_matches(scope: Scope, line: &BodyLine, request_path: bool) -> bool {
    match scope {
        Scope::Any => true,
        Scope::Loop => line.in_loop,
        Scope::Request => request_path,
        Scope::RequestOrLoop => request_path || line.in_loop,
    }
}

fn alias_patterns_match(text: &str, alias: &str, required: &[&str], excluded: &[&str]) -> bool {
    required
        .iter()
        .all(|pattern| text.contains(&pattern.replace("{alias}", alias)))
        && excluded
            .iter()
            .all(|pattern| !text.contains(&pattern.replace("{alias}", alias)))
}

fn plain_patterns_match(text: &str, required: &[&str], excluded: &[&str]) -> bool {
    required.iter().all(|pattern| text.contains(pattern))
        && excluded.iter().all(|pattern| !text.contains(pattern))
}

fn collect_builder_names(file: &ParsedFile, lines: &[BodyLine]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for alias in import_aliases_for(file, "strings") {
        let type_marker = format!("{alias}.Builder");
        let literal_marker = format!("{alias}.Builder{{}}");
        let ptr_literal_marker = format!("&{alias}.Builder{{}}");

        for line in lines {
            if let Some(name) = binding_name_for_var_type(line.text.as_str(), &type_marker) {
                names.insert(name);
            }
            if let Some(name) = binding_name_for_assignment(line.text.as_str(), &literal_marker) {
                names.insert(name);
            }
            if let Some(name) = binding_name_for_assignment(line.text.as_str(), &ptr_literal_marker)
            {
                names.insert(name);
            }
        }
    }

    names
}

fn collect_buffer_names(file: &ParsedFile, lines: &[BodyLine]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for alias in import_aliases_for(file, "bytes") {
        let type_marker = format!("{alias}.Buffer");
        let literal_marker = format!("{alias}.Buffer{{}}");
        let ptr_literal_marker = format!("&{alias}.Buffer{{}}");
        let new_buffer_marker = format!("{alias}.NewBuffer(");
        let new_buffer_string_marker = format!("{alias}.NewBufferString(");

        for line in lines {
            if let Some(name) = binding_name_for_var_type(line.text.as_str(), &type_marker) {
                names.insert(name);
            }
            if let Some(name) = binding_name_for_assignment(line.text.as_str(), &literal_marker) {
                names.insert(name);
            }
            if let Some(name) = binding_name_for_assignment(line.text.as_str(), &ptr_literal_marker)
            {
                names.insert(name);
            }
            if let Some(name) = binding_name_for_assignment(line.text.as_str(), &new_buffer_marker)
            {
                names.insert(name);
            }
            if let Some(name) =
                binding_name_for_assignment(line.text.as_str(), &new_buffer_string_marker)
            {
                names.insert(name);
            }
        }
    }

    names
}

fn collect_once_names(file: &ParsedFile, lines: &[BodyLine]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for alias in import_aliases_for(file, "sync") {
        let type_marker = format!("{alias}.Once");
        let literal_marker = format!("{alias}.Once{{}}");

        for line in lines {
            if let Some(name) = binding_name_for_var_type(line.text.as_str(), &type_marker) {
                names.insert(name);
            }
            if let Some(name) = binding_name_for_assignment(line.text.as_str(), &literal_marker) {
                names.insert(name);
            }
        }
    }

    names
}

fn binding_name_for_var_type(text: &str, type_marker: &str) -> Option<String> {
    let after = text.strip_prefix("var ")?;
    let mut parts = after.split_whitespace();
    let name = parts.next()?.trim_matches(',');
    let type_text = parts.next()?;
    simple_identifier(name)
        .filter(|_| type_text.contains(type_marker))
        .map(ToOwned::to_owned)
}

fn binding_name_for_assignment(text: &str, rhs_marker: &str) -> Option<String> {
    let (left, right) = split_assignment(text)?;
    if !right.contains(rhs_marker) {
        return None;
    }

    let name = left
        .split(',')
        .next()?
        .split_whitespace()
        .last()?
        .trim_matches('&')
        .trim_matches('*');

    simple_identifier(name).map(ToOwned::to_owned)
}

fn simple_identifier(candidate: &str) -> Option<&str> {
    (!candidate.is_empty()
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
    .then_some(candidate)
}

fn receiver_for_method(text: &str, method: &str) -> Option<String> {
    let needle = format!(".{method}(");
    let (left, _) = text.split_once(&needle)?;
    let receiver = left
        .trim_end()
        .rsplit_once(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .map(|(_, suffix)| suffix)
        .unwrap_or(left)
        .trim()
        .trim_matches('&')
        .trim_matches('*');

    simple_identifier(receiver).map(ToOwned::to_owned)
}

fn single_byte_string_literal(text: &str) -> bool {
    let Some(start) = text.find('"') else {
        return false;
    };
    let rest = &text[start + 1..];
    let Some(end) = rest.find('"') else {
        return false;
    };
    rest[..end].len() == 1
}

fn ascii_rune_literal(text: &str) -> bool {
    let Some(start) = text.find('\'') else {
        return false;
    };
    let rest = &text[start + 1..];
    let Some(end) = rest.find('\'') else {
        return false;
    };
    rest[..end].len() == 1
}

fn dedupe_findings(findings: Vec<Finding>) -> Vec<Finding> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();

    for finding in findings {
        let key = (
            finding.rule_id.clone(),
            finding.function_name.clone(),
            finding.start_line,
        );
        if seen.insert(key) {
            deduped.push(finding);
        }
    }

    deduped
}
