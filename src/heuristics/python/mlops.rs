use crate::analysis::{ParsedFile, ParsedFunction};

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}
use crate::model::{Finding, Severity};

// ── Import-gating helpers ─────────────────────────────────────────────────────

fn has_import(file: &ParsedFile, module: &str) -> bool {
    file.imports
        .iter()
        .any(|imp| imp.path.contains(module) || imp.alias.contains(module))
}

fn has_any_import(file: &ParsedFile, modules: &[&str]) -> bool {
    modules.iter().any(|m| has_import(file, m))
}

fn is_handler_or_view(function: &ParsedFunction) -> bool {
    let sig = &function.signature_text;
    sig.contains("@app.route")
        || sig.contains("@bp.route")
        || sig.contains("@router.")
        || sig.contains("@api_view")
        || sig.contains("@app.get")
        || sig.contains("@app.post")
}

// ── Pandas rules ─────────────────────────────────────────────────────────────

pub(super) fn pandas_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_any_import(file, &["pandas", "pd"]) {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // pandas_iterrows_in_loop
    if body.contains(".iterrows()") {
        if let Some(line) = find_line(body, ".iterrows()", function.fingerprint.start_line) {
            findings.push(make_finding(
                "pandas_iterrows_in_loop", Severity::Info, file, function, line,
                "uses df.iterrows() which is very slow; prefer vectorized ops, .apply(), or .itertuples()",
            ));
        }
    }

    // pandas_apply_with_simple_vectorizable_op
    if body.contains(".apply(lambda") {
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains(".apply(lambda") {
                // Check for simple operations that have vectorized equivalents
                let has_simple_op = trimmed.contains("x + ")
                    || trimmed.contains("x - ")
                    || trimmed.contains("x * ")
                    || trimmed.contains("x / ")
                    || trimmed.contains("x.lower()")
                    || trimmed.contains("x.upper()")
                    || trimmed.contains("x.strip()")
                    || trimmed.contains("str(x)")
                    || trimmed.contains("int(x)")
                    || trimmed.contains("float(x)");
                if has_simple_op {
                    findings.push(make_finding(
                        "pandas_apply_with_simple_vectorizable_op", Severity::Info, file, function,
                        function.fingerprint.start_line + i,
                        "uses .apply(lambda) for a simple operation with a vectorized equivalent",
                    ));
                }
            }
        }
    }

    // pandas_concat_in_loop
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() {
            if trimmed.contains("pd.concat(") || trimmed.contains("df.append(") || trimmed.contains(".append(") && trimmed.contains("DataFrame") {
                findings.push(make_finding(
                    "pandas_concat_in_loop", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "concatenates DataFrames inside a loop; collect all and concat once",
                ));
            }
        }
        if let Some(li) = loop_indent {
            if !trimmed.is_empty() && indent_level(line) <= li && !trimmed.starts_with('#') {
                loop_indent = None;
            }
        }
    }

    // pandas_read_csv_without_dtypes
    if body.contains("pd.read_csv(") || body.contains("read_csv(") {
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("read_csv(") && !trimmed.contains("dtype") {
                findings.push(make_finding(
                    "pandas_read_csv_without_dtypes", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "reads CSV without dtype parameter; specify dtypes to avoid double-pass type inference",
                ));
            }
        }
    }

    // pandas_chain_assignment_warning
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        // Detect df['a']['b'] = or df.col1.col2 = chained assignment
        if trimmed.contains("']['") && trimmed.contains(" = ") && !trimmed.contains("==") {
            findings.push(make_finding(
                "pandas_chain_assignment_warning", Severity::Info, file, function,
                function.fingerprint.start_line + i,
                "uses chained indexing which may cause SettingWithCopyWarning; use .loc[] instead",
            ));
        }
    }

    // pandas_inplace_false_reassignment_missing
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains(".drop(") || trimmed.contains(".rename(") || trimmed.contains(".fillna(") || trimmed.contains(".dropna("))
            && !trimmed.contains("inplace=True")
            && !trimmed.contains(" = ")
            && !trimmed.starts_with("return")
        {
            findings.push(make_finding(
                "pandas_inplace_false_reassignment_missing", Severity::Info, file, function,
                function.fingerprint.start_line + i,
                "calls DataFrame method without assigning result or inplace=True; result is silently discarded",
            ));
        }
    }

    // pandas_to_dict_records_in_loop
    loop_indent = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() && trimmed.contains(".to_dict(") {
            findings.push(make_finding(
                "pandas_to_dict_records_in_loop", Severity::Info, file, function,
                function.fingerprint.start_line + i,
                "calls .to_dict() inside a loop; use vectorized access or .itertuples()",
            ));
        }
        if let Some(li) = loop_indent {
            if !trimmed.is_empty() && indent_level(line) <= li && !trimmed.starts_with('#') {
                loop_indent = None;
            }
        }
    }

    // pandas_merge_without_validation
    if body.contains(".merge(") || body.contains("pd.merge(") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains(".merge(") && !trimmed.contains("validate") {
                findings.push(make_finding(
                    "pandas_merge_without_validation", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "merges DataFrames without validate parameter; risk of silent row duplication",
                ));
            }
        }
    }

    // pandas_full_dataframe_print_in_production
    if !function.is_test_function {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("print(df") || trimmed.starts_with("display(df") || trimmed.contains(".to_string()"))
                && !file.path.ends_with("notebook.py")
                && !file.path.to_string_lossy().contains("notebook")
            {
                findings.push(make_finding(
                    "pandas_full_dataframe_print_in_production", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "prints/displays a full DataFrame in production code; use logging or head()",
                ));
            }
        }
    }

    // pandas_eval_string_manipulation
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains(".eval(f") || trimmed.contains(".query(f"))
            && (trimmed.contains("{") || trimmed.contains("format("))
        {
            findings.push(make_finding(
                "pandas_eval_string_manipulation", Severity::Warning, file, function,
                function.fingerprint.start_line + i,
                "uses f-string in .eval()/.query() which risks injection; use parameterized operations",
            ));
        }
    }

    // pandas_copy_in_loop
    loop_indent = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() && trimmed.contains(".copy()") && (trimmed.contains("df") || trimmed.contains("DataFrame")) {
            findings.push(make_finding(
                "pandas_copy_in_loop", Severity::Info, file, function,
                function.fingerprint.start_line + i,
                "copies a DataFrame inside a loop; consider using views or method chaining",
            ));
        }
        if let Some(li) = loop_indent {
            if !trimmed.is_empty() && indent_level(line) <= li && !trimmed.starts_with('#') {
                loop_indent = None;
            }
        }
    }

    findings
}

