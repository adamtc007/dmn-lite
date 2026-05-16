//! Error vocabulary for dmn-lite.

use thiserror::Error;

use crate::ids::SourceSpan;

/// Lexical or syntactic errors produced by `dmn-lite-parser`.
///
/// Every variant carries a [`SourceSpan`] so the caller can report the
/// exact source location of the problem.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    /// An unrecognised byte was encountered during lexing.
    #[error("unexpected character '{ch}' at {span}")]
    UnexpectedChar {
        /// The unexpected character.
        ch: char,
        /// Location of the character.
        span: SourceSpan,
    },

    /// The input ended before the parser expected it to.
    #[error("unexpected end of input; expected {expected}")]
    UnexpectedEof {
        /// Description of what was expected.
        expected: String,
        /// Position where more input was expected.
        span: SourceSpan,
    },

    /// A token appeared where a different token was required.
    #[error("unexpected token '{found}'; expected {expected} at {span}")]
    UnexpectedToken {
        /// Description of what was expected.
        expected: String,
        /// What was actually found.
        found: String,
        /// Location of the unexpected token.
        span: SourceSpan,
    },

    /// A string literal contained an invalid escape sequence or was unterminated.
    #[error("malformed string literal at {span}: {reason}")]
    MalformedString {
        /// Description of the problem.
        reason: String,
        /// Start of the malformed literal.
        span: SourceSpan,
    },

    /// A number literal could not be parsed.
    #[error("malformed number literal '{text}' at {span}")]
    MalformedNumber {
        /// The literal text that failed to parse.
        text: String,
        /// Location of the literal.
        span: SourceSpan,
    },

    /// A hit-policy keyword was not recognised.
    #[error("unknown hit policy '{name}'; expected 'unique' or 'first'")]
    UnknownHitPolicy {
        /// The unrecognised keyword.
        name: String,
        /// Location of the keyword.
        span: SourceSpan,
    },

    /// A valid hit-policy keyword was used that is not supported in Profile v0.1.
    #[error("hit policy '{name}' is not supported in Profile v0.1")]
    UnsupportedHitPolicy {
        /// The unsupported keyword (e.g. `collect`, `any`, `rule_order`).
        name: String,
        /// Location of the keyword.
        span: SourceSpan,
    },

    /// The same attribute keyword appeared more than once in a decision.
    #[error("duplicate field '{keyword}' in decision")]
    DuplicateField {
        /// The keyword that was repeated.
        keyword: String,
        /// Location of the second occurrence.
        span: SourceSpan,
    },

    /// A required attribute keyword was absent from a decision.
    #[error("missing required field '{keyword}'")]
    MissingField {
        /// The missing keyword.
        keyword: String,
        /// Location where the field was expected.
        span: SourceSpan,
    },

    /// A set-membership predicate `in ()` had an empty literal list.
    #[error("empty set '()' is not valid in a set-membership predicate at {span}")]
    EmptySet {
        /// Location of the empty `()`.
        span: SourceSpan,
    },

    /// `and` or `or` was given fewer than two predicates.
    #[error("'{combinator}' requires at least two predicates at {span}")]
    TooFewPredicates {
        /// The combinator keyword (`and` or `or`).
        combinator: String,
        /// Location of the combinator form.
        span: SourceSpan,
    },

    /// A wildcard `*` appeared alongside other predicates in a `:when` block.
    #[error("wildcard '*' cannot be mixed with other predicates at {span}")]
    WildcardMixedWithPredicates {
        /// Location of the wildcard.
        span: SourceSpan,
    },

    /// More than one catch-all rule was found in a single decision.
    #[error("multiple catch-all rules in decision; second at {span}, first at {previous}")]
    MultipleCatchAllRules {
        /// Location of the second catch-all.
        span: SourceSpan,
        /// Location of the first catch-all.
        previous: SourceSpan,
    },

    /// A construct that belongs to a later Profile version was encountered.
    #[error("'{name}' is not supported in Profile v0.1 (planned for Profile {profile})")]
    UnsupportedConstruct {
        /// The name of the unsupported construct.
        name: String,
        /// The Profile version where it will be introduced.
        profile: String,
        /// Location of the construct.
        span: SourceSpan,
    },

    /// A source file contains more than one `(define-decision ...)` form.
    ///
    /// Profile v0.1 supports exactly one decision per source file
    /// (`source-file ::= ws* decision ws*`). The first decision is returned
    /// in the partial AST; subsequent decisions are not parsed.
    #[error(
        "source file contains more than one decision; multi-decision sources are not supported in Profile v0.1"
    )]
    MultipleDecisions {
        /// Span of the second (or subsequent) `(define-decision ...)` form.
        span: SourceSpan,
        /// Span of the first decision, for cross-reference in diagnostics.
        first_decision: SourceSpan,
    },
}

/// Compiler errors. Variants added in Phases 1.2–1.4.
#[derive(Debug, Error)]
pub enum CompileError {
    /// Placeholder. Real variants added in Phase 1.2.
    #[error("compile error: unimplemented in Phase 1.0")]
    Unimplemented,
}

/// Evaluation errors. Variants added in Phase 1.4.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Placeholder. Real variants added in Phase 1.4.
    #[error("eval error: unimplemented in Phase 1.0")]
    Unimplemented,
}
