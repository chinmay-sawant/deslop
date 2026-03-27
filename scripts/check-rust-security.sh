#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p reports/rust-security-baseline
report="reports/rust-security-baseline/latest.txt"

rust_files() {
  if command -v rg >/dev/null 2>&1; then
    rg --files src tests -g '*.rs'
  else
    find src tests -type f -name '*.rs'
  fi
}

append_section() {
  local label="$1"
  echo "## ${label}" >> "$report"
}

run_search() {
  local label="$1"
  local pattern="$2"
  if command -v rg >/dev/null 2>&1; then
    {
      append_section "$label"
      rg -n "$pattern" src tests --glob '*.rs' || true
      echo
    } >> "$report"
  else
    {
      append_section "$label"
      grep -R -n -E "$pattern" src tests --include='*.rs' || true
      echo
    } >> "$report"
  fi
}

run_pcre_search() {
  local label="$1"
  local pattern="$2"
  if command -v rg >/dev/null 2>&1; then
    {
      append_section "$label"
      rg -n -P "$pattern" src tests --glob '*.rs' || true
      echo
    } >> "$report"
  else
    run_search "$label" "$pattern"
  fi
}

run_proximity_search() {
  local label="$1"
  append_section "$label"
  while IFS= read -r file; do
    awk -v file="$file" '
      /exists\(|metadata\(|symlink_metadata\(|read_link\(/ {
        last_check_line = NR;
        last_check_text = $0;
      }
      /(File::open\(|OpenOptions::new\(|\.open\()/ {
        if (last_check_line && NR - last_check_line <= 8) {
          printf "%s:%d: check-then-open candidate after line %d: %s\n", file, NR, last_check_line, $0;
          printf "%s:%d: prior check: %s\n", file, last_check_line, last_check_text;
          last_check_line = 0;
          last_check_text = "";
        }
      }
    ' "$file" >> "$report"
  done < <(rust_files)
  echo >> "$report"
}

: > "$report"
run_search "narrowing_as_casts" ' as (u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)'
run_search "split_at_and_indexing" '\b[A-Za-z_][A-Za-z0-9_]*\.split_at\(|\[[A-Za-z_][^]]*\.\.[^]]*\]|\[[^]]*\.\.[A-Za-z_][^]]+\]'
run_search "toctou_fs_checks" 'exists\(|metadata\(|symlink_metadata\(|read_link\('
run_proximity_search "toctou_check_then_open"
run_pcre_search "secret_comparisons" '(?i)(?:\b(?:if|assert(?:_eq|_ne)?|while|return|match)\b[^\n]*(?:password|secret|token|api_key|access_token|private_key)[^\n]*(?:==|!=)|(?:==|!=)[^\n]*(?:password|secret|token|api_key|access_token|private_key))'
run_search "shared_mutability" 'Rc<\s*RefCell|RefCell<|Rc<'
run_search "unsafe_globals" 'static mut|lazy_static!'
run_search "derive_default" 'derive\(.*Default.*\)'
run_search "thread_spawn_async" 'thread::spawn|spawn\(async'
run_pcre_search "path_join_absolute" "(?<!contains\()\\.join\\((\"/|'/)"

cat "$report"

if [[ "${STRICT:-0}" == "1" ]]; then
  filtered="$(grep -v '^## ' "$report" | grep -v '^$' || true)"
  if [[ -n "$filtered" ]]; then
    echo "rust security scan found matches under STRICT=1" >&2
    exit 1
  fi
fi