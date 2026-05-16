//! dmn-lite engine: stack VM and reference evaluator.
//!
//! Contains two evaluators sharing the same input/output contract:
//!
//! - [`vm`] module: production bytecode stack machine (Phase 1.4).
//!   Evaluates the compiled decision program using a data stack, return
//!   stack, typed input frame, and result accumulator. Never tree-walks
//!   the AST in the hot path.
//!
//! - [`reference`] module: reference evaluator over typed predicate IR
//!   (Phase 1.3). Used as the differential testing oracle for the VM.
//!   Correct but not optimised; replaced by the VM in production paths.
//!
//! The engine depends only on `dmn-lite-types`. It has no knowledge of the
//! compiler implementation — only of the [`CompiledDecision`] artifact shape.
//!
//! Phase 1.0 status: skeleton only.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod reference;
pub mod vm;

use dmn_lite_types::{CompiledDecision, EvalError, TypedInputContext, TypedOutputContext};

/// Evaluate a compiled decision against a typed input context.
///
/// Runs the bytecode VM (Phase 1.4) against the supplied input frame and
/// returns the typed output bindings from the matched rule(s). For differential
/// testing, use [`reference::evaluate`] as the oracle.
///
/// Returns an [`EvalError`] for input type mismatches, missing required inputs,
/// or hit-policy violations (e.g., multiple matches under `UNIQUE`).
///
/// Phase 1.0: returns `unimplemented!()`. Real implementation in Phase 1.4.
pub fn evaluate(
    _decision: &CompiledDecision,
    _input: &TypedInputContext,
) -> Result<TypedOutputContext, EvalError> {
    unimplemented!("dmn-lite-engine: evaluate() not implemented until Phase 1.4")
}
