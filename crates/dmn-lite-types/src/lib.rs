//! dmn-lite core types.
//!
//! This crate owns the vocabulary shared between parser, compiler, analysis,
//! and engine. It contains no behaviour beyond simple constructors and
//! accessors. All semantic logic lives in the consuming crates.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod catalogue;
pub mod compiled;
pub mod errors;
pub mod hit_policy;
pub mod ids;
pub mod instr;
pub mod ir;
pub mod predicates;
pub mod values;

pub use catalogue::{Catalogue, Domain, DomainValue};
pub use compiled::{AnalysisReport, CompiledDecision};
pub use errors::{CatalogueError, CompileError, CompileWarning, EvalError, ParseError};
pub use ids::{DecisionId, DomainId, FieldId, NumberKind, RuleId, SnapshotId, SourceSpan, ValueId};
pub use values::{TypedInputContext, TypedOutputContext};
