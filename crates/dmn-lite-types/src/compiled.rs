//! Compiled decision artifact and analysis report.
//!
//! `CompiledDecision` is the immutable artifact produced by the compiler and
//! consumed by the engine. It contains no unresolved symbols: every input
//! reference is a `FieldId`, every enum value is a resolved entity ID.
//!
//! `AnalysisReport` is the summary produced by the static analyser over a
//! compiled decision, covering coverage, overlap, gap, and unreachable rules.
//!
//! Phase 1.0 status: placeholder structs only. Full shapes added in Phase 1.2.

/// Immutable compiled decision artifact produced by the compiler.
///
/// Contains the bytecode program, const pool, source map, hit policy,
/// input/output schemas, and analysis summary. The engine consumes this
/// artifact without any knowledge of how it was produced — the engine
/// and compiler are deliberately decoupled through this type.
///
/// Phase 1.0: placeholder. Fields added in Phase 1.2.
pub struct CompiledDecision {
    _private: (),
}

/// Static analysis report for a compiled decision.
///
/// Summarises coverage, overlap, gap, unreachable-rule, and hit-policy
/// diagnostics computed by `dmn-lite-analysis`. Stored alongside the
/// compiled artifact for audit and governance purposes.
///
/// Phase 1.0: placeholder. Fields added in Phase 1.6.
pub struct AnalysisReport {
    _private: (),
}
