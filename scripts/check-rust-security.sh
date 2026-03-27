#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p reports/rust-security-baseline
report="reports/rust-security-baseline/latest.txt"

run_search() {
  local label="$1"
  local pattern="$2"
  if command -v rg >/dev/null 2>&1; then
    {
      echo "## ${label}"
      rg -n "$pattern" src tests --glob '*.rs' || true
      echo
    } >> "$report"
  else
    {
      echo "## ${label}"
      grep -R -n -E "$pattern" src tests --include='*.rs' || true
      echo
    } >> "$report"
  fi
}

: > "$report"
run_search "narrowing_as_casts" ' as (u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)'
run_search "split_at_and_indexing" '\.split_at\(|\[[^]]+\.\.[^]]*\]|\[[^]]*\.\.[^]]+\]'
run_search "toctou_fs_checks" 'exists\(|metadata\(|symlink_metadata\(|read_link\('
run_search "secret_comparisons" '(password|token|secret|api_key).*(==|!=)'
run_search "shared_mutability" 'Rc<RefCell|Arc<Mutex'
run_search "unsafe_globals" 'static mut|lazy_static!'
run_search "derive_default" 'derive\(.*Default.*\)'
run_search "thread_spawn_async" 'thread::spawn|spawn\(async'
run_search "path_join" '\.join\('

cat "$report"

if [[ "${STRICT:-0}" == "1" ]]; then
  filtered="$(grep -v '^## ' "$report" | grep -v '^$' || true)"
  if [[ -n "$filtered" ]]; then
    echo "rust security scan found matches under STRICT=1" >&2
    exit 1
  fi
fi