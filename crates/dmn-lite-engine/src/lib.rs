//! dmn-lite engine: reference evaluator and (future) bytecode stack VM.
//!
//! Two evaluators share the same input/output contract:
//!
//! - [`reference`] module: reference evaluator over typed predicate IR
//!   (Phase 1.3). Used as the differential testing oracle for the VM.
//!   Correct but not optimised; never short-circuits.
//!
//! - [`vm`] module: production bytecode stack machine (Phase 1.4, stub).
//!
//! The engine depends only on `dmn-lite-types`. It has no knowledge of the
//! compiler implementation — only of the [`TypedDecision`] artifact shape.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod reference;
pub mod vm;

use dmn_lite_types::ir::TypedDecision;
use dmn_lite_types::{EvalError, TypedInputContext};

pub use reference::{EvaluationOutput, evaluate as reference_evaluate};

/// Evaluate a compiled decision against a typed input context.
///
/// **Phase 1.3:** delegates to [`reference::evaluate`]. Phase 1.4 will
/// re-route this to the bytecode VM, retaining the reference evaluator as a
/// debugging/oracle mode.
///
/// The `source` string is forwarded to the reference evaluator for
/// human-readable predicate descriptions in the evaluation trace. Pass `""`
/// if the original source is unavailable.
pub fn evaluate(
    decision: &TypedDecision,
    input: &TypedInputContext,
    source: &str,
) -> Result<EvaluationOutput, EvalError> {
    reference::evaluate(decision, input, source)
}