// ── NumPy rules ──────────────────────────────────────────────────────────────

pub(super) fn numpy_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_any_import(file, &["numpy", "np"]) {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // numpy_python_loop_over_array
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            // Check if iterating over something that looks like a numpy array
            if trimmed.contains("np.") || trimmed.contains("array") || trimmed.contains("range(len(") {
                findings.push(make_finding(
                    "numpy_python_loop_over_array", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "uses Python loop over array; prefer vectorized NumPy operations",
                ));
            }
        }
    }

    // numpy_append_in_loop
    let mut loop_indent: Option<usize> = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() && trimmed.contains("np.append(") {
            findings.push(make_finding(
                "numpy_append_in_loop", Severity::Info, file, function,
                function.fingerprint.start_line + i,
                "uses np.append() in a loop; pre-allocate with np.zeros/np.empty and fill",
            ));
        }
        if loop_indent.is_some() && !trimmed.is_empty() && (trimmed.contains("np.vstack(") || trimmed.contains("np.hstack(") || trimmed.contains("np.concatenate(")) {
            findings.push(make_finding(
                "numpy_vstack_hstack_in_loop", Severity::Info, file, function,
                function.fingerprint.start_line + i,
                "stacks arrays in a loop; collect and stack once after the loop",
            ));
        }
        if let Some(li) = loop_indent {
            if !trimmed.is_empty() && indent_level(line) <= li && !trimmed.starts_with('#') {
                loop_indent = None;
            }
        }
    }

    // numpy_tolist_in_hot_path
    if body.contains(".tolist()") {
        let in_loop = body.lines().enumerate().any(|(_, line)| {
            let t = line.trim();
            t.contains(".tolist()") && (body.contains("for ") || body.contains("while "))
        });
        if in_loop {
            if let Some(line) = find_line(body, ".tolist()", function.fingerprint.start_line) {
                findings.push(make_finding(
                    "numpy_tolist_in_hot_path", Severity::Info, file, function, line,
                    "calls .tolist() in hot path; keep data as NumPy arrays to avoid Python object overhead",
                ));
            }
        }
    }

    findings
}

