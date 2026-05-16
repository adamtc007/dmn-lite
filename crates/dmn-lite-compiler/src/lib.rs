//! dmn-lite compiler: `Source` AST → `TypedDecision` typed predicate IR.
//!
//! # Usage
//!
//! Load a catalogue, parse source, and compile:
//!
//! ```rust,ignore
//! use dmn_lite_compiler::{compile, load_catalogue_from_path};
//! use dmn_lite_parser::parse;
//!
//! let catalogue = load_catalogue_from_path("test-data/sem-os-stub.toml".as_ref())
//!     .expect("stub catalogue must load");
//! let source = parse("...").expect("source must parse");
//! match compile(source, &catalogue) {
//!     Ok(decision) => println!("compiled: {}", decision.name),
//!     Err(errs) => println!("{} compile errors", errs.errors.len()),
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod catalogue_loader;
mod lower;

pub use catalogue_loader::{load_catalogue_from_path, load_catalogue_from_str};
pub use dmn_lite_types::ir::TypedDecision;
pub use dmn_lite_types::{Catalogue, CatalogueError, CompileError, CompileWarning};

use dmn_lite_parser::Source;
use std::fmt;

/// A collection of compilation errors and warnings from a single compile call.
///
/// Like `ParseErrors`, this struct supports partial output: `partial_decision`
/// is `Some` when input/output schemas resolved successfully, even if some
/// rules failed type-checking.
pub struct CompileErrors {
    /// All compile errors encountered during this compile pass.
    pub errors: Vec<CompileError>,
    /// Non-fatal diagnostics (e.g., advisory domain on non-enum field).
    pub warnings: Vec<CompileWarning>,
    /// Partially-compiled decision, if schemas resolved despite rule errors.
    pub partial_decision: Option<TypedDecision>,
}

impl fmt::Display for CompileErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for CompileErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompileErrors")
            .field("errors", &self.errors)
            .field("warnings", &self.warnings)
            .field(
                "partial_decision",
                &self.partial_decision.as_ref().map(|_| "<TypedDecision>"),
            )
            .finish()
    }
}

impl std::error::Error for CompileErrors {}

/// Compile a parsed [`Source`] against a [`Catalogue`] into a [`TypedDecision`].
///
/// Returns `Ok(TypedDecision)` on success (warnings may still be present — use
/// `compile_with_warnings` to capture them). Returns `Err(CompileErrors)` when
/// one or more static-semantic checks fail.
///
/// The input `source` must contain exactly one decision (enforced by the parser
/// in Phase 1.1a).
#[allow(clippy::result_large_err)]
pub fn compile(source: Source, catalogue: &Catalogue) -> Result<TypedDecision, CompileErrors> {
    let result = compile_with_warnings(source, catalogue);
    if result.errors.is_empty() {
        Ok(result
            .partial_decision
            .expect("no errors → must have decision"))
    } else {
        Err(result)
    }
}

/// Compile a parsed [`Source`] and return all diagnostics including warnings.
///
/// Unlike [`compile`], this always returns the full diagnostic picture.
/// `partial_decision` is `Some` if schemas resolved, even when errors exist.
pub fn compile_with_warnings(source: Source, catalogue: &Catalogue) -> CompileErrors {
    let decision_ast = match source.decisions.into_iter().next() {
        Some(d) => d,
        None => {
            // Empty source — should not happen after Phase 1.1a arity enforcement.
            return CompileErrors {
                errors: vec![CompileError::EmptyInputs { span: source.span }],
                warnings: Vec::new(),
                partial_decision: None,
            };
        }
    };

    let result = lower::lower(&decision_ast, catalogue);

    // If there are errors but we have a partial decision, respect both.
    CompileErrors {
        errors: result.errors,
        warnings: result.warnings,
        partial_decision: result.decision,
    }
}
