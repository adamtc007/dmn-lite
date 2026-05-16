# dmn-lite EBNF Grammar Specification

| Field | Value |
| --- | --- |
| Document | dmn-lite-ebnf |
| Version | v0.1 |
| Scope | Profile v0.1 — static decision-table core |
| Status | Specification (binding for Phase 1.1 parser implementation) |

---

## 1. Framing

This document specifies the formal grammar for the dmn-lite s-expression DSL. It is the implementation contract for `dmn-lite-parser`. The grammar covers the constructs required by Profile v0.1 (static decision-table core) as defined in V&S §17. Constructs outside v0.1 scope are listed in §6 as future extensions.

The language is an s-expression DSL. All structure is expressed through nested parenthesised forms. There is no significant whitespace: any amount of whitespace (spaces, tabs, newlines) may appear between tokens. Line comments beginning with `;` are permitted anywhere whitespace is permitted.

---

## 2. Grammar

EBNF notation used below:

- `::=` defines a production
- `|` is alternation
- `*` is zero or more
- `+` is one or more
- `?` is optional
- `"..."` is a literal token
- `[...]` is a character class
- `(*...*)`  is a comment

```ebnf
(* ============================================================ *)
(* Top-level form                                               *)
(* ============================================================ *)

source-file     ::= ws* decision ws*

decision        ::= "(" "define-decision" ws+ symbol ws+ decision-attrs ")"

decision-attrs  ::= decision-id?
                    hit-policy-attr
                    input-block
                    output-block
                    rule-block

(* ============================================================ *)
(* Decision-level attributes                                    *)
(* ============================================================ *)

decision-id     ::= ":decision-id" ws+ string-literal ws+

hit-policy-attr ::= ":hit-policy" ws+ hit-policy-kind ws+
hit-policy-kind ::= "unique" | "first"

(* ============================================================ *)
(* Input and output declarations                                *)
(* ============================================================ *)

input-block     ::= ":inputs" ws+ "(" ws* input-decl* ")" ws+
output-block    ::= ":outputs" ws+ "(" ws* output-decl* ")" ws+

input-decl      ::= "(" ws* symbol ws+ ":type" ws+ type-ref ws+ ":domain" ws+ domain-ref ws* ")" ws*
output-decl     ::= "(" ws* symbol ws+ ":type" ws+ type-ref ws+ ":domain" ws+ domain-ref ws* ")" ws*

type-ref        ::= "enum"
                  | "bool"
                  | "integer"
                  | "decimal"
                  | "string"

domain-ref      ::= symbol   (* references a Sem OS governed domain *)

(* ============================================================ *)
(* Rule block                                                   *)
(* ============================================================ *)

rule-block      ::= ":rules" ws+ "(" ws* rule* ")"

rule            ::= normal-rule | catch-all-rule

normal-rule     ::= "(" ws* "rule" ws+ symbol ws+
                    ":when" ws+ predicate-block ws+
                    ":then" ws+ assignment-block ws*
                    ")" ws*

catch-all-rule  ::= "(" ws* "rule" ws+ symbol ws+
                    ":when" ws+ "(" ws* "*" ws* ")" ws+
                    ":then" ws+ assignment-block ws*
                    ")" ws*

(* ============================================================ *)
(* Predicate block                                              *)
(* ============================================================ *)

predicate-block ::= "(" ws* predicate+ ws* ")"

(* All predicates in a predicate-block are implicitly conjoined (AND). *)
(* A block containing a single wildcard is the only valid wildcard form. *)

predicate       ::= equality-pred
                  | inequality-pred
                  | set-membership-pred
                  | range-pred
                  | comparison-pred
                  | null-test-pred
                  | not-pred
                  | and-pred
                  | or-pred

(* --- Equality -------------------------------------------- *)

equality-pred   ::= "(" ws* symbol ws+ "=" ws+ literal ws* ")"

(* --- Inequality ------------------------------------------ *)

inequality-pred ::= "(" ws* symbol ws+ "!=" ws+ literal ws* ")"

(* --- Set membership -------------------------------------- *)

set-membership-pred ::= "(" ws* symbol ws+ "in" ws+ "(" ws* literal+ ws* ")" ws* ")"

(* --- Comparison (ordered types only) --------------------- *)

comparison-pred ::= "(" ws* symbol ws+ comparison-op ws+ numeric-literal ws* ")"
comparison-op   ::= "<" | "<=" | ">" | ">="

(* --- Range test (ordered types only) --------------------- *)
(*                                                            *)
(* Syntax: (field in range-expr)                             *)
(*                                                            *)
(* range-expr forms:                                         *)
(*   [a .. b]   lower inclusive, upper inclusive             *)
(*   [a .. b)   lower inclusive, upper exclusive             *)
(*   (a .. b]   lower exclusive, upper inclusive             *)
(*   (a .. b)   lower exclusive, upper exclusive             *)
(*   [a .. *]   lower inclusive, upper unbounded             *)
(*   [* .. b]   lower unbounded, upper inclusive             *)
(*   [* .. *]   always-true range (valid but degenerate)     *)
(*                                                            *)
(* The bracket/paren on the LEFT denotes lower bound type.   *)
(* The bracket/paren on the RIGHT denotes upper bound type.  *)
(* '*' in a bound position means unbounded (open).           *)

range-pred      ::= "(" ws* symbol ws+ "in" ws+ range-expr ws* ")"

range-expr      ::= range-lower-delim ws* range-bound ws* ".." ws* range-bound ws* range-upper-delim

range-lower-delim ::= "[" | "("   (* "[" = inclusive, "(" = exclusive *)
range-upper-delim ::= "]" | ")"   (* "]" = inclusive, ")" = exclusive *)
range-bound       ::= numeric-literal | "*"

(* --- Null tests ------------------------------------------ *)

null-test-pred  ::= "(" ws* symbol ws+ "is-null" ws* ")"
                  | "(" ws* symbol ws+ "is-not-null" ws* ")"

(* --- Boolean combinators --------------------------------- *)

not-pred        ::= "(" ws* "not" ws+ predicate ws* ")"
and-pred        ::= "(" ws* "and" ws+ predicate ws+ predicate+ ws* ")"
or-pred         ::= "(" ws* "or" ws+ predicate ws+ predicate+ ws* ")"

(* ============================================================ *)
(* Assignment block                                             *)
(* ============================================================ *)

assignment-block ::= "(" ws* assignment+ ws* ")"

assignment      ::= "(" ws* symbol ws+ "=" ws+ literal ws* ")" ws*

(* ============================================================ *)
(* Literals                                                     *)
(* ============================================================ *)

literal         ::= symbol
                  | string-literal
                  | numeric-literal
                  | bool-literal

bool-literal    ::= "true" | "false"

numeric-literal ::= integer-literal | decimal-literal

integer-literal ::= "-"? digit+

decimal-literal ::= "-"? digit+ "." digit+

(* String literals use double-quote delimiters. The only escape *)
(* sequence recognised in Phase v0.1 is \" (escaped quote) and  *)
(* \\ (escaped backslash). Other backslash sequences are errors. *)

string-literal  ::= '"' string-char* '"'
string-char     ::= [^"\\]
                  | "\\" '"'
                  | "\\" "\\"

(* ============================================================ *)
(* Symbols and identifiers                                      *)
(* ============================================================ *)

(* Symbols are case-sensitive. Enum value symbols (e.g. LU, SICAV) are    *)
(* conventionally UPPER_CASE or PascalCase; field names (e.g. jurisdiction, *)
(* client-type) are conventionally lower-kebab-case. The grammar makes no  *)
(* distinction — both are symbols.                                          *)

symbol          ::= symbol-start symbol-continue*

symbol-start    ::= [a-zA-Z] | "_"

symbol-continue ::= [a-zA-Z] | [0-9] | "-" | "_" | "."

(* "." is permitted in symbols to support dotted decision IDs   *)
(* (e.g. "booking_eligibility.v1") and future path expressions. *)
(* Path expressions are a Profile v0.4 feature (§6).            *)

(* ============================================================ *)
(* Whitespace and comments                                      *)
(* ============================================================ *)

ws              ::= ( " " | "\t" | "\n" | "\r" | comment )*

(* Line comment: from ";" to end of line.                       *)
(* Block comments are not supported in Phase v0.1.              *)

comment         ::= ";" [^\n]* ( "\n" | EOF )
```

