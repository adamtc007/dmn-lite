//! Vertical-slice end-to-end test — Phase 1.3 §3.9.
//!
//! Proves that source → AST → typed IR → evaluation → output composes
//! correctly across all three crates: parser, compiler, engine.

use dmn_lite_compiler::{compile, compile_with_warnings, load_catalogue_from_str};
use dmn_lite_engine::reference::evaluate;
use dmn_lite_parser::parse;
use dmn_lite_types::{RuleId, TraceOutcome, ir::TypedValue, values::TypedInputContextBuilder};

const STUB: &str = include_str!("../../../test-data/sem-os-stub.toml");
const BOOKING_SRC: &str =
    include_str!("../../dmn-lite-parser/tests/fixtures/booking_eligibility.dmn-lite");
const AGE_SRC: &str = include_str!("../../dmn-lite-parser/tests/fixtures/age_band.dmn-lite");
const KYC_SRC: &str = include_str!("../../dmn-lite-parser/tests/fixtures/kyc_status.dmn-lite");

fn enum_val(cat: &dmn_lite_compiler::Catalogue, domain: &str, sym: &str) -> TypedValue {
    let d = cat.resolve_domain(domain).expect("domain");
    TypedValue::Enum {
        domain_id: d.domain_id,
        value_id: d.resolve_value(sym).expect("value"),
    }
}

/// The booking_eligibility fixture uses UNIQUE hit policy with a catch-all r999.
/// Under UNIQUE, r999 always fires — so any input that ALSO matches r001 would
/// give MultipleMatches (correct evaluator behaviour). The vertical slice is
/// proven by supplying an input where only r999 matches: a non-LU/non-UNKNOWN
/// input that misses r001 and r002, landing exclusively on the catch-all.
#[test]
fn vertical_slice_booking_eligibility_catch_all_unique() {
    let ast = parse(BOOKING_SRC).expect("source should parse");
    let catalogue = load_catalogue_from_str(STUB).expect("catalogue must load");
    let decision = compile(ast, &catalogue).expect("source should compile");

    assert_eq!(decision.name, "booking-eligibility");
    assert_eq!(decision.input_schema.len(), 5);
    assert_eq!(decision.rules.len(), 3);

    // Input that misses r001 (jurisdiction != LU) and r002 (source-of-funds != UNKNOWN)
    // → only the catch-all r999 fires under UNIQUE.
    let mut b = TypedInputContextBuilder::new(&decision.input_schema);
    b.set_by_name("jurisdiction", enum_val(&catalogue, "Jurisdiction", "US"))
        .unwrap();
    b.set_by_name("client-type", enum_val(&catalogue, "CbuType", "CORPORATE"))
        .unwrap();
    b.set_by_name("product", enum_val(&catalogue, "ProductCode", "DEPOSITARY"))
        .unwrap();
    b.set_by_name(
        "booking-principal",
        enum_val(&catalogue, "BookingPrincipal", "BNY_US"),
    )
    .unwrap();
    b.set_by_name(
        "source-of-funds",
        enum_val(&catalogue, "SourceOfFunds", "SALARY"),
    )
    .unwrap();

    let result =
        evaluate(&decision, &b.build(), BOOKING_SRC).expect("only r999 matches → UNIQUE succeeds");

    assert_eq!(
        result.trace.outcome,
        TraceOutcome::Match { rule_id: RuleId(2) },
        "catch-all r999 (index 2) must be the sole match"
    );
    assert_eq!(result.trace.rules.len(), 3);
    assert!(!result.trace.rules[0].matched, "r001 must not match");
    assert!(!result.trace.rules[1].matched, "r002 must not match");
    assert!(result.trace.rules[2].matched, "r999 catch-all must match");

    let eligibility = result
        .output
        .get_by_name(&decision.output_schema, "eligibility")
        .unwrap();
    assert_eq!(
        eligibility,
        &enum_val(&catalogue, "EligibilityOutcome", "NOT_ELIGIBLE")
    );
    let reason = result
        .output
        .get_by_name(&decision.output_schema, "reason-code")
        .unwrap();
    assert_eq!(
        reason,
        &enum_val(&catalogue, "BookingReasonCode", "NO_MATCH")
    );

    // Predicate descriptions non-empty (source supplied)
    for rule_trace in &result.trace.rules {
        for pred in &rule_trace.predicates {
            assert!(
                !pred.description.is_empty(),
                "all predicate descriptions must be non-empty when source is supplied"
            );
        }
    }
}

/// End-to-end with FIRST hit policy (age_band §5.2): proves range predicates,
/// numeric inputs, and ordered hit policy all compose correctly.
#[test]
fn vertical_slice_age_band_first_policy() {
    let catalogue = load_catalogue_from_str(STUB).unwrap();
    let res = compile_with_warnings(parse(AGE_SRC).unwrap(), &catalogue);
    assert!(
        res.errors.is_empty(),
        "age_band compile errors: {:?}",
        res.errors
    );
    let decision = res.partial_decision.unwrap();

    // age = 40 should match r-adult (rule index 2, [26..64] inclusive)
    let mut b = TypedInputContextBuilder::new(&decision.input_schema);
    b.set_by_name("age", TypedValue::Integer(40)).unwrap();
    let result = evaluate(&decision, &b.build(), AGE_SRC).expect("40 → ADULT");

    let band = result
        .output
        .get_by_name(&decision.output_schema, "band")
        .unwrap();
    assert_eq!(band, &enum_val(&catalogue, "AgeBand", "ADULT"));
    assert_eq!(result.trace.rules.len(), 4, "age_band has 4 rules");
    assert!(matches!(result.trace.outcome, TraceOutcome::Match { .. }));
}

/// End-to-end with boolean inputs (kyc_status §5.3): proves bool-typed input
/// and enum output resolve and evaluate correctly.
#[test]
fn vertical_slice_kyc_status_bool_input() {
    let catalogue = load_catalogue_from_str(STUB).unwrap();
    let res = compile_with_warnings(parse(KYC_SRC).unwrap(), &catalogue);
    assert!(
        res.errors.is_empty(),
        "kyc_status compile errors: {:?}",
        res.errors
    );
    let decision = res.partial_decision.unwrap();

    let mut b = TypedInputContextBuilder::new(&decision.input_schema);
    b.set_by_name("documents-submitted", TypedValue::Bool(true))
        .unwrap();
    b.set_by_name(
        "review-outcome",
        enum_val(&catalogue, "ReviewOutcome", "FAIL"),
    )
    .unwrap();
    let result = evaluate(&decision, &b.build(), KYC_SRC).expect("docs+fail → REJECTED");
    let status = result
        .output
        .get_by_name(&decision.output_schema, "kyc-status")
        .unwrap();
    assert_eq!(status, &enum_val(&catalogue, "KycStatus", "REJECTED"));
}