// ── Model inference rules ────────────────────────────────────────────────────

pub(super) fn model_inference_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    let ml_imports = has_any_import(file, &["torch", "tensorflow", "tf", "sklearn", "transformers", "joblib"]);
    if !ml_imports {
        return Vec::new();
    }

    // model_loaded_per_request / tokenizer_loaded_per_request
    let model_load_patterns = &[
        ("torch.load(", "model_loaded_per_request"),
        ("tf.keras.models.load_model(", "model_loaded_per_request"),
        ("joblib.load(", "model_loaded_per_request"),
        ("AutoModel.from_pretrained(", "model_loaded_per_request"),
        ("pipeline(", "model_loaded_per_request"),
        ("AutoTokenizer.from_pretrained(", "tokenizer_loaded_per_request"),
    ];

    if is_handler_or_view(function) {
        for (pattern, rule_id) in model_load_patterns {
            if body.contains(pattern) {
                if let Some(line) = find_line(body, pattern, function.fingerprint.start_line) {
                    findings.push(make_finding(
                        rule_id, Severity::Warning, file, function, line,
                        "loads model/tokenizer per request; load once at startup",
                    ));
                }
            }
        }
    }

    // Check inside loops too
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() {
            for (pattern, rule_id) in model_load_patterns {
                if trimmed.contains(pattern) {
                    findings.push(make_finding(
                        rule_id, Severity::Warning, file, function,
                        function.fingerprint.start_line + i,
                        "loads model/tokenizer inside a loop; load once and reuse",
                    ));
                }
            }
            // model_to_device_in_loop
            if trimmed.contains(".to(device)") || trimmed.contains(".to(\"cuda\")") || trimmed.contains(".to('cuda')") {
                findings.push(make_finding(
                    "model_to_device_in_loop", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "moves model/tensor to device inside a loop; move once before the loop",
                ));
            }
        }
        if let Some(li) = loop_indent {
            if !trimmed.is_empty() && indent_level(line) <= li && !trimmed.starts_with('#') {
                loop_indent = None;
            }
        }
    }

    // model_eval_mode_missing
    if has_import(file, "torch") && (body.contains("model(") || body.contains("model.forward(")) {
        let has_eval = body.contains("model.eval()") || body.contains(".eval()");
        let has_no_grad = body.contains("torch.no_grad()") || body.contains("torch.inference_mode()");
        if !has_eval && !has_no_grad && !body.contains("model.train()") && !body.contains("optimizer") {
            if let Some(line) = find_line(body, "model(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "model.forward(", function.fingerprint.start_line))
            {
                findings.push(make_finding(
                    "model_eval_mode_missing", Severity::Info, file, function, line,
                    "runs model inference without model.eval() or torch.no_grad()",
                ));
            }
        }
    }

    // torch_no_grad_missing_in_inference
    if has_import(file, "torch")
        && (body.contains("model(") || body.contains("model.forward("))
        && !body.contains("torch.no_grad()") && !body.contains("torch.inference_mode()")
        && !body.contains("optimizer") && !body.contains(".backward()")
    {
        if let Some(line) = find_line(body, "model(", function.fingerprint.start_line) {
            findings.push(make_finding(
                "torch_no_grad_missing_in_inference", Severity::Info, file, function, line,
                "model inference without torch.no_grad(); add context manager to save memory",
            ));
        }
    }

    // training_loop_without_zero_grad
    if has_import(file, "torch") && body.contains("optimizer.step()") && !body.contains("zero_grad()") {
        if let Some(line) = find_line(body, "optimizer.step()", function.fingerprint.start_line) {
            findings.push(make_finding(
                "training_loop_without_zero_grad", Severity::Warning, file, function, line,
                "calls optimizer.step() without zero_grad(); gradients will accumulate",
            ));
        }
    }

    // dataset_not_using_dataloader
    if has_import(file, "torch") && body.contains("for ")
        && !body.contains("DataLoader") && body.contains("dataset")
        && (body.contains("[i:") || body.contains("batch_size"))
    {
        if let Some(line) = find_line(body, "for ", function.fingerprint.start_line) {
            findings.push(make_finding(
                "dataset_not_using_dataloader", Severity::Info, file, function, line,
                "manually batches dataset; use torch.utils.data.DataLoader instead",
            ));
        }
    }

    // embedding_computed_per_request
    if is_handler_or_view(function)
        && (body.contains("model.encode(") || body.contains("Embedding.create(") || body.contains(".embed("))
    {
        if let Some(line) = find_line(body, "encode(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "Embedding.create(", function.fingerprint.start_line))
            .or_else(|| find_line(body, ".embed(", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                "embedding_computed_per_request", Severity::Info, file, function, line,
                "computes embeddings per request; pre-compute and cache for static text",
            ));
        }
    }

    findings
}

// ── LLM / Prompt rules ──────────────────────────────────────────────────────

pub(super) fn llm_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_llm = has_any_import(file, &["openai", "anthropic", "langchain", "litellm", "cohere"]);
    if !has_llm {
        return Vec::new();
    }
    let mut findings = Vec::new();

    let llm_call_patterns = &[
        "ChatCompletion.create(",
        "chat.completions.create(",
        "messages.create(",
        "Completion.create(",
        "client.chat(",
        "llm(",
        "chain(",
    ];

    // llm_api_call_in_loop_without_batching
    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() {
            for pattern in llm_call_patterns {
                if trimmed.contains(pattern) {
                    findings.push(make_finding(
                        "llm_api_call_in_loop_without_batching", Severity::Warning, file, function,
                        function.fingerprint.start_line + i,
                        "calls LLM API inside a loop; batch requests to reduce cost and latency",
                    ));
                    break;
                }
            }
            // prompt_template_string_concat_in_loop
            if (trimmed.contains("prompt +=") || trimmed.contains("prompt = prompt +"))
                && (trimmed.contains('"') || trimmed.contains('\''))
            {
                findings.push(make_finding(
                    "prompt_template_string_concat_in_loop", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "concatenates prompt string inside a loop; build template once",
                ));
            }
        }
        if let Some(li) = loop_indent {
            if !trimmed.is_empty() && indent_level(line) <= li && !trimmed.starts_with('#') {
                loop_indent = None;
            }
        }
    }

    // hardcoded_api_key_in_source
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains("api_key") || trimmed.contains("API_KEY") || trimmed.contains("OPENAI_API_KEY"))
            && trimmed.contains(" = ")
            && (trimmed.contains("\"sk-") || trimmed.contains("'sk-") || trimmed.contains("\"key-") || trimmed.contains("'key-"))
        {
            findings.push(make_finding(
                "hardcoded_api_key_in_source", Severity::Warning, file, function,
                function.fingerprint.start_line + i,
                "hardcodes API key in source; use environment variables or secret management",
            ));
        }
    }

    // retry_on_rate_limit_without_backoff
    if body.contains("retry") || body.contains("RateLimitError") || body.contains("rate_limit") {
        let has_backoff = body.contains("backoff") || body.contains("exponential") || body.contains("Retry-After");
        if !has_backoff && body.contains("sleep(") {
            if let Some(line) = find_line(body, "sleep(", function.fingerprint.start_line) {
                findings.push(make_finding(
                    "retry_on_rate_limit_without_backoff", Severity::Info, file, function, line,
                    "retries without exponential backoff; implement backoff to respect rate limits",
                ));
            }
        }
    }

    // token_count_not_checked_before_api_call
    for pattern in llm_call_patterns {
        if body.contains(pattern)
            && !body.contains("token")
            && !body.contains("tiktoken")
            && !body.contains("count_tokens")
        {
            if let Some(line) = find_line(body, pattern, function.fingerprint.start_line) {
                findings.push(make_finding(
                    "token_count_not_checked_before_api_call", Severity::Info, file, function, line,
                    "sends prompt to LLM without token counting; risk of context window overflow",
                ));
            }
            break;
        }
    }

    findings
}

