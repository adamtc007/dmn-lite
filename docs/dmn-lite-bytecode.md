# dmn-lite Bytecode Specification

| Field | Value |
| --- | --- |
| Document | dmn-lite-bytecode |
| Version | v0.1 |
| Scope | Profile v0.1 — static decision-table core |
| Status | Specification (binding for Phase 1.4 compiler emission and VM implementation) |

---

## 1. Framing

This document is the **contract between `dmn-lite-compiler` (emitter) and `dmn-lite-engine` (executor)**. Both sides must hold to it. Neither crate may deviate from the stack discipline or branch semantics below without a versioned amendment to this document.

The bytecode is not an optimisation target in Phase v0.1. It is a faithful, bounded, deterministic executable representation of a compiled decision. The reference evaluator in `dmn-lite-engine::reference` is an alternative oracle over the same semantic model; the VM and reference evaluator must produce identical results for all well-formed inputs.

---

## 2. Machine architecture

The dmn-lite stack VM has the following components:

```text
CompiledDecision
  program:    Vec<Instr>      — immutable instruction stream
  const_pool: ConstPool       — early-bound constant values
  set_pool:   ConstSetPool    — precompiled constant sets
  range_pool: RangePool       — precompiled range bounds
  source_map: SourceMap       — instruction → source span + rule ID

Runtime machine (EvalMachine):
  pc:           usize                      — program counter
  data_stack:   SmallVec<[TypedValue; 32]> — operand stack
  return_stack: SmallVec<[ReturnFrame; 8]> — control-flow stack (unused in v0.1)
  input_frame:  &TypedInputContext         — late-bound decision inputs (read-only)
  output_frame: TypedOutputContext         — output accumulator
  accumulator:  RuleMatchAccumulator       — matched rules + hit-policy state
```

**Data stack:** Holds `TypedValue` operands. Most instructions pop their inputs from and push their result onto the data stack. The stack is bounded by the decision's static structure: the compiler must verify at emit time that no execution path produces unbounded stack growth. In Phase v0.1, the maximum stack depth is `max(predicates per rule + 2)`.

**Return stack:** Holds return addresses for governed function calls and quantifier frames. In Phase v0.1, no instruction uses the return stack. It is declared but always empty.

**Input frame:** A read-only typed binding from `FieldId` to `TypedValue`. Populated by the caller before evaluation begins. `LoadField` reads from this frame.

**Output frame:** An initially empty map from output `FieldId` to `TypedValue`. `StoreOutput` writes into this frame. After `EndDecision`, this frame contains the final output context.

**Accumulator:** Records which rules matched, for hit-policy state machine. Consulted at `EndDecision` to validate hit-policy constraints and determine the final `DecisionOutcome`.

---

## 3. Instruction set (Phase v0.1)

### 3.1 Instruction summary table

| Instruction | Stack before | Stack after | Notes |
| --- | --- | --- | --- |
| `LoadField(FieldId)` | `[...]` | `[..., value]` | Push input field value |
| `PushConst(ConstId)` | `[...]` | `[..., value]` | Push constant from pool |
| `PushConstSet(ConstSetId)` | `[...]` | `[..., set]` | Push precompiled set |
| `Eq` | `[..., a, b]` | `[..., bool]` | `a == b` |
| `NotEq` | `[..., a, b]` | `[..., bool]` | `a != b` |
| `InSet` | `[..., value, set]` | `[..., bool]` | `value ∈ set` |
| `RangeCheck(RangeId)` | `[..., value]` | `[..., bool]` | Range membership test |
| `IsNull` | `[..., value]` | `[..., bool]` | `value is null/absent` |
| `IsNotNull` | `[..., value]` | `[..., bool]` | `value is present` |
| `And` | `[..., a, b]` | `[..., bool]` | `a && b` (both already evaluated) |
| `Or` | `[..., a, b]` | `[..., bool]` | `a \|\| b` (both already evaluated) |
| `Not` | `[..., a]` | `[..., bool]` | `!a` |
| `Br(Addr)` | `[...]` | `[...]` | Unconditional jump |
| `BrFalse(Addr)` | `[..., bool]` | `[...]` | Pop; jump if false |
| `StoreOutput(OutputFieldId)` | `[..., value]` | `[...]` | Pop; write to output frame |
| `RuleMatched(RuleId)` | `[...]` | `[...]` | Record match in accumulator |
| `EndDecision` | `[...]` | `[...]` | Apply hit policy; terminate |

### 3.2 Instruction definitions

---

#### `LoadField(FieldId)`

Reads the value of the input field identified by `FieldId` from the current input frame and pushes it onto the data stack.

If the field is absent from the input frame, the runtime raises `EvalError::InputMissing(FieldId)` immediately (no partial stack state left).

Stack effect: `[] → [value]`

---

#### `PushConst(ConstId)`

Pushes the `TypedValue` identified by `ConstId` from the const pool onto the data stack. `ConstId` is an index into `CompiledDecision.const_pool`.

