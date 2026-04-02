# TypeScript Backend Research Spike

## Status

- Bucket: `backlog`
- Phase: `5`
- Tier: `research/deferred`
- Scope owner: backend architecture and rule-governance design, not implementation

## Purpose

This spike defines what must be true before deslop starts a fourth backend for TypeScript or JavaScript.

The goal is to reduce future re-planning by recording parser, index, rollout, and validation expectations now, while keeping the actual backend intentionally deferred until the Go, Python, and Rust governance work is stable.

## Preconditions

Do not begin implementation until all of the following are true:

- the rule registry remains the single source of truth for shipped detectors
- docs sync and drift checks are routine and green in CI
- the corpus harness is part of normal promotion work, not ad hoc release prep
- Go and Python expansion work is no longer blocked by governance gaps
- Rust runtime and workspace work has a stable promotion path

## Why TypeScript First

TypeScript and JavaScript are the preferred fourth backend because:

- the current architecture already favors source-driven, import-aware heuristics
- service, API, CLI, and frontend-adjacent repos create a broad real-world evaluation corpus
- the likely detector surface overlaps well with deslop's current strengths:
  - request-boundary churn
  - repeated client construction
  - async misuse
  - schema and contract imprecision
  - framework and data-layer operational smells

## Required Architecture Decisions

### Parser and file coverage

- support `.ts`, `.tsx`, `.js`, `.jsx`, and common declaration boundaries deliberately
- decide whether `.d.ts` files are indexed only, scan-only, or excluded from normal findings
- treat generated frontend build artifacts as ignored by default

### Repository index model

- resolve local imports across relative paths, barrel exports, and `index.ts` patterns
- account for `tsconfig.json` path aliases before claiming import hallucinations
- record framework-facing symbols conservatively so repo-level rules can stay low-noise

### Package metadata awareness

- parse `package.json` and `tsconfig.json` as first-class repository metadata
- support script and entrypoint resolution the same way Python Phase 3 now uses `pyproject.toml`
- define how workspace metadata from npm, pnpm, or Turborepo-style monorepos affects indexing

### Rollout and tiering

- introduce the backend behind explicit `experimental` status first
- keep any first-wave framework packs gated if noise is not yet proven low
- avoid shipping a broad default-on backend before corpus validation exists

## First-Wave Rule Families

When the backend starts, the first milestone should stay narrow and application-oriented:

- request-path client construction:
  - database clients
  - Redis clients
  - HTTP clients
  - SDK clients
- async misuse:
  - fire-and-forget promise chains
  - unbounded `Promise.all`
  - missing timeout or abort signals on upstream requests
- framework boundaries:
  - Express
  - Fastify
  - Next.js route handlers
  - NestJS controllers
- contract quality:
  - `any` in public API surfaces
  - schema validation gaps on external JSON boundaries
  - route handlers assembling oversized response objects repeatedly

## Explicit Non-Goals

- full frontend lint parity with ESLint ecosystems
- style-only findings that duplicate existing JS or TS linters
- bundler-specific optimization advice before backend service rules are proven
- a monorepo-wide architecture rewrite to support the backend

## Research Deliverables

The spike is complete when these deliverables exist:

- a written parser and metadata strategy
- a sample repository corpus list
- a first-wave rule inventory draft with `stable` versus `experimental` candidates
- a validation plan that uses the same corpus-harness and registry workflows as the existing backends

## Candidate Corpus

Minimum starting corpus:

- one Express or Fastify API service
- one Next.js or full-stack route-handler repo
- one NestJS or controller-heavy service
- one monorepo with `tsconfig` path aliases and package-workspace metadata

## Exit Criteria

This guide should remain research-only until:

- the master roadmap marks the backend as active work
- a concrete implementation guide exists for parser, index, and rule rollout
- the corpus list is populated with real targets
- the first detector families have fixture, suppression, and corpus evidence plans
