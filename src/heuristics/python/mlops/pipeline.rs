use super::*;

pub(crate) fn data_pipeline_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    let has_random = has_any_import(file, &["random", "numpy", "torch"]);
    if has_random
        && (body.contains("random.")
            || body.contains("np.random.")
            || body.contains("torch.manual_seed"))
        && !body.contains("seed(")
        && !body.contains("manual_seed(")
        && (function.fingerprint.name.contains("train")
            || function.fingerprint.name.contains("eval")
            || function.fingerprint.name == "main")
    {
        findings.push(make_finding(
            "random_seed_not_set",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "uses random operations without setting seed; experiments may not be reproducible",
        ));
    }

    let has_tracking = has_any_import(file, &["wandb", "mlflow"]);
    if has_tracking {
        let lines: Vec<&str> = body.lines().collect();
        let mut loop_depth = 0;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("for ") && trimmed.ends_with(':') {
                loop_depth += 1;
            }
            if loop_depth >= 2
                && (trimmed.contains("wandb.log(") || trimmed.contains("mlflow.log_metric("))
            {
                findings.push(make_finding(
                    "wandb_mlflow_log_in_tight_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "logs metrics in inner loop; batch or log at epoch level",
                ));
            }
        }
    }

    if (function.fingerprint.name.starts_with("process")
        || function.fingerprint.name.starts_with("transform")
        || function.fingerprint.name.starts_with("load")
        || function.fingerprint.name.starts_with("extract"))
        && body.contains("global ")
        && let Some(line) = find_line(body, "global ", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "global_state_in_data_pipeline",
            Severity::Warning,
            file,
            function,
            line,
            "modifies global state in data pipeline; use function parameters for thread safety",
        ));
    }

    if has_any_import(file, &["torch", "tensorflow", "sklearn"])
        && (function.fingerprint.name.contains("train")
            || function.fingerprint.name.contains("eval"))
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
                    "print_metrics_instead_of_logging",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "prints metrics instead of using logging/experiment tracking framework",
                ));
            }
        }
    }

    findings
}

pub(crate) fn mlops_extra_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    if has_import(file, "numpy") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("np.array(") && trimmed.contains(".astype(") {
                findings.push(make_finding(
                    "numpy_dtype_mismatch_implicit_cast",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "creates array then immediately casts dtype; specify dtype= in np.array()",
                ));
            }
        }
    }

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
                && !body.contains("cache")
                && !body.contains("lru_cache")
                && !body.contains("@cache")
            {
                findings.push(make_finding(
                    "llm_response_not_cached_same_input",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "calls LLM API in loop without caching; repeated identical prompts waste tokens",
                ));
            }
            if loop_indent.is_some()
                && !trimmed.is_empty()
                && !trimmed.starts_with(' ')
                && !trimmed.starts_with('\t')
                && !trimmed.starts_with('#')
            {
                loop_indent = None;
            }
        }
    }

    if has_any_import(file, &["openai", "anthropic", "langchain"])
        && body.contains(".choices")
        && body.contains("json()")
        && let Some(line) = find_line(body, "json()", function.fingerprint.start_line)
        && !body.contains("stream")
    {
        findings.push(make_finding(
            "llm_full_response_loaded_into_memory",
            Severity::Info,
            file,
            function,
            line,
            "loads full LLM response into memory; consider streaming for large responses",
        ));
    }

    if has_any_import(file, &["sentence_transformers", "openai", "torch"])
        && body.contains("embedding")
        && body.contains("cosine_similarity")
        && !body.contains(".shape")
        && !body.contains(".size(")
        && !body.contains("assert")
        && let Some(line) = find_line(body, "cosine_similarity", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "embedding_dimension_mismatch_silent",
            Severity::Info,
            file,
            function,
            line,
            "compares embeddings without dimension validation; mismatched dims silently produce garbage",
        ));
    }

    if has_import(file, "pandas") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.contains("pd.read_csv(")
                || trimmed.contains("pd.read_parquet(")
                || trimmed.contains("pd.read_json("))
                && !trimmed.contains("chunksize")
                && !trimmed.contains("nrows")
                && (function.fingerprint.name.contains("load")
                    || function.fingerprint.name.contains("ingest")
                    || function.fingerprint.name.contains("import")
                    || function.fingerprint.name.contains("process"))
            {
                findings.push(make_finding(
                    "pandas_read_without_chunksize_large_file",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "reads file without chunksize in data-loading function; may OOM on large files",
                ));
            }
        }
    }

    if has_import(file, "pandas") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains(".copy()") && (trimmed.contains("df") || trimmed.contains("data")) {
                findings.push(make_finding(
                    "entire_dataframe_copied_for_transform",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "copies entire DataFrame; consider operating on a column subset or using inplace operations",
                ));
            }
        }
    }

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
        if !has_validation
            && (body.contains(".json()") || body.contains("json.load"))
            && let Some(line) = find_line(body, ".json()", function.fingerprint.start_line)
                .or_else(|| find_line(body, "json.load", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                "no_schema_validation_on_external_data",
                Severity::Info,
                file,
                function,
                line,
                "parses external data without schema validation; corrupt input propagates silently",
            ));
        }
    }

    if (function.fingerprint.name.contains("pipeline")
        || function.fingerprint.name.contains("etl")
        || function.fingerprint.name.contains("process"))
        && has_any_import(file, &["pandas", "numpy", "spark"])
    {
        let has_error_handling =
            body.contains("try:") || body.contains("except ") || body.contains("raise ");
        if !has_error_handling && body.lines().count() > 10 {
            findings.push(make_finding(
                "data_pipeline_no_error_handling",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line,
                "data pipeline function has no error handling; failures silently corrupt downstream data",
            ));
        }
    }

    if has_import(file, "pandas") {
        let assign_count = body
            .lines()
            .filter(|l| {
                let t = l.trim();
                (t.contains("= pd.") || t.contains("= df.") || t.contains("= data."))
                    && t.contains('=')
                    && !t.starts_with('#')
            })
            .count();
        if assign_count >= 4 && !body.contains("del ") && !body.contains("gc.collect") {
            findings.push(make_finding(
                "intermediate_dataframe_not_freed",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line,
                "creates multiple intermediate DataFrames without freeing; memory builds up",
            ));
        }
    }

    if has_any_import(file, &["torch", "tensorflow"]) {
        let has_cuda_ops =
            body.contains(".cuda()") || body.contains(".to(device") || body.contains("tf.device");
        let has_cleanup = body.contains("torch.cuda.empty_cache()")
            || body.contains("gc.collect")
            || body.contains("tf.keras.backend.clear_session");
        if has_cuda_ops
            && !has_cleanup
            && (function.fingerprint.name.contains("train")
                || function.fingerprint.name.contains("experiment")
                || function.fingerprint.name.contains("run"))
            && let Some(line) = find_line(body, ".cuda()", function.fingerprint.start_line)
                .or_else(|| find_line(body, ".to(device", function.fingerprint.start_line))
                .or_else(|| find_line(body, "tf.device", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                "gpu_memory_not_cleared_between_experiments",
                Severity::Info,
                file,
                function,
                line,
                "uses GPU without clearing memory; call torch.cuda.empty_cache() between experiments",
            ));
        }
    }

    findings
}
