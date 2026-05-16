//! Bytecode instruction set: `Instr`, address and pool ID types.
//!
//! `Instr` is the token type for the compiled decision program. The compiler
//! emits a `Vec<Instr>` and the stack VM executes it. The instruction set is
//! specified in `docs/dmn-lite-bytecode.md`.
//!
//! Phase v0.1 instruction set (to be added in Phase 1.4):
//! `LoadField`, `PushConst`, `PushConstSet`, `Eq`, `NotEq`, `InSet`,
//! `RangeCheck`, `IsNull`, `IsNotNull`, `And`, `Or`, `Not`,
//! `Br`, `BrFalse`, `StoreOutput`, `RuleMatched`, `EndDecision`.
//!
//! Phase 1.0 status: empty enum. Variants defined in Phase 1.4 against the
//! bytecode specification in `docs/dmn-lite-bytecode.md`.

/// Bytecode instruction executed by the dmn-lite stack VM.
///
/// Each instruction operates on the data stack and/or the typed input frame.
/// The full stack discipline for each instruction is specified in
/// `docs/dmn-lite-bytecode.md`.
///
/// Phase 1.0: no variants. Variants added in Phase 1.4.
pub enum Instr {}