// ── Data pipeline rules ──────────────────────────────────────────────────────

pub(super) fn data_pipeline_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // random_seed_not_set
    let has_random = has_any_import(file, &["random", "numpy", "torch"]);
    if has_random
        && (body.contains("random.") || body.contains("np.random.") || body.contains("torch.manual_seed"))
        && !body.contains("seed(") && !body.contains("manual_seed(")
        && (function.fingerprint.name.contains("train") || function.fingerprint.name.contains("eval") || function.fingerprint.name == "main")
    {
        findings.push(make_finding(
            "random_seed_not_set", Severity::Info, file, function,
            function.fingerprint.start_line,
            "uses random operations without setting seed; experiments may not be reproducible",
        ));
    }

    // wandb_mlflow_log_in_tight_loop
    let has_tracking = has_any_import(file, &["wandb", "mlflow"]);
    if has_tracking {
        let lines: Vec<&str> = body.lines().collect();
        let mut loop_depth = 0;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("for ") && trimmed.ends_with(':') {
                loop_depth += 1;
            }
            if loop_depth >= 2 {
                if trimmed.contains("wandb.log(") || trimmed.contains("mlflow.log_metric(") {
                    findings.push(make_finding(
                        "wandb_mlflow_log_in_tight_loop", Severity::Info, file, function,
                        function.fingerprint.start_line + i,
                        "logs metrics in inner loop; batch or log at epoch level",
                    ));
                }
            }
        }
    }

    // global_state_in_data_pipeline
    // Detect module-level mutable state modifications
    if (function.fingerprint.name.starts_with("process")
        || function.fingerprint.name.starts_with("transform")
        || function.fingerprint.name.starts_with("load")
        || function.fingerprint.name.starts_with("extract"))
        && body.contains("global ")
    {
        if let Some(line) = find_line(body, "global ", function.fingerprint.start_line) {
            findings.push(make_finding(
                "global_state_in_data_pipeline", Severity::Warning, file, function, line,
                "modifies global state in data pipeline; use function parameters for thread safety",
            ));
        }
    }

    // print_metrics_instead_of_logging
    if has_any_import(file, &["torch", "tensorflow", "sklearn"])
        && (function.fingerprint.name.contains("train") || function.fingerprint.name.contains("eval"))
    {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("print(")
                && (trimmed.contains("accuracy")
                    || trimmed.contains("loss")
                    || trimmed.contains("metric")
                    || trimmed.contains("epoch")
                    || trimmed.contains("f1"))
            {
                findings.push(make_finding(
                    "print_metrics_instead_of_logging", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "prints metrics instead of using logging/experiment tracking framework",
                ));
            }
        }
    }

    findings
}

