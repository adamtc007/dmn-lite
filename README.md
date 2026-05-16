# dmn-lite

dmn-lite is the decisioning engine half of a Rust-native Camunda 8 replacement. It provides a static compiler and deterministic stack-VM runtime for decision tables authored in a formal s-expression DSL. The compiler pipeline runs parse → AST → typed predicate IR → bytecode; the runtime executes a compact token stream over a typed input frame and never tree-walks the AST in production.

For architecture, scope, and acceptance criteria, read the V&S:
**`bpmn-dmn-lite-vs-v0_4.md`** — Vision and Scope (authoritative).

For the execution model and bytecode design rationale, read:
**`dmn-lite-compiler-owned-execution-mini-thesis-v0_1.md`** — Semantic intent vs. execution order.

---

## Crate layout

| Crate | Purpose |
| --- | --- |
| `dmn-lite-types` | Shared type vocabulary: IDs, values, predicates, bytecode instructions, error types, compiled artifact |
| `dmn-lite-parser` | S-expression DSL parser → typed AST (Phase 1.1) |
| `dmn-lite-compiler` | AST → typed predicate IR → bytecode emission (Phases 1.2–1.4) |
| `dmn-lite-analysis` | Static analysis: gap, overlap, unreachable rules, hit-policy diagnostics (Phase 1.6) |
| `dmn-lite-engine` | Stack VM (Phase 1.4) and reference evaluator oracle (Phase 1.3) |

Dependency rule: `dmn-lite-engine` depends only on `dmn-lite-types`. It has no knowledge of the compiler. The engine and compiler communicate exclusively through the `CompiledDecision` artifact type.

---

## Implementation status

**Phase 1.0 complete.** Workspace scaffolded, all crates compile clean, three specification documents written.

**Phase 1.1 (parser) is next.**

Phases 1.0–1.6 implement the full static decision-table core (Profile v0.1 per V&S §17).

---

## Quick start

```sh
cd dmn-lite
cargo build        # should succeed with no warnings
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

No tests yet — Phase 1.0 produces compile-only skeletons. Every public function returns `unimplemented!()` with a phase-attributed message.

---

## Specification documents

| Document | Contents |
| --- | --- |
| `docs/dmn-lite-ebnf.md` | Formal EBNF grammar for the s-expression DSL |
| `docs/dmn-lite-semantics.md` | Static and dynamic semantics, hit-policy rules, predicate evaluation |
| `docs/dmn-lite-bytecode.md` | Bytecode instruction set, stack discipline, const pools, verification invariants |

These documents are implementation contracts. Phases 1.1–1.4 implement against them.
