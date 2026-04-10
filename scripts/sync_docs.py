#!/usr/bin/env python3
"""Synchronize machine-managed docs content from the central rule registry."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GUIDES_PATH = ROOT / "guides"
README_PATH = ROOT / "README.md"
DOCS_CONTENT_PATH = ROOT / "frontend" / "src" / "features" / "docs" / "docs-content.ts"
FRONTEND_RULES_GENERATED_DIR = (
    ROOT / "frontend" / "src" / "features" / "docs" / "generated"
)
FRONTEND_RULE_MANIFEST_PATH = FRONTEND_RULES_GENERATED_DIR / "rule-manifest.json"
FRONTEND_RULE_CHUNKS_DIR = FRONTEND_RULES_GENERATED_DIR / "rules"
ACTION_PATH = ROOT / "action.yml"
CARGO_TOML_PATH = ROOT / "Cargo.toml"
RULES_REGISTRY_PATH = ROOT / "rules" / "registry.json"

LANGUAGE_ORDER = ["common", "go", "python", "rust"]
STATUS_ORDER = ["stable", "experimental", "research"]
README_ACTION_REF = "chinmay-sawant/deslop@v{version}"
FRONTEND_ACTION_REF = "${currentRelease.actionRef}"

FAMILY_METADATA: dict[tuple[str, str], dict[str, str]] = {
    ("common", "comments"): {
        "label": "Comments",
        "summary": "Generated-looking or tutorial-style commentary that adds noise instead of signal.",
        "why": "Comment rules matter because comments are supposed to compress understanding. When they only narrate obvious code, they slow review without adding intent.",
        "fix": "Keep comments for invariants, tradeoffs, or surprising behavior. Delete commentary that only restates what the code already says.",
    },
    ("common", "hallucination"): {
        "label": "Hallucination",
        "summary": "Calls that look plausible but do not resolve against repository-local symbols.",
        "why": "Hallucination rules matter because plausible-looking references to symbols the repository cannot resolve often ship broken or misleading code.",
        "fix": "Prefer symbols that are actually imported or declared in the local repository context, and verify generated calls against the codebase before merging.",
    },
    ("common", "naming"): {
        "label": "Naming",
        "summary": "Weak naming and type-shape signals that hide intent.",
        "why": "Naming rules matter because vague names and weak type contracts make intent harder to recover during review and maintenance.",
        "fix": "Use domain nouns, shorter purpose-driven names, and stronger types so the contract is visible without extra detective work.",
    },
    ("common", "security"): {
        "label": "Security",
        "summary": "Shared secret-handling and boundary checks that apply across languages.",
        "why": "Shared security rules matter because unsafe defaults, leaked secrets, and boundary mistakes can become real incidents even in otherwise small changes.",
        "fix": "Move secrets and security-sensitive decisions behind explicit secure APIs, validated config, or centralized guardrails.",
    },
    ("common", "test_quality"): {
        "label": "Test Quality",
        "summary": "Tests that read like placeholders or never prove failure behavior.",
        "why": "Test-quality rules matter because a green test suite is not useful if the tests never prove that the risky behavior can fail correctly.",
        "fix": "Add assertions with real expectations, cover negative paths, and remove placeholder tests that only gesture at safety.",
    },
    ("go", "architecture"): {
        "label": "Architecture",
        "summary": "Layering, DTO ownership, transactions, startup flow, handlers, services, and repositories.",
        "why": "Go architecture rules matter because boundary drift and mixed responsibilities make handlers, services, and repositories harder to change safely.",
        "fix": "Keep each layer focused on one job, push business logic out of transport code, and make ownership of transactions, mapping, and lifecycle explicit.",
    },
    ("go", "concurrency"): {
        "label": "Concurrency",
        "summary": "Channels, goroutines, shutdown paths, and coordination hazards.",
        "why": "Concurrency rules matter because lifecycle mistakes around goroutines and channels often stay invisible until the service is under load.",
        "fix": "Give concurrent work a clear owner, cancellation path, and close/stop protocol instead of relying on ambient behavior.",
    },
    ("go", "consistency"): {
        "label": "Consistency",
        "summary": "Project-wide naming and structural consistency checks.",
        "why": "Consistency rules matter because repeated shape drift across a codebase makes the same concept look different in every file.",
        "fix": "Centralize the shared pattern once, then reuse the same structure and naming everywhere that concept appears.",
    },
    ("go", "context"): {
        "label": "Context",
        "summary": "Context propagation, cancellation, detaching, and request-lifetime misuse.",
        "why": "Context rules matter because request-scoped cancellation and deadlines are easy to lose accidentally, which creates hidden leaks and shutdown bugs.",
        "fix": "Pass the correct context explicitly, detach only with intent, and avoid wrapping or replacing context values without a clear boundary reason.",
    },
    ("go", "data_access"): {
        "label": "Data Access",
        "summary": "Query shape, materialization, batching, and database access costs.",
        "why": "Data-access rules matter because query shape and result handling choices can quietly dominate latency, allocation, and database load.",
        "fix": "Keep query ownership inside the repository layer, bound result sizes, and avoid work that scales with rows when one-time setup would do.",
    },
    ("go", "errors"): {
        "label": "Errors",
        "summary": "Error-flow handling patterns that hide real failure behavior.",
        "why": "Error rules matter because flattened or inconsistent error handling makes operations fail in ways that are harder to classify, retry, or explain.",
        "fix": "Preserve the important error signal, map it once at the right boundary, and avoid dropping detail that callers still need.",
    },
    ("go", "gin"): {
        "label": "Gin",
        "summary": "Gin-specific routing, binding, handlers, and transport boundary issues.",
        "why": "Gin rules matter because request-path code is easy to overload with parsing, validation, persistence, and business logic all at once.",
        "fix": "Keep handlers narrow, normalize request data early, and delegate real business work to services or repositories with transport-neutral contracts.",
    },
    ("go", "hot_path"): {
        "label": "Hot Path",
        "summary": "High-frequency request-path work that repeats avoidable computation.",
        "why": "Hot-path rules matter because tiny costs inside frequently executed code paths turn into real CPU and allocation bills at traffic scale.",
        "fix": "Move invariant work out of the hot path, reuse buffers or parsed state, and only do per-request work that must actually happen there.",
    },
    ("go", "idioms"): {
        "label": "Idioms",
        "summary": "Go-specific API and language-idiom smells.",
        "why": "Idiom rules matter because code that fights the language is usually harder to read, maintain, and optimize than code that follows the normal Go shape.",
        "fix": "Lean on the standard Go conventions for names, ownership, and contracts so readers do not have to decode a custom style first.",
    },
    ("go", "library"): {
        "label": "Library",
        "summary": "Misuse of standard or third-party library surfaces.",
        "why": "Library rules matter because small misuse of a library API often compiles fine while still paying unnecessary correctness or performance costs.",
        "fix": "Use the narrowest library API that matches the job, and keep library-specific behavior behind a focused adapter when it starts leaking everywhere.",
    },
    ("go", "mod"): {
        "label": "Module Layout",
        "summary": "Package and module structure checks for repository organization.",
        "why": "Module-layout rules matter because entrypoints and package boundaries are part of the public maintenance story of the repository.",
        "fix": "Keep package roles obvious, and place startup or entrypoint code where maintainers expect to find it.",
    },
    ("go", "performance"): {
        "label": "Performance",
        "summary": "Low-level CPU, allocation, and repeated-work performance smells.",
        "why": "Performance rules matter because repeated work inside loops and request paths compounds quickly, even when the local code change looks harmless.",
        "fix": "Hoist invariant work, reuse allocations, and choose data movement patterns that scale linearly instead of paying the same cost again and again.",
    },
    ("go", "security"): {
        "label": "Security",
        "summary": "Secrets, crypto, injection, response, and exposure patterns.",
        "why": "Security rules matter because transport and persistence code often hide the exact boundary where data becomes dangerous or sensitive.",
        "fix": "Use explicit safe APIs, validate untrusted input close to the edge, and keep security-sensitive behavior in reviewed shared helpers.",
    },
    ("go", "style"): {
        "label": "Style",
        "summary": "Formatting and import-grouping hygiene checks.",
        "why": "Style rules matter because low-friction structural cleanup keeps the rest of the codebase easier to scan and maintain.",
        "fix": "Prefer the standard shape the rest of the repository already follows, especially for imports and file-level organization.",
    },
    ("python", "ai_smells"): {
        "label": "AI Smells",
        "summary": "Generated-looking helper, naming, and commentary patterns.",
        "why": "AI-smell rules matter because code can look polished while still carrying the vague naming and filler structure of low-context generation.",
        "fix": "Rename helpers around the real domain job, collapse noise, and keep only the structure that pays for itself in maintenance.",
    },
    ("python", "architecture"): {
        "label": "Architecture",
        "summary": "Repository, service, DTO, and package ownership drift.",
        "why": "Python architecture rules matter because soft module boundaries make it easy for transport, persistence, and business logic to blur together over time.",
        "fix": "Choose one owner for mapping, orchestration, and data access, then keep each package aligned with that role.",
    },
    ("python", "boundaries"): {
        "label": "Boundaries",
        "summary": "Boundary hygiene for network, files, config, resources, and security edges.",
        "why": "Boundary rules matter because the sharp edges of a system usually live where code touches files, the network, config, or external resources.",
        "fix": "Normalize inputs at the boundary, centralize sensitive resource handling, and keep external effects behind deliberate interfaces.",
    },
    ("python", "discipline"): {
        "label": "Discipline",
        "summary": "Error handling, typing, and testing discipline.",
        "why": "Discipline rules matter because permissive Python code can keep moving while quietly dropping guarantees about types, failures, and tests.",
        "fix": "Tighten contracts, preserve error intent, and make tests prove both the success path and the failure behavior that matters.",
    },
    ("python", "duplication"): {
        "label": "Duplication",
        "summary": "Copy-paste and repeated structure across modules or functions.",
        "why": "Duplication rules matter because repeated logic diverges quickly and turns simple bug fixes into hunt-the-copy maintenance work.",
        "fix": "Extract the stable shared behavior once, but only after the duplication clearly represents the same concept.",
    },
    ("python", "framework"): {
        "label": "Framework",
        "summary": "Web and framework-specific boundary and request-path smells.",
        "why": "Framework rules matter because view, route, and serializer code tends to collect too many concerns when the boundary is not policed.",
        "fix": "Keep framework objects close to the edge and hand off domain work to framework-neutral helpers or services quickly.",
    },
    ("python", "hot_path"): {
        "label": "Hot Path",
        "summary": "Hot-path loops and repeated work in frequently executed code.",
        "why": "Hot-path rules matter because repeated work in Python gets expensive fast when the path runs for every request, batch, or record.",
        "fix": "Precompute what you can once, reuse objects or compiled state, and avoid rebuilding the same work inside the loop.",
    },
    ("python", "hot_path_ext"): {
        "label": "Hot Path Extended",
        "summary": "Expanded hot-path checks for profiling-sensitive waste.",
        "why": "Extended hot-path rules matter because profiling usually shows the same avoidable patterns repeating in slightly different forms.",
        "fix": "Treat frequently executed code as a budget: keep the inner loop lean and move expensive setup or conversion work outward.",
    },
    ("python", "maintainability"): {
        "label": "Maintainability",
        "summary": "Commented-out code, sync/async confusion, and other upkeep signals.",
        "why": "Maintainability rules matter because cleanup debt compounds until every file carries a little uncertainty about what is still real.",
        "fix": "Delete dead code, keep async and sync boundaries explicit, and reduce shapes that future maintainers have to second-guess.",
    },
    ("python", "mlops"): {
        "label": "MLOps",
        "summary": "Pipeline, dataset, prompt, and inference workflow smells.",
        "why": "MLOps rules matter because data pipelines and model-serving paths amplify waste, leakage, and reproducibility problems quickly.",
        "fix": "Make pipeline ownership explicit, keep expensive data work deliberate, and favor stable prompt or model contracts over ad hoc assembly.",
    },
    ("python", "observability"): {
        "label": "Observability",
        "summary": "Logging, metrics, module design, and operational visibility issues.",
        "why": "Observability rules matter because weak logging and module boundaries make production behavior harder to reason about under pressure.",
        "fix": "Log once with useful fields, keep observation helpers shared, and avoid scattering module-level operational policy across the repo.",
    },
    ("python", "packaging"): {
        "label": "Packaging",
        "summary": "Entrypoints, package layout, and import-surface hygiene.",
        "why": "Packaging rules matter because import surfaces and entrypoints shape how predictable the repository feels to users and maintainers.",
        "fix": "Keep package roles obvious, avoid surprising side effects on import, and put entrypoint code where tooling and humans expect it.",
    },
    ("python", "performance"): {
        "label": "Performance",
        "summary": "General performance patterns outside the hot-path packs.",
        "why": "Performance rules matter because a few avoidable data-shape or allocation choices can dominate runtime costs in Python.",
        "fix": "Choose the cheaper structure early, avoid repeated conversions, and reserve broad materialization for places where it is actually needed.",
    },
    ("python", "quality"): {
        "label": "Quality",
        "summary": "Baseline code-quality rules for reliability and clarity.",
        "why": "Quality rules matter because small structural shortcuts often make the code look finished before it is actually review-safe.",
        "fix": "Prefer explicit intent, clearer failure handling, and one obvious place for each responsibility.",
    },
    ("python", "structure"): {
        "label": "Structure",
        "summary": "Classes, functions, state, and responsibility boundaries.",
        "why": "Structure rules matter because oversized classes and mixed-concern functions hide the real domain seams of the codebase.",
        "fix": "Split responsibilities around the real domain boundary, and keep stateful objects narrow enough that their role is obvious.",
    },
    ("rust", "api_design"): {
        "label": "API Design",
        "summary": "API shapes, contracts, and surface-area design smells.",
        "why": "Rust API-design rules matter because good ownership and type contracts are a large part of what keeps Rust code ergonomic and safe.",
        "fix": "Use the type system to make the contract obvious, and avoid APIs that force callers to remember hidden modes or invalid states.",
    },
    ("rust", "async_patterns"): {
        "label": "Async Patterns",
        "summary": "Async/runtime usage patterns that create hidden hazards.",
        "why": "Rust async rules matter because runtime boundaries and task ownership mistakes can stay subtle until production concurrency appears.",
        "fix": "Make task ownership, cancellation, and runtime assumptions explicit instead of relying on implicit background behavior.",
    },
    ("rust", "boundary"): {
        "label": "Boundary",
        "summary": "Cross-module and crate-boundary ownership issues.",
        "why": "Boundary rules matter because crate and module edges are where Rust projects either clarify ownership or hide it.",
        "fix": "Keep boundary contracts explicit and avoid leaking implementation details across module lines without a strong reason.",
    },
    ("rust", "domain_modeling"): {
        "label": "Domain Modeling",
        "summary": "Types and domain objects that hide business intent or invalid states.",
        "why": "Domain-modeling rules matter because Rust gives you strong tools to encode intent, and weak modeling throws that advantage away.",
        "fix": "Model the real business state directly in the types, and keep impossible or unsupported states out of normal flows.",
    },
    ("rust", "hygiene"): {
        "label": "Hygiene",
        "summary": "Leftovers, placeholder code, and repository-local hygiene checks.",
        "why": "Hygiene rules matter because generated leftovers and placeholder shapes make the codebase look more certain than it really is.",
        "fix": "Delete dead scaffolding, keep comments honest, and make sure the local crate references actually resolve the way the code claims.",
    },
    ("rust", "module_surface"): {
        "label": "Module Surface",
        "summary": "Module exports, placement, and surface-area drift.",
        "why": "Module-surface rules matter because every extra public edge expands what future changes have to preserve.",
        "fix": "Expose the smallest surface that supports the use case, and keep module roles obvious from their location and exports.",
    },
    ("rust", "performance"): {
        "label": "Performance",
        "summary": "Repeated work, allocation churn, and avoidable hot-path waste.",
        "why": "Rust performance rules matter because efficient code is often the result of avoiding small repeated costs before they become default structure.",
        "fix": "Reuse allocations, hoist invariant work, and keep the hot path doing only the work that truly belongs there.",
    },
    ("rust", "runtime_boundary"): {
        "label": "Runtime Boundary",
        "summary": "Runtime-layer ownership and boundary mismatches.",
        "why": "Runtime-boundary rules matter because async executors, blocking work, and runtime-owned resources need clear placement.",
        "fix": "Keep blocking work, runtime setup, and request-scope behavior on the side of the boundary that owns them.",
    },
    ("rust", "runtime_ownership"): {
        "label": "Runtime Ownership",
        "summary": "Ownership of runtime resources, handles, and long-lived state.",
        "why": "Runtime-ownership rules matter because unclear ownership of tasks or runtime resources turns shutdown and coordination into guesswork.",
        "fix": "Give runtime resources a visible owner and a clear drop or shutdown story instead of letting them escape implicitly.",
    },
    ("rust", "security_footguns"): {
        "label": "Security Footguns",
        "summary": "Unsafe defaults and security-sensitive misuse patterns.",
        "why": "Security-footgun rules matter because a small convenience shortcut can erase the guarantees people assume Rust code is giving them.",
        "fix": "Prefer safe-by-default APIs, validate boundary input, and isolate sensitive behavior behind strongly typed helpers.",
    },
    ("rust", "unsafe_soundness"): {
        "label": "Unsafe Soundness",
        "summary": "Unsafe operations that deserve a sharper review pass.",
        "why": "Unsafe-soundness rules matter because every unsafe block carries a local proof obligation that future maintainers also have to trust.",
        "fix": "Keep unsafe blocks small, document the invariant they rely on, and move complexity back into safe abstractions whenever possible.",
    },
}

LOWERCASE_TITLE_WORDS = {
    "a",
    "an",
    "and",
    "as",
    "at",
    "by",
    "for",
    "in",
    "of",
    "on",
    "or",
    "the",
    "to",
    "vs",
    "with",
    "without",
}

TITLE_WORD_OVERRIDES = {
    "api": "API",
    "auth": "Auth",
    "cli": "CLI",
    "cpu": "CPU",
    "ctx": "Ctx",
    "db": "DB",
    "dto": "DTO",
    "ffi": "FFI",
    "gin": "Gin",
    "gorm": "GORM",
    "grpc": "gRPC",
    "http": "HTTP",
    "https": "HTTPS",
    "id": "ID",
    "io": "I/O",
    "json": "JSON",
    "llm": "LLM",
    "ml": "ML",
    "mlops": "MLOps",
    "orm": "ORM",
    "os": "OS",
    "pgx": "pgx",
    "sql": "SQL",
    "sqlx": "sqlx",
    "tx": "TX",
    "uow": "UoW",
    "uri": "URI",
    "url": "URL",
    "uuid": "UUID",
}


def resolve_features_path() -> Path:
    legacy_path = GUIDES_PATH / "features-and-detections.md"
    if legacy_path.exists():
        return legacy_path

    versioned_candidates: list[tuple[tuple[int, int, int], Path]] = []
    for child in GUIDES_PATH.iterdir():
        if not child.is_dir():
            continue
        match = re.fullmatch(r"v(\d+)\.(\d+)\.(\d+)", child.name)
        if not match:
            continue

        candidate = child / "features-and-detections.md"
        if candidate.exists():
            versioned_candidates.append(
                (tuple(int(part) for part in match.groups()), candidate)
            )

    if not versioned_candidates:
        raise SystemExit(
            "failed to find guides/features-and-detections.md or a versioned replacement"
        )

    versioned_candidates.sort(key=lambda item: item[0], reverse=True)
    return versioned_candidates[0][1]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate without writing files")
    args = parser.parse_args()

    registry, registry_json = load_registry()
    validate_registry(registry)

    cargo_version = load_cargo_version()
    action_inputs = parse_action_inputs(ACTION_PATH.read_text(encoding="utf-8"))

    changed = []
    changed.extend(
        sync_registry_json(
            RULES_REGISTRY_PATH,
            registry_json,
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_RULE_SUMMARY_START -->",
            "<!-- GENERATED_RULE_SUMMARY_END -->",
            build_readme_rule_summary(registry),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_INPUTS_START -->",
            "<!-- GENERATED_ACTION_INPUTS_END -->",
            build_action_inputs_markdown(action_inputs),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_SCAN_EXAMPLE_START -->",
            "<!-- GENERATED_ACTION_SCAN_EXAMPLE_END -->",
            build_readme_action_example(
                build_action_scan_example(README_ACTION_REF.format(version=cargo_version))
            ),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_JSON_EXAMPLE_START -->",
            "<!-- GENERATED_ACTION_JSON_EXAMPLE_END -->",
            build_readme_action_example(
                build_action_json_example(README_ACTION_REF.format(version=cargo_version))
            ),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            README_PATH,
            "<!-- GENERATED_ACTION_BENCH_EXAMPLE_START -->",
            "<!-- GENERATED_ACTION_BENCH_EXAMPLE_END -->",
            build_readme_action_example(
                build_action_bench_example(README_ACTION_REF.format(version=cargo_version))
            ),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            DOCS_CONTENT_PATH,
            "// GENERATED_RULES_START",
            "// GENERATED_RULES_END",
            build_frontend_rules_block(registry),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_frontend_rule_catalog(
            registry,
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            DOCS_CONTENT_PATH,
            "// GENERATED_ACTION_INPUTS_START",
            "// GENERATED_ACTION_INPUTS_END",
            build_frontend_action_inputs_block(action_inputs),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_marked_block(
            DOCS_CONTENT_PATH,
            "// GENERATED_ACTION_EXAMPLES_START",
            "// GENERATED_ACTION_EXAMPLES_END",
            build_frontend_action_examples_block(),
            check_only=args.check,
        )
    )
    changed.extend(
        sync_features_inventory(
            registry,
            cargo_version,
            check_only=args.check,
        )
    )

    if args.check:
        if changed:
            for path in changed:
                print(f"out of date: {path}")
            return 1
        print("docs are in sync")
        return 0

    for path in changed:
        print(f"updated: {path}")
    return 0


def load_registry() -> tuple[list[dict], str]:
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--", "rules", "--json"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout), result.stdout


def sync_registry_json(
    path: Path,
    registry_json: str,
    *,
    check_only: bool,
) -> list[Path]:
    original = path.read_text(encoding="utf-8")
    if original == registry_json:
        return []

    if not check_only:
        path.write_text(registry_json, encoding="utf-8")
    return [path]


def sync_frontend_rule_catalog(
    registry: list[dict],
    *,
    check_only: bool,
) -> list[Path]:
    desired_files = build_frontend_rule_catalog_files(registry)
    existing_files = set()

    if FRONTEND_RULES_GENERATED_DIR.exists():
        existing_files = {
            path
            for path in FRONTEND_RULES_GENERATED_DIR.rglob("*.json")
            if path.is_file()
        }

    changed: list[Path] = []
    stale_files = existing_files - set(desired_files)
    for path in sorted(stale_files):
        changed.append(path)
        if not check_only:
            path.unlink()

    for path, content in sorted(desired_files.items()):
        original = path.read_text(encoding="utf-8") if path.exists() else None
        if original == content:
            continue
        changed.append(path)
        if not check_only:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

    return changed


def validate_registry(registry: list[dict]) -> None:
    pairs = [(item["language"], item["id"]) for item in registry]
    if len(pairs) != len(set(pairs)):
        duplicates = [
            f"{language}:{rule_id}"
            for (language, rule_id), count in Counter(pairs).items()
            if count > 1
        ]
        raise SystemExit(f"duplicate language-scoped rule ids in registry: {duplicates}")

    for item in registry:
        if item["language"] not in LANGUAGE_ORDER:
            raise SystemExit(f"unknown registry language: {item['language']}")
        if item["status"] not in STATUS_ORDER:
            raise SystemExit(f"unknown registry status: {item['status']}")
        if not item["family"] or not item["description"]:
            raise SystemExit(f"registry entry is missing family/description: {item['id']}")

    expected = sorted(
        registry,
        key=lambda item: (
            LANGUAGE_ORDER.index(item["language"]),
            item["family"],
            item["id"],
        ),
    )
    if registry != expected:
        raise SystemExit("registry must stay sorted by language, family, then id")


def load_cargo_version() -> str:
    cargo_text = CARGO_TOML_PATH.read_text(encoding="utf-8")
    match = re.search(r'^version = "([^"]+)"$', cargo_text, re.M)
    if not match:
        raise SystemExit("failed to parse Cargo.toml version")
    return match.group(1)


def parse_action_inputs(text: str) -> list[dict]:
    inputs: list[dict] = []
    lines = text.splitlines()
    in_inputs = False
    current: dict | None = None

    for raw_line in lines:
        line = raw_line.rstrip()
        stripped = line.strip()
        indent = len(line) - len(line.lstrip(" "))

        if stripped == "inputs:":
            in_inputs = True
            current = None
            continue

        if in_inputs and indent == 0 and stripped.endswith(":") and stripped != "inputs:":
            break

        if not in_inputs or not stripped:
            continue

        if indent == 2 and stripped.endswith(":"):
            if current is not None:
                inputs.append(current)
            current = {"name": stripped[:-1]}
            continue

        if current is None or indent < 4 or ":" not in stripped:
            continue

        key, value = stripped.split(":", 1)
        current[key.strip()] = value.strip().strip("'\"")

    if current is not None:
        inputs.append(current)

    return inputs


def sync_marked_block(
    path: Path,
    start_marker: str,
    end_marker: str,
    generated_body: str,
    *,
    check_only: bool,
) -> list[Path]:
    original = path.read_text(encoding="utf-8")
    start_token = f"{start_marker}\n"
    start_index = original.find(start_token)
    if start_index < 0:
        raise SystemExit(f"failed to locate generated block in {path}")

    content_start = start_index + len(start_token)
    end_index = original.find(end_marker, content_start)
    if end_index < 0:
        raise SystemExit(f"failed to locate generated block in {path}")

    replacement_body = generated_body
    if replacement_body:
        replacement_body += "\n"
    updated = original[:content_start] + replacement_body + original[end_index:]

    if updated == original:
        return []

    if not check_only:
        path.write_text(updated, encoding="utf-8")
    return [path]


def sync_features_inventory(
    registry: list[dict],
    cargo_version: str,
    *,
    check_only: bool,
) -> list[Path]:
    features_path = resolve_features_path()
    original = features_path.read_text(encoding="utf-8")
    generated = build_features_inventory(registry, cargo_version)
    pattern = re.compile(
        r"(## What deslop detects today\n\n)(.*?)(\n## Detection philosophy)",
        re.S,
    )
    updated, count = pattern.subn(
        lambda match: f"{match.group(1)}{generated}{match.group(3)}",
        original,
    )
    if count != 1:
        raise SystemExit(f"failed to locate inventory section in {features_path}")

    if updated == original:
        return []

    if not check_only:
        features_path.write_text(updated, encoding="utf-8")
    return [features_path]


def build_readme_rule_summary(registry: list[dict]) -> str:
    counts = summarize_counts(registry)
    lines = [
        "deslop now publishes a central rule registry that drives the CLI and the synced docs surfaces.",
        "",
        "| Language | Stable | Experimental | Research | Total |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    total_stable = total_experimental = total_research = total_rules = 0
    for language in LANGUAGE_ORDER:
        stable = counts[language]["stable"]
        experimental = counts[language]["experimental"]
        research = counts[language]["research"]
        total = stable + experimental + research
        total_stable += stable
        total_experimental += experimental
        total_research += research
        total_rules += total
        lines.append(
            f"| {language} | {stable} | {experimental} | {research} | {total} |"
        )
    lines.append(
        f"| total | {total_stable} | {total_experimental} | {total_research} | {total_rules} |"
    )
    lines.append("")
    lines.append(
        "The totals above are language-scoped rule entries, so a shared rule ID implemented in more than one backend appears in each relevant language bucket."
    )
    lines.append(
        "The registry is now the source of truth for `deslop rules`, the frontend rule catalog, and the generated detection inventory guide."
    )
    return "\n".join(lines)


def build_action_inputs_markdown(inputs: list[dict]) -> str:
    lines = []
    for item in inputs:
        description = item.get("description", "")
        default = item.get("default", "")
        required = item.get("required", "")
        extra = []
        if default:
            extra.append(f"Defaults to `{default}`.")
        if required:
            extra.append("Required." if required == "true" else "Optional.")
        suffix = f" {' '.join(extra)}" if extra else ""
        lines.append(f"- `{item['name']}`: {description}{suffix}")
    return "\n".join(lines)


def build_readme_action_example(body: str) -> str:
    return f"```yaml\n{body}\n```"


def build_frontend_rules_block(registry: list[dict]) -> str:
    del registry
    return (
        "// Rule catalog now lives in generated JSON under ./generated/rule-manifest.json\n"
        "// and ./generated/rules/<language>/<family>.json."
    )


def build_frontend_rule_catalog_files(registry: list[dict]) -> dict[Path, str]:
    grouped: dict[str, dict[str, list[dict]]] = defaultdict(lambda: defaultdict(list))
    for item in registry:
        grouped[item["language"]][item["family"]].append(item)

    manifest = {"languages": {}}
    files: dict[Path, str] = {}
    generated_rule_docs: list[dict[str, str]] = []

    for language in LANGUAGE_ORDER:
        family_entries = []
        language_families = grouped[language]
        for family in sorted(language_families):
            items = language_families[family]
            metadata = family_metadata(language, family)
            family_entries.append(
                {
                    "id": family,
                    "label": metadata["label"],
                    "summary": metadata["summary"],
                    "ruleCount": len(items),
                    "rules": [
                        {
                            "id": item["id"],
                            "label": humanize_identifier(item["id"]),
                            "defaultSeverity": item["default_severity"],
                            "status": item["status"],
                        }
                        for item in items
                    ],
                }
            )

            rule_docs = [build_rule_doc(language, family, item) for item in items]
            for rule_doc in rule_docs:
                generated_rule_docs.append(
                    {
                        "language": language,
                        "family": family,
                        "id": rule_doc["id"],
                        "explanation": rule_doc["explanation"],
                        "fix": rule_doc["fix"],
                    }
                )

            family_chunk = {
                "language": language,
                "family": family,
                "label": metadata["label"],
                "summary": metadata["summary"],
                "rules": rule_docs,
            }
            files[
                FRONTEND_RULE_CHUNKS_DIR / language / f"{family}.json"
            ] = json.dumps(family_chunk, indent=2) + "\n"

        manifest["languages"][language] = {
            "ruleCount": sum(entry["ruleCount"] for entry in family_entries),
            "families": family_entries,
        }

    files[FRONTEND_RULE_MANIFEST_PATH] = json.dumps(manifest, indent=2) + "\n"
    validate_unique_rule_copy(generated_rule_docs)
    return files


def build_frontend_action_inputs_block(inputs: list[dict]) -> str:
    lines = ["const githubActionInputs: GitHubActionInput[] = ["]
    for item in inputs:
        description = item.get("description", "")
        default = item.get("default", "")
        required = item.get("required", "")
        extra = []
        if default:
            extra.append(f"Defaults to {default}.")
        if required:
            extra.append("Required." if required == "true" else "Optional.")
        final_description = " ".join([description, *extra]).strip()
        lines.append(
            f"  {{ name: '{item['name']}', description: '{ts_string(final_description)}' }},"
        )
    lines.append("]")
    return "\n".join(lines)


def build_frontend_action_examples_block() -> str:
    workflow = ts_template(build_action_scan_example(FRONTEND_ACTION_REF))
    json_example = ts_template(build_action_json_example(FRONTEND_ACTION_REF))
    bench_example = ts_template(build_action_bench_example(FRONTEND_ACTION_REF))
    return "\n".join(
        [
            f"const githubActionWorkflow = `{workflow}`",
            "",
            f"const githubActionJsonExample = `{json_example}`",
            "",
            f"const githubActionBenchExample = `{bench_example}`",
        ]
    )


def family_metadata(language: str, family: str) -> dict[str, str]:
    metadata = FAMILY_METADATA.get((language, family))
    if metadata is not None:
        return metadata

    label = humanize_identifier(family)
    return {
        "label": label,
        "summary": f"{label} rules for the {language} catalog.",
        "why": f"{label} rules matter because they highlight review-worthy patterns before they spread across the repository.",
        "fix": f"Keep the {label.lower()} boundary intentional and prefer one clear owner for the pattern this rule is pointing at.",
    }


def build_rule_doc(language: str, family: str, item: dict) -> dict:
    metadata = family_metadata(language, family)
    why = build_rule_why(item, metadata["why"])
    fix = build_rule_fix(item, metadata["fix"])
    return {
        "id": item["id"],
        "label": humanize_identifier(item["id"]),
        "description": item["description"],
        "defaultSeverity": item["default_severity"],
        "status": item["status"],
        "configurability": item["configurability"],
        "explanation": why,
        "fix": fix,
    }


def build_rule_why(item: dict, base: str) -> str:
    label = humanize_identifier(item["id"])
    language = humanize_identifier(item["language"])
    description = clean_rule_description(item["description"])
    clause = build_rule_impact_clause(item, description, base)
    return f"In {language}, {label} matters because the pattern here, {description}, {clause}."


def build_rule_fix(item: dict, base: str) -> str:
    label = humanize_identifier(item["id"])
    language = humanize_identifier(item["language"])
    clause = build_rule_fix_clause(item, base)
    return f"For {label} in {language}, {clause}."


def build_rule_impact_clause(item: dict, description: str, base: str) -> str:
    text = rule_search_text(item)
    description_lower = description.lower()

    if contains_any(text, "comment", "comments", "commentary", "doc", "documentation", "docstring", "tutorial"):
        return "turns documentation into noise instead of preserving the intent a maintainer actually needs"
    if contains_any(text, "typing", "annotation", "schema", "response_model", "content-type"):
        return "weakens the contract at the exact place callers need the shape or boundary to be explicit"
    if contains_any(text, "name", "names", "identifier", "naming"):
        return "hides the real job of the symbol behind wording that is harder to scan during review"
    if contains_any(text, "import", "package", "module", "router", "blueprint", "entrypoint"):
        return "makes ownership and navigation harder when someone has to trace behavior across modules or services"
    if contains_any(text, "config", "env", "environment", "startup", "bootstrap"):
        return "pushes configuration or startup ownership into the wrong phase of the program"
    if contains_any(text, "class", "constructor", "inheritance", "surface area"):
        return "concentrates too much responsibility into one shape and makes future change riskier"
    if contains_any(text, "dataframe", "dataset", "embedding", "dataloader", "pipeline", "inference"):
        return "pushes data or model work into a shape that is harder to scale, cache, or reproduce"
    if " instead of " in description_lower:
        return "shows that the code is bypassing a clearer or safer default that readers usually expect to see"
    if contains_fragment(description_lower, "inside loops", "inside loop", "each iteration", "per request", "per row", "per chunk"):
        return "pushes avoidable work into a repeated path where the cost compounds quickly"
    if " without " in description_lower:
        return "removes a guardrail or signal that reviewers expect before trusting the surrounding code"
    if description_lower.startswith("tests that"):
        return "can keep the suite green while still leaving the risky path unproven"
    if description_lower.startswith("functions that"):
        return "makes the API contract harder to scan and easier to misuse"
    if description_lower.startswith("package-qualified calls") or description_lower.startswith("same-package calls"):
        return "looks plausible on the surface but breaks down once someone follows the symbol through local code"
    if contains_any(text, "loop", "loops", "repeated", "batch"):
        return "turns local work into repeated cost that spreads with traffic or input size"
    if contains_any(text, "transaction", "commit", "rollback", "preload"):
        return "stretches the write boundary and makes failure ownership harder to reason about"
    if contains_any(text, "handler", "middleware", "route", "router", "gin"):
        return "blurs the boundary between request plumbing and the rest of the application"
    if contains_any(text, "service", "repository", "dto", "model", "mapper"):
        return "lets layer-specific contracts leak into places that should stay simpler and easier to test"
    if contains_any(text, "context", "goroutine", "channel", "ticker", "shutdown", "cancel", "async", "await", "spawn"):
        return "creates lifecycle behavior that is easy to miss until the system is under load or shutting down"
    if contains_any(text, "secret", "crypto", "auth", "tenant", "sql", "query", "plugin", "password", "credential", "token"):
        return "hides a sensitive edge where convenience shortcuts can become security or data-handling incidents"
    if contains_any(text, "test", "mock", "assert", "placeholder"):
        return "weakens the evidence that tests or review are supposed to provide"
    if contains_any(text, "unsafe", "unchecked", "assume_init", "raw", "aliasing"):
        return "depends on an invariant that future maintainers also have to preserve exactly"

    return build_base_impact_clause(base)


def build_rule_fix_clause(item: dict, base: str) -> str:
    text = rule_search_text(item)
    description = clean_rule_description(item["description"])
    description_lower = description.lower()

    if contains_any(text, "comment", "comments", "commentary", "doc", "documentation", "docstring", "tutorial"):
        return "rewrite or delete the comment so it captures intent, invariants, or tradeoffs instead of narrating obvious steps"
    if contains_any(text, "typing", "annotation", "schema", "response_model", "content-type"):
        return "tighten the type, annotation, schema, or response contract so callers can see the expected shape at the boundary"
    if contains_any(text, "name", "names", "identifier", "naming"):
        return "rename the symbol so its domain job is obvious without reading the whole body"
    if contains_any(text, "import", "package", "module", "router", "blueprint", "entrypoint"):
        return "move the code to the package, module, router, or entrypoint layer that already owns this boundary so navigation and responsibility stay aligned"
    if contains_any(text, "config", "env", "environment", "startup", "bootstrap"):
        return "load, validate, and wire this configuration at startup so request paths and imports only consume already-owned state"
    if contains_any(text, "assert") and contains_any(text, "runtime", "production", "input"):
        return "replace assert-based runtime checks with explicit validation and normal error handling that still runs in production"
    if contains_any(text, "class", "constructor", "inheritance", "surface area"):
        return "split the class or constructor work so one object owns one role and collaborators are injected or composed deliberately"
    if contains_any(text, "dataframe", "dataset", "embedding", "dataloader", "pipeline", "inference"):
        return "move expensive data or model work to the right pipeline stage, cache stable results, and keep the runtime path lean"
    if " instead of " in description_lower:
        preferred = description.split(" instead of ", 1)[1].strip().rstrip(".")
        return (
            f"prefer {preferred} when the semantics match, and keep the expected or cheaper API choice as the default"
        )
    if contains_fragment(description_lower, "inside loops", "inside loop", "each iteration", "per request", "per row", "per chunk"):
        return (
            "move the one-time work out of the repeated path, cache reusable state, or batch the operation so the loop only does per-item work"
        )
    if " without " in description_lower:
        missing = description.split(" without ", 1)[1].strip().rstrip(".")
        return f"add {missing} or move the code to a path where that safeguard is guaranteed before the risky work happens"
    if description_lower.startswith("tests that"):
        return "add assertions, negative-path coverage, or explicit failure checks that prove the behavior this pattern currently leaves implicit"
    if description_lower.startswith("functions that"):
        if contains_any(text, "context"):
            return "reshape the signature so context stays explicit at the call boundary and the expected parameter order is visible immediately"
        return "reshape the signature or call contract so the expected parameter order, ownership, or return shape is obvious at the definition site"
    if description_lower.startswith("package-qualified calls") or description_lower.startswith("same-package calls"):
        return "replace the unresolved symbol with a locally declared or imported symbol that actually exists in the scanned repository context"
    if contains_any(text, "string", "concat", "builder", "buffer"):
        return "prefer a builder, buffer, or one-time composition step instead of growing the same value incrementally"
    if contains_any(text, "loop", "loops", "repeated"):
        return "move one-time initialization, parsing, or allocation outside the loop when the per-iteration state does not actually change"
    if contains_any(text, "transaction", "commit", "rollback"):
        return "keep the transaction focused on the minimum database work and move unrelated orchestration outside the open transaction"
    if contains_any(text, "handler", "middleware", "route", "router", "gin"):
        return "move the non-transport work into middleware, services, bootstrap code, or shared helpers so the request edge stays narrow"
    if contains_any(text, "service", "repository", "dto", "model", "mapper"):
        return "choose one layer to own the contract shape and perform the mapping there instead of passing boundary-specific types through the stack"
    if contains_any(text, "context", "goroutine", "channel", "ticker", "shutdown", "cancel", "async", "await", "spawn"):
        return "give the work an explicit owner, pass the right context or handle, and make cancellation or shutdown visible in the API"
    if contains_any(text, "secret", "crypto", "auth", "tenant", "password", "credential", "token"):
        return "pull the sensitive value from env, config, or a secret manager instead of embedding it directly in source"
    if contains_any(text, "sql", "query", "plugin"):
        return "switch to parameterized access, validated input, or a reviewed helper so the data boundary stays explicit and safe"
    if contains_any(text, "test", "mock", "assert", "placeholder"):
        return "tighten the tests until the failure mode, edge case, or assertion signal is obvious to a reviewer"
    if contains_any(text, "unsafe", "unchecked", "assume_init", "raw", "aliasing"):
        return "shrink the unsafe surface, document the invariant, or move the behavior behind a safe abstraction"

    return to_sentence_clause(base)


def clean_rule_description(description: str) -> str:
    return re.sub(r"\s+", " ", description.strip()).rstrip(".")


def contains_fragment(text: str, *needles: str) -> bool:
    return any(needle in text for needle in needles)


def build_base_impact_clause(base: str) -> str:
    stripped = base.strip().rstrip(".")
    if " because " in stripped:
        _, reason = stripped.split(" because ", 1)
        return f"is worth review because {to_sentence_clause(reason)}"
    return "is the kind of code shape that quietly spreads maintenance and review cost once it becomes normal"


def to_sentence_clause(text: str) -> str:
    stripped = text.strip().rstrip(".")
    if stripped and stripped[0].isalpha():
        return stripped[0].lower() + stripped[1:]
    return stripped


def validate_unique_rule_copy(rule_docs: list[dict[str, str]]) -> None:
    seen_explanations: dict[str, str] = {}
    seen_fixes: dict[str, str] = {}
    duplicates: list[str] = []

    for rule in rule_docs:
        key = f"{rule['language']}/{rule['family']}/{rule['id']}"

        explanation_key = rule["explanation"].strip()
        prior_explanation = seen_explanations.get(explanation_key)
        if prior_explanation is not None:
            duplicates.append(
                f"duplicate explanation: {key} matches {prior_explanation}"
            )
        else:
            seen_explanations[explanation_key] = key

        fix_key = rule["fix"].strip()
        prior_fix = seen_fixes.get(fix_key)
        if prior_fix is not None:
            duplicates.append(f"duplicate fix: {key} matches {prior_fix}")
        else:
            seen_fixes[fix_key] = key

    if duplicates:
        preview = "\n".join(duplicates[:20])
        remainder = len(duplicates) - min(len(duplicates), 20)
        suffix = f"\n... and {remainder} more" if remainder > 0 else ""
        raise SystemExit(f"generated rule copy is not unique:\n{preview}{suffix}")


def rule_search_text(item: dict) -> str:
    return " ".join(
        [
            item["language"],
            item["family"],
            item["id"].replace("_", " "),
            item["description"],
        ]
    ).lower()


def contains_any(text: str, *needles: str) -> bool:
    return any(
        re.search(rf"\b{re.escape(needle)}\b", text) is not None
        for needle in needles
    )


def merge_sentences(base: str, extra_sentences: list[str]) -> str:
    parts = [base.strip(), *[sentence.strip() for sentence in extra_sentences if sentence.strip()]]
    return " ".join(part.rstrip(".") + "." for part in parts if part)


def humanize_identifier(value: str) -> str:
    words = value.replace("-", "_").split("_")
    output = []
    for index, word in enumerate(words):
        lower = word.lower()
        if lower in TITLE_WORD_OVERRIDES:
            output.append(TITLE_WORD_OVERRIDES[lower])
            continue
        if index > 0 and lower in LOWERCASE_TITLE_WORDS:
            output.append(lower)
            continue
        output.append(lower.capitalize())
    return " ".join(output)


def build_features_inventory(registry: list[dict], cargo_version: str) -> str:
    counts = summarize_counts(registry)
    total_rules = len(registry)
    lines = [
        f"The shipped registry currently tracks **{total_rules} language-scoped rule entries** in deslop `{cargo_version}`.",
        "",
        "| Language | Stable | Experimental | Research | Total |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for language in LANGUAGE_ORDER:
        stable = counts[language]["stable"]
        experimental = counts[language]["experimental"]
        research = counts[language]["research"]
        total = stable + experimental + research
        lines.append(
            f"| {language} | {stable} | {experimental} | {research} | {total} |"
        )

    lines.append("")
    lines.append(
        "The sections below are generated from the rule registry and grouped by language and family."
    )
    lines.append(
        "When the same rule ID is implemented in more than one backend, it appears once in each relevant language section."
    )
    lines.append("")

    by_language: dict[str, dict[str, list[dict]]] = defaultdict(lambda: defaultdict(list))
    for item in registry:
        by_language[item["language"]][item["family"]].append(item)

    for language in LANGUAGE_ORDER:
        language_rules = by_language[language]
        total = sum(len(items) for items in language_rules.values())
        lines.append(f"### {language.title()} rules ({total})")
        lines.append("")
        for family in sorted(language_rules):
            lines.append(f"#### {family.replace('_', ' ').title()} ({len(language_rules[family])})")
            for item in language_rules[family]:
                status_suffix = (
                    f" *(status: {item['status']})*"
                    if item["status"] != "stable"
                    else ""
                )
                lines.append(
                    f"- `{item['id']}`: {item['description']}{status_suffix}"
                )
            lines.append("")

    return "\n".join(lines).rstrip()


def summarize_counts(registry: list[dict]) -> dict[str, Counter]:
    counts = {language: Counter() for language in LANGUAGE_ORDER}
    for item in registry:
        counts[item["language"]][item["status"]] += 1
    return counts


def ts_string(value: str) -> str:
    return value.replace("\\", "\\\\").replace("'", "\\'")


def ts_template(value: str) -> str:
    return value.replace("\\", "\\\\").replace("`", "\\`")


def build_action_scan_example(action_ref: str) -> str:
    return "\n".join(
        [
            "name: Deslop",
            "",
            "on:",
            "  pull_request:",
            "  push:",
            "    branches:",
            "      - main",
            "",
            "jobs:",
            "  scan:",
            "    runs-on: ubuntu-latest",
            "    steps:",
            "      - uses: actions/checkout@v4",
            f"      - uses: {action_ref}",
            "        with:",
            "          path: .",
        ]
    )


def build_action_json_example(action_ref: str) -> str:
    return "\n".join(
        [
            "- uses: actions/checkout@v4",
            f"- uses: {action_ref}",
            "  with:",
            "    path: .",
            "    json: 'true'",
            "    details: 'true'",
            "    fail-on-findings: 'false'",
        ]
    )


def build_action_bench_example(action_ref: str) -> str:
    return "\n".join(
        [
            "- uses: actions/checkout@v4",
            f"- uses: {action_ref}",
            "  with:",
            "    command: bench",
            "    path: .",
            "    repeats: '10'",
            "    warmups: '2'",
        ]
    )


if __name__ == "__main__":
    sys.exit(main())