---

## 3. Phase v0.1 scope

### 3.1 In scope

| Construct | Notes |
| --- | --- |
| Decision tables | Single `define-decision` per source file |
| Typed inputs | `enum`, `bool`, `integer`, `decimal`, `string` types |
| Typed outputs | Same set of types; at least one output per decision |
| Hit policies | `unique` and `first` only |
| Equality predicates | `=` over enum, bool, integer, decimal, string |
| Inequality predicates | `!=` over same types |
| Set membership | `in (...)` over enum and other types |
| Comparison predicates | `<`, `<=`, `>`, `>=` over integer and decimal |
| Range predicates | `in [a .. b]` with inclusive/exclusive bounds and unbounded (`*`) |
| Null tests | `is-null`, `is-not-null` |
| Boolean combinators | `and`, `or`, `not` inside a predicate |
| Implicit conjunction | Multiple predicates in `:when (...)` are implicitly AND'd |
| Wildcard/catch-all | `:when (*)` — always-true rule |
| String literals | Double-quoted, with `\"` and `\\` escapes |
| Line comments | `;` to end of line |

### 3.2 Out of scope (deferred to later profiles)

| Construct | Profile | Notes |
| --- | --- | --- |
| `any`, `rule_order`, `collect` hit policies | v0.2 | Multi-hit semantics |
| Cross-field predicates (`EqField`, `CompareField`) | v0.2 | Field-to-field comparison |
| `ForAll` / `Exists` quantifiers | v0.2 | Over governed collections |
| Bounded aggregation | v0.2 | `Count`, `Sum`, `Min`, `Max`, `Mean` |
| DMN XML import | v0.3 | Parse and lower Camunda DMN tables |
| FEEL unary test subset | v0.3 | Approximately 15–20 grammar productions |
| Temporal predicates | v0.3 | `DateBefore`, `DateAfter`, `DateWithin`, `SnapshotAgeBelow` |
| DRD dependencies | v0.4 | Decision requirement graphs |
| Path expressions | v0.4 | `person.address.city`-style field navigation |
| Governed functions / BKMs | v0.5 | Compiled non-recursive reusable functions |
| Block comments | — | `; comment` is sufficient for v0.1 |
| Multi-decision source files | — | One decision per file is sufficient for v0.1 |