Constants in the pool are all early-bound: enum entity IDs, boolean literals, integer literals, decimal literals, string literals. No const ID is ever out-of-bounds in a well-formed artifact.

Stack effect: `[] → [value]`

---

#### `PushConstSet(ConstSetId)`

Pushes a precompiled constant set (a `HashSet<TypedValue>`) from the set pool onto the data stack. Used as the set operand for `InSet`.

Stack effect: `[] → [set]`

---

#### `Eq`

Pops two values `a` (top) and `b` (below top). Pushes `true` if `a == b` under typed equality, `false` otherwise.

Typed equality rules:
- `enum == enum`: identity comparison on resolved entity IDs.
- `bool == bool`: value comparison.
- `integer == integer`: numeric equality.
- `decimal == decimal`: numeric equality (exact decimal, no floating-point rounding).
- `string == string`: Unicode code-point equality (NFC-normalised at compile time).
- Comparing values of different types is always `false` (no implicit coercion at runtime).

Stack effect: `[a, b] → [bool]`

---

#### `NotEq`

Identical to `Eq` but pushes `!(a == b)`.

Stack effect: `[a, b] → [bool]`

---

#### `InSet`

Pops a set `s` (top) and a value `v` (below top). Pushes `true` if `v ∈ s`, `false` otherwise. Membership is tested using typed equality (same rules as `Eq`).

The set `s` is always a `PushConstSet` result; it is never constructed dynamically at runtime.

Stack effect: `[v, s] → [bool]`

---

#### `RangeCheck(RangeId)`

Pops a value `v` from the top of the data stack. Looks up the precompiled range identified by `RangeId` from the range pool. Pushes `true` if `v` satisfies the range, `false` otherwise.

Range pool entry format (from compiled artifact):
```text
RangeEntry {
    lower_bound:       Option<TypedValue>,  // None = unbounded
    lower_inclusive:   bool,
    upper_bound:       Option<TypedValue>,  // None = unbounded
    upper_inclusive:   bool,
}
```

Evaluation:
- `lower_bound.is_none() || (lower_inclusive ? v >= lb : v > lb)`
  AND
- `upper_bound.is_none() || (upper_inclusive ? v <= ub : v < ub)`

Range checks over `integer` and `decimal` types only. The compiler must not emit `RangeCheck` for non-ordered types.

Stack effect: `[v] → [bool]`

---

#### `IsNull`

Pops a value from the data stack. Pushes `true` if the value is the null/absent sentinel, `false` otherwise.

Stack effect: `[v] → [bool]`

---

#### `IsNotNull`

Pops a value from the data stack. Pushes `true` if the value is present (non-null), `false` otherwise. Equivalent to `IsNull` followed by `Not`.

Stack effect: `[v] → [bool]`

---

#### `And`

Pops two boolean values `a` (top) and `b` (below top). Pushes `a && b`.

**Note:** This instruction evaluates both operands. Short-circuit AND is achieved at the predicate-block level using `BrFalse` instructions, not via this instruction. `And` is used when both sub-predicates are unconditionally needed (e.g., inside an explicit `(and p1 p2)` form where short-circuit is not required by the semantics).

Stack effect: `[a, b] → [bool]`

---

#### `Or`

Pops two boolean values `a` (top) and `b` (below top). Pushes `a || b`.

Stack effect: `[a, b] → [bool]`

---

#### `Not`

Pops one boolean value `a`. Pushes `!a`.

Stack effect: `[a] → [bool]`

---

#### `Br(Addr)`

Unconditional jump. Sets `pc = Addr`. Does not affect the data stack.

`Addr` is an absolute index into `CompiledDecision.program`. It must be a valid instruction index in a well-formed artifact.

Stack effect: none

---

#### `BrFalse(Addr)`

Pops the top boolean value from the data stack. If the value is `false`, jumps to `Addr`. If the value is `true`, execution continues at the next instruction.

This is the primary short-circuit instruction. A predicate-block of `N` predicates emits `N` pairs of `(evaluate predicate; BrFalse next_rule_addr)`.

Stack effect: `[bool] → []`

---

#### `StoreOutput(OutputFieldId)`

Pops the top value from the data stack and stores it in the output frame under `OutputFieldId`. `OutputFieldId` is a compile-time resolved index into the output schema.

A well-formed artifact stores each output field exactly once on any execution path that reaches `EndDecision` via a rule match. Storing the same output field twice under a single rule is a compiler validation error (caught at emit time).

Stack effect: `[value] → []`

---

#### `RuleMatched(RuleId)`

Records the match of rule `RuleId` in the result accumulator. Does not affect the data stack.

`RuleId` is used by the hit-policy state machine:
- Under `FIRST`: the accumulator notes the match; `EndDecision` will return this result and ignore subsequent rules.
- Under `UNIQUE`: the accumulator notes the match; if a second `RuleMatched` fires before `EndDecision`, a hit-policy violation is recorded.

`RuleMatched` must have an associated `SourceSpan` in the source map (required for hit-policy explanations and audit).

Stack effect: none

---

#### `EndDecision`

