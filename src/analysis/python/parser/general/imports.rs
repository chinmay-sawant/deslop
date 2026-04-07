use tree_sitter::Node;

use crate::analysis::ImportSpec;

pub(crate) fn collect_imports(root: Node<'_>, source: &str) -> Vec<ImportSpec> {
    let mut imports = Vec::new();
    visit_imports(root, source, &mut imports);
    imports
}

fn visit_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    if matches!(node.kind(), "import_statement" | "import_from_statement")
        && let Some(text) = source.get(node.byte_range())
    {
        let line = node.start_position().row + 1;
        let new_imports = if node.kind() == "import_statement" {
            parse_import_statement_text(text, line)
        } else {
            parse_import_from_stmt(text, line)
        };
        imports.extend(new_imports);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_imports(child, source, imports);
    }
}

fn parse_import_statement_text(text: &str, line: usize) -> Vec<ImportSpec> {
    let normalized = normalize_import_text(text);
    let entries = normalized
        .strip_prefix("import ")
        .map(split_import_list)
        .unwrap_or_default();

    entries
        .into_iter()
        .map(|entry| {
            let (path, alias) = parse_alias(&entry);
            ImportSpec {
                line,
                group_line: line,
                alias,
                path: path.clone(),
                namespace_path: namespace_path(&path),
                imported_name: imported_name(&path),
                is_public: false,
            }
        })
        .collect()
}

fn parse_import_from_stmt(text: &str, line: usize) -> Vec<ImportSpec> {
    let normalized = normalize_import_text(text);
    let Some(without_prefix) = normalized.strip_prefix("from ") else {
        return Vec::new();
    };
    let Some((module_path, imported_names)) = without_prefix.split_once(" import ") else {
        return Vec::new();
    };

    split_import_list(imported_names)
        .into_iter()
        .map(|entry| {
            let (path, alias) = parse_alias(&entry);
            let full_path = if module_path == "." {
                format!(".{path}")
            } else {
                format!("{module_path}.{path}")
            };
            ImportSpec {
                line,
                group_line: line,
                alias,
                path: full_path,
                namespace_path: Some(module_path.to_string()),
                imported_name: Some(path),
                is_public: false,
            }
        })
        .collect()
}

fn namespace_path(path: &str) -> Option<String> {
    path.rsplit_once('.')
        .map(|(namespace, _)| namespace.to_string())
}

fn imported_name(path: &str) -> Option<String> {
    path.rsplit('.').next().map(str::to_string)
}

fn parse_alias(entry: &str) -> (String, String) {
    let trimmed = entry
        .trim()
        .trim_matches(|character| character == '(' || character == ')');
    if let Some((path, alias)) = trimmed.rsplit_once(" as ") {
        return (path.trim().to_string(), alias.trim().to_string());
    }

    let alias = trimmed
        .rsplit('.')
        .next()
        .unwrap_or(trimmed)
        .trim()
        .to_string();
    (trimmed.to_string(), alias)
}

fn normalize_import_text(text: &str) -> String {
    text.lines()
        .map(strip_python_comment)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .replace("( ", "")
        .replace(" )", "")
}

fn strip_python_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut previous_was_escape = false;

    for (index, character) in line.char_indices() {
        match character {
            '\\' if in_single || in_double => {
                previous_was_escape = !previous_was_escape;
                continue;
            }
            '\'' if !in_double && !previous_was_escape => {
                in_single = !in_single;
            }
            '"' if !in_single && !previous_was_escape => {
                in_double = !in_double;
            }
            '#' if !in_single && !in_double => {
                return &line[..index];
            }
            _ => {}
        }

        if character != '\\' {
            previous_was_escape = false;
        }
    }

    line
}

fn split_import_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}
