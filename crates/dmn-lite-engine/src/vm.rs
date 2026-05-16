//! Production bytecode stack VM.
//!
//! Executes the compiled decision bytecode using:
//! - a data stack for predicate and output values;
//! - a return stack for control-flow continuations (unused in Phase v0.1);
//! - a typed input frame supplying late-bound decision inputs;
//! - a result accumulator for matched rules and hit-policy state.
//!
//! The VM never resolves symbols dynamically. All field references are
//! pre-resolved `FieldId`s; all enum values are pre-resolved entity IDs
//! from the const pool. Execution is a simple dispatch loop:
//! `while pc < program.len() { execute program[pc] }`.
//!
//! Phase 1.0 status: empty. VM implemented in Phase 1.4.
