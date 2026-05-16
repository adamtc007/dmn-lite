# CLAUDE.md — dmn-lite

> **Last reviewed:** 2026-05-16
> **Repo:** github.com/adamtc007/dmn-lite
> **Status:** Phase 1 complete (Profile v0.1 — all six sub-phases done)
> **Related:** github.com/adamtc007/bpmn-lite (process vocabulary); github.com/adamtc007/ob-poc (BNY onboarding platform)
> **V&S:** `ob-poc/todo/dmn-lite/bpmn-dmn-lite-vs-v1_1.md`

dmn-lite is the **decision vocabulary** of the compilation-and-execution kernel described in V&S v1.1. It compiles an s-expression DSL to verified bytecode, evaluates typed decisions on a stack VM, and registers compiled decisions in the FFI catalogue so bpmn-lite processes can invoke them via `Instr::ExecFfi`.

---

## Quick Start

```bash
# Build
cargo build

# Test (345 tests)
cargo test

# Run analysis tests
cargo test -p dmn-lite-analysis

# Build and test the FFI bridge
cargo test -p dmn-lite-bridge
```

---

## Workspace Structure

```
dmn-lite/
├── Cargo.toml                    workspace root (6 members, resolver = "3", edition 2024)
├── crates/
│   ├── dmn-lite-types/           IDs (DomainId, ValueId, FieldId, RuleId, SnapshotId),
│   │                             IR types (TypedDecision, TypedRule, TypedValue, ResolvedType),
│   │                             compiled artifact (CompiledDecision, VerifiedDecision, ArtifactHash),
│   │                             analysis types (AnalysisReport, FindingKind, Severity),
│   │                             eval types (EvalError, EvaluationTrace, TypedInputContext)
│   ├── dmn-lite-parser/          S-expression DSL → AST (Source). Multi-error recovery.
│   ├── dmn-lite-compiler/        Source + Catalogue → CompiledDecision. Phases: resolve,
│   │                             type-check, emit bytecode, BLAKE3 artifact hash.
│   │                             verify() → VerifiedDecision (type-state separation).
│   ├── dmn-lite-engine/          Two evaluators sharing the same input/output contract:
│   │                             - reference: tree-walking, obviously-correct, no short-circuit
│   │                             - vm: production stack machine, short-circuit, verified-only
│   │                             Both: evaluate(&VerifiedDecision, &TypedInputContext, &str)
│   ├── dmn-lite-analysis/        Static analysis on VerifiedDecision (separate from compilation):
│   │                             SA-001 (UNIQUE+catch-all), overlap, unreachable-rule, gap,
│   │                             cost ceiling, string-input precision warning.
│   └── dmn-lite-bridge/          FfiExecutionOwner implementation (A10).
│                                 DmnLiteOwner: register_decision → FfiTemplate,
│                                 invoke(FfiCall) → FfiResult via dmn-lite stack VM.
│                                 Cross-repo dep: ffi-types from github.com/adamtc007/bpmn-lite.
```

---

## DSL Grammar (Profile v0.1)

```lisp
(define-decision <name>
  :hit-policy first | unique
  :inputs  ((<field> :type bool | integer | decimal | string | enum
                     :domain <domain-name>)
             ...)
  :outputs ((<field> :type <type> :domain <domain-name>)
             ...)
  :rules   ((rule <id>
               :when (<predicate> ...)  ; or (*) for catch-all
               :then (<assignment> ...))
             ...))
```

Predicate forms: `(field = literal)`, `(field != literal)`, `(field in [lo .. hi])`, `(field in (v1 v2 ...))`, `(*)` catch-all.

---

## Compilation Pipeline

```
Source (s-expression DSL)
  ↓ dmn-lite-parser::parse()
AST
  ↓ dmn-lite-compiler::compile(source, catalogue, source_text)
CompiledDecision { bytecode, typed_ir, artifact_hash, ... }
  ↓ dmn-lite-compiler::verify()
VerifiedDecision                  ← only this reaches the VM
  ↓ dmn-lite-engine::evaluate()
EvaluationOutput { output, trace }
```

**Artifact hash:** `BLAKE3(normalised_source + resolved_entity_ids + compiled_ir)`. Same source against same catalogue → same hash. Content-addressed identity.

