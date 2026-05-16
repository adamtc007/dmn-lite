//! Reference evaluator over typed predicate IR.
//!
//! Evaluates compiled predicates by interpreting the typed predicate IR
//! directly — correct but not optimised. Used exclusively as the differential
//! testing oracle for the bytecode VM: the same inputs fed to both evaluators
//! must produce the same outputs for all well-formed decisions.
//!
//! The reference evaluator is never used in production evaluation paths.
//! Production code uses the bytecode VM in the [`super::vm`] module.
//!
//! Phase 1.0 status: empty. Reference evaluator implemented in Phase 1.3.
