use super::*;

pub(crate) fn llm_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_llm = has_any_import(
        file,
        &["openai", "anthropic", "langchain", "litellm", "cohere"],
    );
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
                        "llm_api_call_in_loop_without_batching",
                        Severity::Warning,
                        file,
                        function,
                        function.fingerprint.start_line + i,
                        "calls LLM API inside a loop; batch requests to reduce cost and latency",
                    ));
                    break;
                }
            }
            if (trimmed.contains("prompt +=") || trimmed.contains("prompt = prompt +"))
                && (trimmed.contains('"') || trimmed.contains('\''))
            {
                findings.push(make_finding(
                    "prompt_template_string_concat_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "concatenates prompt string inside a loop; build template once",
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

    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains("api_key")
            || trimmed.contains("API_KEY")
            || trimmed.contains("OPENAI_API_KEY"))
            && trimmed.contains(" = ")
            && (trimmed.contains("\"sk-")
                || trimmed.contains("'sk-")
                || trimmed.contains("\"key-")
                || trimmed.contains("'key-"))
        {
            findings.push(make_finding(
                "hardcoded_api_key_in_source",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line + i,
                "hardcodes API key in source; use environment variables or secret management",
            ));
        }
    }

    if body.contains("retry") || body.contains("RateLimitError") || body.contains("rate_limit") {
        let has_backoff = body.contains("backoff")
            || body.contains("exponential")
            || body.contains("Retry-After");
        if !has_backoff
            && body.contains("sleep(")
            && let Some(line) = find_line(body, "sleep(", function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "retry_on_rate_limit_without_backoff",
                Severity::Info,
                file,
                function,
                line,
                "retries without exponential backoff; implement backoff to respect rate limits",
            ));
        }
    }

    for pattern in llm_call_patterns {
        if body.contains(pattern)
            && !body.contains("token")
            && !body.contains("tiktoken")
            && !body.contains("count_tokens")
        {
            if let Some(line) = find_line(body, pattern, function.fingerprint.start_line) {
                findings.push(make_finding(
                    "token_count_not_checked_before_api_call",
                    Severity::Info,
                    file,
                    function,
                    line,
                    "sends prompt to LLM without token counting; risk of context window overflow",
                ));
            }
            break;
        }
    }

    if is_handler_or_view(function)
        && has_any_import(
            file,
            &[
                "qdrant_client",
                "chromadb",
                "pinecone",
                "weaviate",
                "faiss",
                "lancedb",
                "milvus",
            ],
        )
    {
        for pattern in [
            "QdrantClient(",
            "PersistentClient(",
            "chromadb.Client(",
            "Pinecone(",
            "weaviate.Client(",
            "faiss.Index",
            "lancedb.connect(",
            "MilvusClient(",
        ] {
            if let Some(line) = find_line(body, pattern, function.fingerprint.start_line) {
                findings.push(make_finding(
                    "vector_store_client_created_per_request",
                    Severity::Warning,
                    file,
                    function,
                    line,
                    "creates a vector-store client on a request path; reuse the client across requests",
                ));
                break;
            }
        }
    }

    if is_handler_or_view(function)
        && has_any_import(file, &["langchain", "langchain_core", "llama_index"])
    {
        for pattern in [
            "LLMChain(",
            "RetrievalQA.from_chain_type(",
            "ChatPromptTemplate.from_template(",
            "PromptTemplate.from_template(",
            "VectorStoreIndex.from_documents(",
            "ServiceContext.from_defaults(",
        ] {
            if let Some(line) = find_line(body, pattern, function.fingerprint.start_line) {
                findings.push(make_finding(
                    "langchain_chain_built_per_request",
                    Severity::Info,
                    file,
                    function,
                    line,
                    "builds a LangChain/LlamaIndex-style chain on a request path; cache reusable prompt and chain wiring",
                ));
                break;
            }
        }
    }

    if has_any_import(file, &["transformers", "tokenizers", "tiktoken"]) {
        let lines: Vec<&str> = body.lines().collect();
        let mut loop_indent: Option<usize> = None;
        let has_cache_signal = body.contains("cache")
            || body.contains("lru_cache")
            || body.contains("@cache")
            || body.contains("batch_encode_plus(")
            || body.contains("encode_batch(");

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("for ") && trimmed.ends_with(':') {
                loop_indent = Some(indent_level(line));
                continue;
            }
            if loop_indent.is_some()
                && !has_cache_signal
                && (trimmed.contains("tokenizer.encode(")
                    || trimmed.contains("encoding.encode(")
                    || trimmed.contains(".encode(") && trimmed.contains("token"))
            {
                findings.push(make_finding(
                    "tokenizer_encode_in_loop_without_cache",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "encodes tokens inside a loop without caching or batching; reuse token counts or batch the call",
                ));
                break;
            }
            if let Some(li) = loop_indent
                && !trimmed.is_empty()
                && indent_level(line) <= li
                && !trimmed.starts_with('#')
            {
                loop_indent = None;
            }
        }
    }

    findings
}