---

## 4. Resolved ambiguities

The V&S §12.1 provides an illustrative grammar sketch. The following points were ambiguous or unspecified and are resolved here for Phase 1.1 implementation:

**Whitespace**: Significant whitespace is not used. Any amount of whitespace (including newlines) may appear between any two tokens. A token is the smallest lexical unit: a keyword (`:inputs`, `rule`, `=`, etc.), a symbol, a numeric literal, a string literal, or a bracket/paren.

**Wildcard rule form**: The catch-all rule uses `:when (*)` — a predicate-block containing the single token `*`. The `*` token is only valid as the sole content of a predicate-block. A predicate-block containing `*` mixed with other predicates is a parse error.

**Attribute ordering in `decision-attrs`**: `:decision-id` is optional and may appear before or after `:hit-policy`. The ordering `hit-policy → inputs → outputs → rules` is required for `:hit-policy`, `:inputs`, `:outputs`, and `:rules`. Implementation may enforce strict ordering for simplicity.

**Symbol case sensitivity**: Symbols are case-sensitive. `LU` and `lu` are distinct symbols. Enum value symbols are conventionally UPPER_CASE; field name symbols are conventionally `lower-kebab-case`. The parser makes no distinction.

**Numeric literal disambiguation**: An integer literal without a decimal point is always parsed as `integer-literal`. A literal with a decimal point is always parsed as `decimal-literal`. Negative literals begin with `-` with no space between `-` and the digits.

**Set membership literal list**: The literal list in `in (...)` must contain at least one literal. An empty set `in ()` is a parse error. Semantic validation (that all literals belong to the declared domain) is a compiler responsibility, not a parser responsibility.

**Range predicate and comparison predicate**: Range predicates (`in [a .. b]`) and comparison predicates (`< 18`) both operate over ordered types. The parser accepts them over any declared input; the compiler validates that the input type is ordered (`integer` or `decimal`). Using a range or comparison predicate over an `enum` input is a compile error, not a parse error.

