use super::*;

pub(crate) fn model_inference_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    let ml_imports = has_any_import(
        file,
        &[
            "torch",
            "tensorflow",
            "tf",
            "sklearn",
            "transformers",
            "joblib",
        ],
    );
    if !ml_imports {
        return Vec::new();
    }

    let model_load_patterns = &[
        ("torch.load(", "model_loaded_per_request"),
        ("tf.keras.models.load_model(", "model_loaded_per_request"),
        ("joblib.load(", "model_loaded_per_request"),
        ("AutoModel.from_pretrained(", "model_loaded_per_request"),
        ("pipeline(", "model_loaded_per_request"),
        (
            "AutoTokenizer.from_pretrained(",
            "tokenizer_loaded_per_request",
        ),
    ];

    if is_handler_or_view(function) {
        for (pattern, rule_id) in model_load_patterns {
            if body.contains(pattern)
                && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
            {
                findings.push(make_finding(
                    rule_id,
                    Severity::Warning,
                    file,
                    function,
                    line,
                    "loads model/tokenizer per request; load once at startup",
                ));
            }
        }
    }

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
                        rule_id,
                        Severity::Warning,
                        file,
                        function,
                        function.fingerprint.start_line + i,
                        "loads model/tokenizer inside a loop; load once and reuse",
                    ));
                }
            }
            if trimmed.contains(".to(device)")
                || trimmed.contains(".to(\"cuda\")")
                || trimmed.contains(".to('cuda')")
            {
                findings.push(make_finding(
                    "model_to_device_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "moves model/tensor to device inside a loop; move once before the loop",
                ));
            }
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }

    if has_import(file, "torch") && (body.contains("model(") || body.contains("model.forward(")) {
        let has_eval = body.contains("model.eval()") || body.contains(".eval()");
        let has_no_grad =
            body.contains("torch.no_grad()") || body.contains("torch.inference_mode()");
        if !has_eval
            && !has_no_grad
            && !body.contains("model.train()")
            && !body.contains("optimizer")
            && let Some(line) = find_line(body, "model(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "model.forward(", function.fingerprint.start_line))
        {
            findings.push(make_finding(
                "model_eval_mode_missing",
                Severity::Info,
                file,
                function,
                line,
                "runs model inference without model.eval() or torch.no_grad()",
            ));
        }
    }

    if has_import(file, "torch")
        && (body.contains("model(") || body.contains("model.forward("))
        && !body.contains("torch.no_grad()")
        && !body.contains("torch.inference_mode()")
        && !body.contains("optimizer")
        && !body.contains(".backward()")
        && let Some(line) = find_line(body, "model(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "torch_no_grad_missing_in_inference",
            Severity::Info,
            file,
            function,
            line,
            "model inference without torch.no_grad(); add context manager to save memory",
        ));
    }

    if has_import(file, "torch")
        && body.contains("optimizer.step()")
        && !body.contains("zero_grad()")
        && let Some(line) = find_line(body, "optimizer.step()", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "training_loop_without_zero_grad",
            Severity::Warning,
            file,
            function,
            line,
            "calls optimizer.step() without zero_grad(); gradients will accumulate",
        ));
    }

    if has_import(file, "torch")
        && body.contains("for ")
        && !body.contains("DataLoader")
        && body.contains("dataset")
        && (body.contains("[i:") || body.contains("batch_size"))
        && let Some(line) = find_line(body, "for ", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "dataset_not_using_dataloader",
            Severity::Info,
            file,
            function,
            line,
            "manually batches dataset; use torch.utils.data.DataLoader instead",
        ));
    }

    if is_handler_or_view(function)
        && (body.contains("model.encode(")
            || body.contains("Embedding.create(")
            || body.contains(".embed("))
        && let Some(line) = find_line(body, "encode(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "Embedding.create(", function.fingerprint.start_line))
            .or_else(|| find_line(body, ".embed(", function.fingerprint.start_line))
    {
        findings.push(make_finding(
            "embedding_computed_per_request",
            Severity::Info,
            file,
            function,
            line,
            "computes embeddings per request; pre-compute and cache for static text",
        ));
    }

    findings
}