**Type-state separation:** `VerifiedDecision(CompiledDecision)` is a newtype. The VM accepts only `VerifiedDecision`. `new_verified()` is callable only from `verify()`.

---

## Analysis (Phase 1.6, complete)

`dmn_lite_analysis::analyse(&VerifiedDecision, &Catalogue) -> AnalysisReport`

| Finding | Severity | Description |
|---------|----------|-------------|
| `UniqueWithCatchAll` | Error | UNIQUE hit-policy with a catch-all rule — always produces MultipleMatches |
| `Overlap { rule_a, rule_b }` | Warning (UNIQUE) / Info (FIRST) | Two rules match the same input |
| `UnreachableRule { unreachable, shadowing }` | Error | Earlier rule subsumes later rule under FIRST |
| `Gap { field_gaps }` | Warning | Input combination no rule matches |
| `CostCeilingExceeded` | Error | Predicate count exceeds configurable ceiling (default 10,000) |
| `AnalysisLimitedByStringInput` | Info | String/decimal fields are treated as opaque; analysis is approximate |

---

## dmn-lite-bridge (A10)

Registers compiled decisions as FFI templates for bpmn-lite processes.

```rust
let owner = Arc::new(DmnLiteOwner::new());

// Register a decision — returns the FfiTemplate for publication.
let template = owner.register_decision(
    verified_decision,
    vec![FieldSchema { name: "score".into(), kind: SchemaKind::I64, required: true }],
    vec![FieldSchema { name: "eligible".into(), kind: SchemaKind::Bool, required: false }],
    Idempotency::Idempotent,
    tenant_id,
    publisher,
);

// Publish template to FfiCatalogue (in bpmn-lite), register owner with FfiDispatcher.
// bpmn-lite processes call this decision via Instr::ExecFfi.
```

**Marshalling:** JSON `input_payload` → `TypedInputContext`. `Bool` → `TypedValue::Bool`, `I64` → `TypedValue::Integer`, `F64` → `TypedValue::Decimal`, `String` → `TypedValue::Str`. For `SemOsDomain` fields, an optional `ValueResolver` trait converts symbol strings to `TypedValue::Enum { domain_id, value_id }`; without it, falls back to `Str` (causes `InputTypeMismatch` — A12 scope for Sem OS catalogue integration).

**owner_metadata:** 32-byte BLAKE3 `artifact_hash` of the `VerifiedDecision`. Template identity is therefore unique per compiled artifact.

**ffi-types dep:** Public git dep on `github.com/adamtc007/bpmn-lite`. In the ob-poc monorepo, bpmn-lite's `[patch]` section redirects this to the local path so there is exactly one copy of `ffi-types` in the build graph.

---

## Two-Valued Null Semantics

Per `dmn-lite-semantics.md` §3.2: null on either side of `=` or `!=` produces `false` (not an error, not null-propagation). This differs from SQL three-valued logic. The VM's `NotEq` handler was fixed in Phase 1.5a to match this.

The reference evaluator never short-circuits; the VM does. Phase 1.5 differential harness proves equivalence across 3,000+ generated inputs per fixture.

---

## Profile Roadmap

| Profile | Status | Features |
|---------|--------|----------|
| v0.1 | ✅ Complete | integer/bool/enum/decimal/string, FIRST/UNIQUE, static analysis |
| v0.2 | ⬜ | Quantifiers + aggregation over governed collections, ANY/RULE_ORDER/COLLECT; gated on Sem OS governed-collection schema |
| v0.3 | ⬜ | DMN XML import linter, FEEL unary-test recogniser, temporal predicates |
| v0.4 | ⬜ | DRD cross-decision dependencies, path expressions |
| v0.5 | ⬜ | BKM non-recursive functions |

---

## Test Counts (2026-05-16)

| Crate | Tests |
|-------|-------|
| dmn-lite-parser | 78 |
| dmn-lite-compiler | 39 |
| dmn-lite-engine | 48 |
| dmn-lite-analysis | 57 |
| dmn-lite-bridge | 6 |
| **Phase 1 total** | **345** (228 without bridge) |
