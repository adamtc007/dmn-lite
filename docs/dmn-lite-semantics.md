# dmn-lite Semantics Specification

| Field | Value |
| --- | --- |
| Document | dmn-lite-semantics |
| Version | v0.1 |
| Scope | Profile v0.1 — static decision-table core |
| Status | Specification (binding for Phase 1.2–1.3 implementation) |

---

## 1. Framing

This document specifies the static and dynamic semantics of the dmn-lite s-expression DSL for Profile v0.1. It is the implementation contract for the compiler (static semantics) and the reference evaluator and stack VM (dynamic semantics).

The guiding principle from V&S §8.1:

> **Authoring captures meaning. The compiler derives execution strategy. The runtime executes a boring, compiled artifact.**

The compiler's job is to reject anything it cannot compile safely and to produce an artifact whose evaluation is deterministic, bounded, and auditable. The runtime's job is to execute that artifact faithfully.

---

## 2. Static semantics

Static semantics are checked by the compiler during the compilation passes described in §4. A decision that fails any static check must be rejected with a precise diagnostic. **Unsupported constructs are first-class diagnostics, not silently ignored** (V&S §8.3).

### 2.1 Name resolution rules

Every symbol appearing in a predicate or assignment is either a **late-bound** or **early-bound** symbol.

**Late-bound symbols** are input field references. They resolve at evaluation time against the typed input context.

Resolution rule: a symbol used as the left-hand side of a predicate (the field reference position) must exactly match a symbol declared in the `:inputs` block of the same decision. Matching is case-sensitive. Unresolved field references are a compile error.

**Early-bound symbols** are enum value references, domain identifiers, and decision IDs. They resolve at compile time against the Sem OS catalogue snapshot in scope at compilation.

Resolution rule: every enum value literal used in a predicate or assignment must belong to the domain declared for the corresponding input or output field. The compiler validates this against the Sem OS snapshot. Unresolved or out-of-domain enum values are a compile error.

In Phase 1.0–1.2, Sem OS integration is not yet implemented. The compiler validates domains locally against the declared `:domain` reference. Full Sem OS catalogue resolution is a Phase 1.2 completion milestone.

### 2.2 Type checking rules

| Predicate form | Permitted input types |
| --- | --- |
| `=`, `!=` | `enum`, `bool`, `integer`, `decimal`, `string` |
| `in (...)` (set membership) | Any; elements must match the field's declared type |
| `<`, `<=`, `>`, `>=` (comparison) | `integer`, `decimal` only |
| `in [a .. b]` (range) | `integer`, `decimal` only |
| `is-null`, `is-not-null` | Any |
| `and`, `or`, `not` | Boolean-valued sub-predicates |

Type errors (e.g., a comparison predicate applied to an `enum` field) are compile errors.

### 2.3 Domain checking rules

For `enum`-typed fields, every literal used in an equality, inequality, or set-membership predicate must be a member of the domain declared for that field.

For `bool`-typed fields, the only valid literals are `true` and `false`.

For `integer` and `decimal` fields, literals are validated as the correct numeric type. An integer literal used against a `decimal` field is accepted (widened). A decimal literal used against an `integer` field is a compile error.

For `string`-typed fields, any string literal is accepted. Domains for string fields are advisory in Phase v0.1 and are not enumeration-validated by the compiler.

### 2.4 Predicate well-formedness rules

1. A predicate-block may not mix a wildcard (`*`) with other predicates. A block is either `(*)` (catch-all) or a list of one or more non-wildcard predicates.

2. `and` and `or` must contain at least two sub-predicates. `not` must contain exactly one sub-predicate.

3. Set membership (`in (...)`) must contain at least one literal. An empty set is a compile error.

4. In a range predicate, if both bounds are specified (not `*`), the lower bound must be less than or equal to the upper bound (for inclusive bounds) or strictly less than (for exclusive-on-both-sides bounds). An empty range (e.g., `[5 .. 3]`) is a compile error.

