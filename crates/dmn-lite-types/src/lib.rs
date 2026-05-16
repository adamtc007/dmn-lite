//! dmn-lite core types.
//!
//! This crate owns the vocabulary shared between parser, compiler, analysis,
//! and engine. It contains no behaviour beyond simple constructors and
//! accessors. All semantic logic lives in the consuming crates.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod compiled;
pub mod errors;
pub mod hit_policy;
pub mod ids;
pub mod instr;
pub mod predicates;
pub mod values;

pub use compiled::{AnalysisReport, CompiledDecision};
pub use errors::{CompileError, EvalError, ParseError};
pub use ids::{NumberKind, SourceSpan};
pub use values::{TypedInputContext, TypedOutputContext};