Terminates execution. Applies the hit-policy state machine to the accumulator and produces the final `DecisionOutcome`:

- `UNIQUE` with zero matches: `DecisionOutcome::NoMatch`.
- `UNIQUE` with exactly one match: `DecisionOutcome::Result(output_frame)`.
- `UNIQUE` with multiple matches: `DecisionOutcome::Incident(HitPolicyViolation)`.
- `FIRST` with at least one match: `DecisionOutcome::Result(output_frame)` (first match already recorded by `BrFalse` short-circuit).
- `FIRST` with zero matches: `DecisionOutcome::NoMatch`.

Every reachable execution path must end in `EndDecision`. The verifier checks this invariant.

Stack effect: none (terminates execution)

---

## 4. Branch semantics and code layout

### 4.1 Addresses

All branch addresses are **absolute** indices into `CompiledDecision.program`. A branch to index `i` means the next instruction executed is `program[i]`. Address 0 is the first instruction of the program.

### 4.2 Typical rule emission pattern

For a rule with `N` predicates under `FIRST` hit policy:

```text
; Rule rXXX
LOAD_FIELD f1
PUSH_CONST v1
EQ
BR_FALSE  <next_rule_start>   ; short-circuit: skip rule if pred 1 fails

LOAD_FIELD f2
PUSH_CONST v2
EQ
BR_FALSE  <next_rule_start>   ; short-circuit: skip rule if pred 2 fails

... (repeat for each predicate) ...

; All predicates passed — execute :then block
PUSH_CONST output_v1
STORE_OUTPUT out_field1
PUSH_CONST output_v2
STORE_OUTPUT out_field2
RULE_MATCHED rXXX
BR <end_decision>             ; FIRST: jump to EndDecision immediately

<next_rule_start>:
... next rule ...

<end_decision>:
END_DECISION
```

Under `UNIQUE` hit policy, the `BR <end_decision>` after `RULE_MATCHED` is omitted: execution continues to evaluate the remaining rules to detect violations.

### 4.3 Catch-all rule

A catch-all rule emits no predicate instructions. It emits only the `:then` assignment block followed by `RuleMatched`:

```text
; Catch-all rule r999
PUSH_CONST NOT_ELIGIBLE
STORE_OUTPUT eligibility
PUSH_CONST NO_MATCH
STORE_OUTPUT reason_code
RULE_MATCHED r999
```

---

## 5. Pools

### 5.1 Const pool

`ConstPool` is an indexed array of `TypedValue` entries. `ConstId` is a `u32` index. All early-bound constants (enum entity IDs, numeric literals, boolean literals, string literals) are deduplicated and stored in the const pool at compile time.

### 5.2 Const-set pool

`ConstSetPool` is an indexed array of `HashSet<TypedValue>`. `ConstSetId` is a `u32` index. A const set is emitted once per `in (...)` predicate at compile time. If two rules use the same literal set, they share a single pool entry (deduplication is an optimisation but not required by the spec).

### 5.3 Range pool

`RangePool` is an indexed array of range entries. `RangeId` is a `u32` index. Each range entry records lower/upper bounds, their types, and inclusivity flags.

---

## 6. Source mapping

Every instruction in the program has an optional `SourceSpan` entry in `SourceMap`. The `SourceMap` is an indexed map from instruction index to `Option<SourceSpan>`.

`SourceSpan` records:
- `rule_id: RuleId` — which rule this instruction belongs to.
- `start: u32`, `end: u32` — byte offsets into the original source string.
- `predicate_index: Option<u32>` — which predicate within the rule (for diagnostics).

`RuleMatched(RuleId)` instructions must have a `SourceSpan`. This is required for audit trails and hit-policy explanation output.

---

## 7. Well-formedness invariants (for the Phase 1.4 verifier)

A `CompiledDecision` is well-formed if and only if:

1. **No out-of-bounds branches.** Every `Br(addr)` and `BrFalse(addr)` has `addr < program.len()`. Branch to `program.len()` is not valid; use `EndDecision` to terminate.

2. **No stack underflow on any reachable path.** For every instruction on every reachable execution path, the data stack has at least as many values as the instruction requires. The compiler must verify this at emit time using a static stack-depth analysis.

3. **Every reachable path ends in `EndDecision`.** There is no "fall off the end" of the program. The last instruction of every possible execution path must be `EndDecision`. The verifier checks this using a reachability analysis.

4. **No out-of-bounds pool references.** Every `ConstId`, `ConstSetId`, `RangeId`, `FieldId`, and `OutputFieldId` referenced in any instruction is a valid index into the corresponding pool or schema.

5. **Output frame completeness.** On every execution path that reaches `EndDecision` through a `RuleMatched` instruction, every declared output field has been stored exactly once via `StoreOutput`. Paths that reach `EndDecision` without a `RuleMatched` (the no-match path) leave the output frame empty.

The Phase 1.4 verifier validates these invariants on every `CompiledDecision` before it is published. A verification failure is an internal compiler error (not a user diagnostic).