5. The `:then` assignment block must assign every declared output exactly once. Missing output assignments are a compile error. Duplicate output assignments (assigning the same output field twice in one rule) are a compile error.

6. Every output assignment must reference a symbol declared in the `:outputs` block.

### 2.5 Hit-policy constraints

**`UNIQUE`:** The decision is well-formed if the rules are mutually exclusive. The compiler should attempt to statically verify exclusivity for finite-domain (enum) inputs. If static verification is not possible, the compiler emits a warning and the runtime enforces uniqueness dynamically, returning an `EvalError` if multiple rules match.

**`FIRST`:** No static constraint on rule overlap. Rules are evaluated in source order; the first matching rule wins. Rule order is semantically meaningful. The compiler must preserve source rule order in the emitted bytecode.

### 2.6 Catch-all rule constraints

A catch-all rule (`:when (*)`) is always-true. It satisfies the following constraints:

1. There must be at most one catch-all rule per decision. Multiple catch-all rules are a compile error.

2. Under `FIRST` hit policy, the catch-all rule must be the **last** rule in the `:rules` block. A catch-all rule appearing before a normal rule under `FIRST` would make all subsequent rules unreachable, which is a compile error.

3. Under `UNIQUE` hit policy, the catch-all rule may appear in any position, but the compiler must verify (or flag) that all other rules are mutually exclusive with each other. If non-catch-all rules can overlap, and the catch-all can never be reached, the compiler may warn about unreachable rules.

### 2.7 Symbol resolution and artifact hash

The compiler performs two distinct passes (V&S §11.5):

1. **Late-bound resolution pass**: validate every field reference exists in `:inputs`; record its `FieldId` (ordinal index into the input schema).

2. **Early-bound resolution pass**: for every literal symbol in a predicate or assignment, resolve it to a `ValueId` (entity ID from the Sem OS catalogue, or a locally scoped numeric/string constant). Validate domain membership.

The **artifact hash** is computed over `normalised_source + resolved_entity_ids + compiled_ir`. Two compilations of the same source with the same resolved entity IDs must produce the same artifact hash. This hash is the load-bearing invariant for long-running process replay (V&S §3.4 and §11.5).

---

## 3. Dynamic semantics (evaluation)

### 3.1 Input/output binding

At evaluation time, the caller supplies a `TypedInputContext` — a map from `FieldId` to `TypedValue`. Every declared input field must be present. Missing required inputs produce an `EvalError` with an `InputMissing` variant (Phase 1.4).

After evaluation, the engine returns a `TypedOutputContext` — a map from output `FieldId` to `TypedValue`. The shape is determined by the matched rule's `:then` block.

### 3.2 Predicate evaluation

Predicates are evaluated against the current typed input context. All predicates are **pure**: they have no side effects, no I/O, and no mutation of state.

| Predicate | Evaluation |
| --- | --- |
| `(f = v)` | `input[f] == v` |
| `(f != v)` | `input[f] != v` |
| `(f in (v1 v2 ...))` | `input[f] ∈ {v1, v2, ...}` |
| `(f < n)` | `input[f] < n` (ordered types) |
| `(f <= n)` | `input[f] <= n` |
| `(f > n)` | `input[f] > n` |
| `(f >= n)` | `input[f] >= n` |
| `(f in [a .. b])` | `a <= input[f] <= b` (brackets control inclusivity) |
| `(f in [* .. b])` | `input[f] <= b` (unbounded lower) |
| `(f in [a .. *])` | `input[f] >= a` (unbounded upper) |
| `(f is-null)` | `input[f]` is absent or null |
| `(f is-not-null)` | `input[f]` is present and non-null |
| `(not p)` | `! evaluate(p)` |
| `(and p1 p2 ...)` | `evaluate(p1) && evaluate(p2) && ...` (short-circuit) |
| `(or p1 p2 ...)` | `evaluate(p1) \|\| evaluate(p2) \|\| ...` (short-circuit) |
| `:when (*)` | always `true` |