// ── Remaining Plan 3 Wave 5 rules ────────────────────────────────────────────

pub(super) fn mlops_extra_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    // numpy_dtype_mismatch_implicit_cast
    if has_import(file, "numpy") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("np.array(") && trimmed.contains(".astype(") {
                findings.push(make_finding(
                    "numpy_dtype_mismatch_implicit_cast", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "creates array then immediately casts dtype; specify dtype= in np.array()",
                ));
            }
        }
    }

    // llm_response_not_cached_same_input
    if has_any_import(file, &["openai", "anthropic", "langchain"]) {
        let lines: Vec<&str> = body.lines().collect();
        let mut loop_indent: Option<usize> = None;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("for ") && trimmed.ends_with(':') {
                loop_indent = Some(indent_level(line));
                continue;
            }
            if loop_indent.is_some()
                && (trimmed.contains(".create(") || trimmed.contains(".completions.create("))
                && !body.contains("cache") && !body.contains("lru_cache") && !body.contains("@cache")
            {
                findings.push(make_finding(
                    "llm_response_not_cached_same_input", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "calls LLM API in loop without caching; repeated identical prompts waste tokens",
                ));
            }
            if loop_indent.is_some() && !trimmed.is_empty() && !trimmed.starts_with(' ') && !trimmed.starts_with('\t') && !trimmed.is_empty() && !trimmed.starts_with('#') {
                loop_indent = None;
            }
        }
    }

    // llm_full_response_loaded_into_memory
    if has_any_import(file, &["openai", "anthropic", "langchain"]) {
        if body.contains(".choices") && body.contains("json()") {
            if let Some(line) = find_line(body, "json()", function.fingerprint.start_line) {
                if !body.contains("stream") {
                    findings.push(make_finding(
                        "llm_full_response_loaded_into_memory", Severity::Info, file, function, line,
                        "loads full LLM response into memory; consider streaming for large responses",
                    ));
                }
            }
        }
    }

    // embedding_dimension_mismatch_silent
    if has_any_import(file, &["sentence_transformers", "openai", "torch"]) {
        if body.contains("embedding") && body.contains("cosine_similarity") {
            // Heuristic: using cosine_similarity without shape/dim check
            if !body.contains(".shape") && !body.contains(".size(") && !body.contains("assert") {
                if let Some(line) = find_line(body, "cosine_similarity", function.fingerprint.start_line) {
                    findings.push(make_finding(
                        "embedding_dimension_mismatch_silent", Severity::Info, file, function, line,
                        "compares embeddings without dimension validation; mismatched dims silently produce garbage",
                    ));
                }
            }
        }
    }

    // pandas_read_without_chunksize_large_file
    if has_import(file, "pandas") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.contains("pd.read_csv(") || trimmed.contains("pd.read_parquet(") || trimmed.contains("pd.read_json("))
                && !trimmed.contains("chunksize") && !trimmed.contains("nrows")
            {
                // Check if file path hints at large data
                if function.fingerprint.name.contains("load")
                    || function.fingerprint.name.contains("ingest")
                    || function.fingerprint.name.contains("import")
                    || function.fingerprint.name.contains("process")
                {
                    findings.push(make_finding(
                        "pandas_read_without_chunksize_large_file", Severity::Info, file, function,
                        function.fingerprint.start_line + i,
                        "reads file without chunksize in data-loading function; may OOM on large files",
                    ));
                }
            }
        }
    }

    // entire_dataframe_copied_for_transform
    if has_import(file, "pandas") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains(".copy()") && (trimmed.contains("df") || trimmed.contains("data")) {
                // Check if only a few columns are used afterward
                findings.push(make_finding(
                    "entire_dataframe_copied_for_transform", Severity::Info, file, function,
                    function.fingerprint.start_line + i,
                    "copies entire DataFrame; consider operating on a column subset or using inplace operations",
                ));
            }
        }
    }

    // no_schema_validation_on_external_data
    if (has_any_import(file, &["requests", "aiohttp", "httpx"])
        || has_any_import(file, &["pandas", "json"]))
        && (function.fingerprint.name.contains("fetch")
            || function.fingerprint.name.contains("load")
            || function.fingerprint.name.contains("ingest"))
    {
        let has_validation = body.contains("pydantic")
            || body.contains("marshmallow")
            || body.contains("cerberus")
            || body.contains("jsonschema")
            || body.contains("validate(")
            || body.contains("Schema(");
        if !has_validation && (body.contains(".json()") || body.contains("json.load")) {
            if let Some(line) = find_line(body, ".json()", function.fingerprint.start_line)
                .or_else(|| find_line(body, "json.load", function.fingerprint.start_line))
            {
                findings.push(make_finding(
                    "no_schema_validation_on_external_data", Severity::Info, file, function, line,
                    "parses external data without schema validation; corrupt input propagates silently",
                ));
            }
        }
    }

    // data_pipeline_no_error_handling
    if (function.fingerprint.name.contains("pipeline")
        || function.fingerprint.name.contains("etl")
        || function.fingerprint.name.contains("process"))
        && has_any_import(file, &["pandas", "numpy", "spark"])
    {
        let has_error_handling = body.contains("try:") || body.contains("except ") || body.contains("raise ");
        if !has_error_handling && body.lines().count() > 10 {
            findings.push(make_finding(
                "data_pipeline_no_error_handling", Severity::Info, file, function,
                function.fingerprint.start_line,
                "data pipeline function has no error handling; failures silently corrupt downstream data",
            ));
        }
    }

    // intermediate_dataframe_not_freed
    if has_import(file, "pandas") {
        let assign_count = body.lines().filter(|l| {
            let t = l.trim();
            (t.contains("= pd.") || t.contains("= df.") || t.contains("= data."))
                && t.contains('=')
                && !t.starts_with('#')
        }).count();
        if assign_count >= 4 && !body.contains("del ") && !body.contains("gc.collect") {
            findings.push(make_finding(
                "intermediate_dataframe_not_freed", Severity::Info, file, function,
                function.fingerprint.start_line,
                "creates multiple intermediate DataFrames without freeing; memory builds up",
            ));
        }
    }

    // gpu_memory_not_cleared_between_experiments
    if has_any_import(file, &["torch", "tensorflow"]) {
        let has_cuda_ops = body.contains(".cuda()") || body.contains(".to(device") || body.contains("tf.device");
        let has_cleanup = body.contains("torch.cuda.empty_cache()")
            || body.contains("gc.collect")
            || body.contains("tf.keras.backend.clear_session");
        if has_cuda_ops && !has_cleanup {
            if function.fingerprint.name.contains("train")
                || function.fingerprint.name.contains("experiment")
                || function.fingerprint.name.contains("run")
            {
                if let Some(line) = find_line(body, ".cuda()", function.fingerprint.start_line)
                    .or_else(|| find_line(body, ".to(device", function.fingerprint.start_line))
                    .or_else(|| find_line(body, "tf.device", function.fingerprint.start_line))
                {
                    findings.push(make_finding(
                        "gpu_memory_not_cleared_between_experiments", Severity::Info, file, function, line,
                        "uses GPU without clearing memory; call torch.cuda.empty_cache() between experiments",
                    ));
                }
            }
        }
    }

    findings
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_line(body: &str, needle: &str, base_line: usize) -> Option<usize> {
    for (i, line) in body.lines().enumerate() {
        if line.contains(needle) {
            return Some(base_line + i);
        }
    }
    None
}

fn make_finding(
    rule_id: &str,
    severity: Severity,
    file: &ParsedFile,
    function: &ParsedFunction,
    line: usize,
    msg_suffix: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!("function {} {msg_suffix}", function.fingerprint.name),
        evidence: vec![format!("rule={rule_id}")],
    }
}
