//! Value types: `TypedValue`, `TypedInputContext`, `TypedOutputContext`.
//!
//! Phase 1.0 status: placeholder structs only. Full type vocabulary added in Phase 1.2.

/// Typed input bindings supplied to the engine at evaluation time.
///
/// Maps each declared input field to a resolved `TypedValue`. The engine
/// reads field values from this context during bytecode execution.
///
/// Phase 1.0: placeholder. Fields added in Phase 1.2.
pub struct TypedInputContext {
    _private: (),
}

/// Typed output bindings produced by the engine after evaluation.
///
/// Contains the output field values assigned by the matched rule(s),
/// shaped according to the decision's output schema.
///
/// Phase 1.0: placeholder. Fields added in Phase 1.2.
pub struct TypedOutputContext {
    _private: (),
}