**Implicit conjunction:** Multiple predicates in a `:when` predicate-block are implicitly AND'd. Evaluation short-circuits: if any predicate in the block evaluates to false, remaining predicates in the same block are not evaluated.

### 3.3 Hit-policy application

**`UNIQUE`:**
1. Evaluate all rules in source order.
2. Collect all matching rules.
3. If zero rules match: return `NoMatch`.
4. If exactly one rule matches: return `Result` with that rule's output bindings.
5. If more than one rule matches: return an `EvalError` with a `HitPolicyViolation(UNIQUE)` variant (unless the compiler statically proved this impossible).

**`FIRST`:**
1. Evaluate rules in source order.
2. Return `Result` with the first matching rule's output bindings.
3. Do not evaluate any subsequent rules after the first match.
4. If no rule matches (and no catch-all is present): return `NoMatch`.

A catch-all rule (`:when (*)`) always matches and is evaluated last under `FIRST`. It cannot be preceded by a rule that always matches.

### 3.4 Output collection

When a rule matches, the engine executes the `:then` assignment block. Each assignment `(output-field = value)` stores the resolved value into the output context under the field's `FieldId`. The result is a `TypedOutputContext` with one entry per declared output.

### 3.5 Bounded computation guarantees (V&S §8.4)

Profile v0.1 decisions are unconditionally bounded:

- Every decision is a finite list of rules.
- Every rule is a finite conjunction of predicates.
- No recursion, no iteration, no arbitrary function calls.
- Worst-case evaluation cost: `|rules| × |max predicates per rule|` predicate evaluations.
- The compiler computes this bound and may surface it as metadata in the compiled artifact.

Short-circuit evaluation means best-case cost is often much lower. The compiler emits `BrFalse` instructions after each predicate to exploit this.

---

## 4. Compilation passes

The compiler executes the following passes in order:

| Pass | Responsibility |
| --- | --- |
| 1. Parse | Source text → AST (Phase 1.1, `dmn-lite-parser`) |
| 2. Late-bound resolution | Validate field references; assign `FieldId`s |
| 3. Early-bound resolution | Resolve enum values to entity IDs; validate domains |
| 4. Type/domain checking | Validate predicate types against input declarations |
| 5. Semantic validation | Hit-policy constraints, catch-all rules, assignment completeness |
| 6. IR lowering | AST → typed predicate IR (`Predicate` tree) |
| 7. Decision planning | Predicate IR → ordered execution plan (Phase v0.1: source order with short-circuit) |
| 8. Bytecode emission | Execution plan → `Vec<Instr>` with const pool and source map |

Passes 2–8 are the responsibility of `dmn-lite-compiler`. The typed predicate IR from Pass 6 is also stored in the `CompiledDecision` artifact for the reference evaluator oracle.

---

## 5. Edge cases and error conditions

| Condition | Behaviour |
| --- | --- |
| Missing required input at evaluation time | `EvalError::InputMissing(FieldId)` |
| Input value not in declared domain at evaluation time | `EvalError::InputDomainViolation(FieldId, value)` |
| Multiple matches under `UNIQUE` | `EvalError::HitPolicyViolation(UNIQUE, vec![RuleId])` |
| No match, no catch-all | Returns `DecisionOutcome::NoMatch` (not an error) |
| Empty predicate block | Compile error: predicate block must contain at least one predicate |
| Null value for non-nullable field | `EvalError::InputMissing` (treated as absent); `is-null` predicate returns true |
| Boolean `true`/`false` used as enum literals | Compile error: `bool`-typed fields require `true`/`false` literals, not `TRUE`/`FALSE` symbols |
| Integer literal against `decimal` field | Accepted; the integer is widened to decimal at compile time |
| Decimal literal against `integer` field | Compile error |
| Empty `:inputs` block | Compile error: at least one input is required |
| Empty `:outputs` block | Compile error: at least one output is required |
| Empty `:rules` block | Compile warning: a decision with no rules always returns `NoMatch` |