**Decision ID**: The `:decision-id` value is a string literal (double-quoted). It is advisory metadata: it appears in the compiled artifact but does not affect evaluation semantics.

**Attribute repetition**: Repeating the same attribute keyword (e.g., two `:hit-policy` clauses) is a parse error.

---

## 5. Worked examples

### 5.1 Booking eligibility (UNIQUE, multiple outputs)

```lisp
(define-decision booking-eligibility
  :decision-id "booking_eligibility.v1"
  :hit-policy unique
  :inputs
    ((jurisdiction      :type enum :domain Jurisdiction)
     (client-type       :type enum :domain CbuType)
     (product           :type enum :domain ProductCode)
     (booking-principal :type enum :domain BookingPrincipal)
     (source-of-funds   :type enum :domain SourceOfFunds))
  :outputs
    ((eligibility :type enum :domain EligibilityOutcome)
     (reason-code :type enum :domain BookingReasonCode))
  :rules
    ((rule r001
       :when ((jurisdiction = LU)
              (client-type = SICAV)
              (product in (CUSTODY FUND_ACCOUNTING))
              (booking-principal in (BNY_LUX BNY_IE))
              (source-of-funds != UNKNOWN))
       :then ((eligibility = ELIGIBLE)
              (reason-code = STANDARD_LUX_SICAV)))
     (rule r002
       :when ((source-of-funds = UNKNOWN))
       :then ((eligibility = NEEDS_REVIEW)
              (reason-code = SOF_MISSING)))
     (rule r999
       :when (*)
       :then ((eligibility = NOT_ELIGIBLE)
              (reason-code = NO_MATCH)))))
```

### 5.2 Age-band classification (FIRST, numeric ranges)

```lisp
; Classify customer by age band for onboarding risk scoring.
(define-decision age-band-classification
  :decision-id "age_band.v1"
  :hit-policy first
  :inputs
    ((age :type integer :domain AgeYears))
  :outputs
    ((band :type enum :domain AgeBand))
  :rules
    ((rule r-minor
       :when ((age in [* .. 18)))   ; age < 18
       :then ((band = MINOR)))
     (rule r-young-adult
       :when ((age in [18 .. 25])) ; 18 <= age <= 25
       :then ((band = YOUNG_ADULT)))
     (rule r-adult
       :when ((age in [26 .. 64])) ; 26 <= age <= 64
       :then ((band = ADULT)))
     (rule r-senior
       :when ((age in [65 .. *]))  ; age >= 65
       :then ((band = SENIOR)))))
```

### 5.3 Boolean switch (FIRST, is-null)

```lisp
; Determine KYC status based on document submission and review outcome.
(define-decision kyc-status
  :decision-id "kyc_status.v1"
  :hit-policy first
  :inputs
    ((documents-submitted :type bool   :domain TruthValue)
     (review-outcome      :type enum   :domain ReviewOutcome))
  :outputs
    ((kyc-status :type enum :domain KycStatus))
  :rules
    ((rule r-not-submitted
       :when ((documents-submitted = false))
       :then ((kyc-status = PENDING_DOCUMENTS)))
     (rule r-passed
       :when ((documents-submitted = true)
              (review-outcome = PASS))
       :then ((kyc-status = APPROVED)))
     (rule r-failed
       :when ((documents-submitted = true)
              (review-outcome = FAIL))
       :then ((kyc-status = REJECTED)))
     (rule r-fallback
       :when (*)
       :then ((kyc-status = UNDER_REVIEW)))))
```

---

## 6. Future grammar extensions

| Profile | Constructs added |
| --- | --- |
| v0.2 | `any`, `rule_order`, `collect` in `hit-policy-kind`; `for-all`, `exists`, `aggregate` predicate forms; `eq-field`, `compare-field` predicate forms |
| v0.3 | `date-before`, `date-after`, `date-within`, `snapshot-age-below` predicate forms; FEEL unary test surface for DMN XML import |
| v0.4 | Path expression symbols (`symbol "." symbol ("." symbol)*`) in field references |
| v0.5 | `define-bkm` top-level form; `call` predicate/assignment form for governed BKM invocation |
