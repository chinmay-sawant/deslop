#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

pattern='panic!\(|\.unwrap\(|\.expect\(|fs::read_to_string\('
if command -v rg >/dev/null 2>&1; then
  matches="$(rg -n "$pattern" src --glob '*.rs' --glob '!src/main.rs' --glob '!src/cli/**' --glob '!src/**/tests.rs' || true)"
else
  matches="$(grep -R -n -E "$pattern" src --include='*.rs' | grep -v '^src/main.rs:' | grep -v '^src/cli/' || true)"
fi

status=0
while IFS= read -r match; do
  [[ -z "$match" ]] && continue
  case "$match" in
    src/*/tests.rs:* ) continue ;;
    src/analysis/go/parser/tests.rs:* ) continue ;;
    src/analysis/python/parser/tests.rs:* ) continue ;;
    src/config.rs:* ) continue ;;
    src/io.rs:* ) continue ;;
    src/analysis/mod.rs:* ) continue ;;
    src/analysis/rust/parser.rs:* ) continue ;;
    src/index/mod.rs:* ) continue ;;
    src/scan/mod.rs:* ) continue ;;
  esac
  printf '%s\n' "$match"
  status=1
done <<< "$matches"

if [[ "$status" -ne 0 ]]; then
  echo "forbidden Rust hygiene patterns remain in production source" >&2
  exit 1
fi