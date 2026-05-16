//! dmn-lite compiler: AST → typed predicate IR → bytecode.
//!
//! Compiler responsibilities (Phases 1.2–1.4):
//! - resolve Sem OS concept references (early-bound symbols);
//! - validate type/domain assignments;
//! - infer decision input/output shape;
//! - lower predicates to typed predicate IR;
//! - lower outputs to typed result IR;
//! - validate hit policy;
//! - detect overlap, unreachable rules, and gaps;
//! - emit bytecode against the spec in `docs/dmn-lite-bytecode.md`.
//!
//! Phase 1.0 status: skeleton only.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use dmn_lite_parser::Source;
use dmn_lite_types::{CompileError, CompiledDecision};

/// Compile a parsed [`Source`] into an executable [`CompiledDecision`] artifact.
///
/// The compiled artifact contains no unresolved symbols: every input reference
/// is a `FieldId` and every enum value is a resolved entity ID (UUIDv7 into
/// the Sem OS catalogue). See `docs/dmn-lite-bytecode.md` for the artifact
/// contract and `docs/dmn-lite-semantics.md` for the compilation rules.
///
/// Phase 1.0: returns `unimplemented!()`. Real implementation in Phases 1.2–1.4.
pub fn compile(_source: Source) -> Result<CompiledDecision, CompileError> {
    unimplemented!("dmn-lite-compiler: compile() not implemented until Phase 1.2")
}
