//! dmn-lite static analysis: overlap, gap, unreachable rules, hit-policy diagnostics.
//!
//! Analyses a compiled decision artifact for:
//! - rule coverage: are all possible input combinations covered?
//! - overlap: can multiple rules match the same inputs (violation for `UNIQUE`)?
//! - unreachable rules: rules that can never be reached given earlier rules.
//! - hit-policy consistency: does the rule set satisfy the declared hit policy?
//!
//! The analyser operates over the typed predicate IR in the compiled artifact,
//! not the source AST. Finite domains (enum types) enable exhaustive analysis.
//!
//! Phase 1.0 status: skeleton only. Analysis implemented in Phase 1.6.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use dmn_lite_types::{AnalysisReport, CompiledDecision};

/// Run static analysis on a compiled decision.
///
/// Returns an [`AnalysisReport`] summarising coverage, overlap, gap,
/// unreachable-rule, and hit-policy diagnostics. For decisions with finite
/// enum domains, the analysis is exhaustive. For decisions with unbounded
/// numeric ranges, the analysis is conservative.
///
/// Phase 1.0: returns `unimplemented!()`. Real implementation in Phase 1.6.
pub fn analyse(_decision: &CompiledDecision) -> AnalysisReport {
    unimplemented!("dmn-lite-analysis: analyse() not implemented until Phase 1.6")
}
