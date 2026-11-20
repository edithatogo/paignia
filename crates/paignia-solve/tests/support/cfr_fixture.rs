//! Scientific qualification of the test fixture. Deserialize directly into
//! structs so duplicate fields are rejected before checking the reference.
use paignia_solve::{CfrVariant, CFR_VERSION};
use serde::{de::Error, Deserialize, Deserializer};

const SOURCE: &str = include_str!("../../../../fixtures/conformance/cfr-reference-v1.json");
const INDEPENDENT_LEDUC: &str =
    include_str!("../../../../fixtures/conformance/leduc-open-spiel-v1.json");
// Frozen payoff and NashConv pairs from the recorded OpenSpiel CSV. These
// bind the samples to the run even if a fixture keeps valid schedule labels.
const RECORDED_LEDUC_CHECKPOINTS: [[(f64, f64); 2]; 2] = [
    [
        (-0.091_211_783_171_159_65, 0.079_626_554_629_196_66),
        (-0.088_422_478_833_235_26, 0.034_685_342_220_880_946),
    ],
    [
        (-0.085_730_220_924_806_17, 0.011_737_789_016_919_464),
        (-0.085_637_227_475_745_73, 0.002_932_121_222_831_971),
    ],
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fixture {
    schema: String,
    method_version: String,
    diagnostic: String,
    pub(super) methods: [Method; 2],
    pub(super) kuhn: Kuhn,
    pub(super) leduc: Leduc,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub(super) struct Method(pub(super) CfrVariant);

impl TryFrom<String> for Method {
    type Error = &'static str;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        [CfrVariant::Cfr, CfrVariant::CfrPlus]
            .into_iter()
            .find(|method| method.id() == id)
            .map(Self)
            .ok_or("unknown CFR method")
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Kuhn {
    ante: u32,
    bet: u32,
    deals: u32,
    #[serde(deserialize_with = "fraction")]
    player_one_value: f64,
    pub(super) iteration_counts: [u32; 3],
    pub(super) player_one_value_decimal: f64,
    pub(super) value_absolute_tolerance: f64,
    pub(super) final_nash_conv_upper_bound: f64,
    #[serde(rename = "citation")]
    _citation: String,
    #[serde(rename = "notes")]
    _notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Leduc {
    pub(super) iteration_counts: [u32; 2],
    pub(super) information_states: usize,
    pub(super) player_one_value: f64,
    pub(super) published_value_check: PublishedCheck,
    #[serde(deserialize_with = "fraction")]
    pub(super) all_call_best_response_per_seat: f64,
    #[serde(deserialize_with = "fraction")]
    pub(super) all_call_nash_conv: f64,
    #[serde(rename = "citation")]
    _citation: String,
    #[serde(rename = "rules_citation")]
    _rules_citation: String,
    #[serde(rename = "source_url")]
    _source_url: String,
    #[serde(rename = "notes")]
    _notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PublishedCheck {
    pub(super) method: Method,
    pub(super) iterations: u32,
    pub(super) absolute_tolerance: f64,
    pub(super) nash_conv_upper_bound: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndependentLeduc {
    schema: String,
    open_spiel_commit: String,
    game: String,
    average_weight: String,
    cfr_plus_update_order: [u8; 2],
    curves: [IndependentCurve; 2],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndependentCurve {
    method: Method,
    samples: [TrainingSample; 2],
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrainingSample {
    pub(super) iterations: u32,
    pub(super) first_player_value: f64,
    pub(super) nash_conv: f64,
}

pub(super) fn independent_leduc_training(
    variant: CfrVariant,
    budgets: [u32; 2],
) -> [TrainingSample; 2] {
    let reference = qualify_independent_leduc(INDEPENDENT_LEDUC, budgets)
        .expect("invalid independent Leduc training fixture");
    reference
        .curves
        .into_iter()
        .find(|curve| curve.method.0 == variant)
        .expect("reference curve for this variant")
        .samples
}

fn qualify_independent_leduc(input: &str, budgets: [u32; 2]) -> Result<IndependentLeduc, String> {
    let reference: IndependentLeduc =
        serde_json::from_str(input).map_err(|error| error.to_string())?;
    if reference.schema != "paignia/leduc-open-spiel/1.0.0"
        || reference.open_spiel_commit != "4956843d7375e29a4c986425c66f897d7ac15340"
        || reference.game != "leduc_poker"
        || reference.average_weight != "own_reach"
        || reference.cfr_plus_update_order != [1, 0]
    {
        return Err("independent Leduc source or schedule".to_owned());
    }
    let [vanilla, plus] = &reference.curves;
    if [vanilla.method.0, plus.method.0] != [CfrVariant::Cfr, CfrVariant::CfrPlus] {
        return Err("independent Leduc methods".to_owned());
    }
    for (curve, recorded) in reference.curves.iter().zip(RECORDED_LEDUC_CHECKPOINTS) {
        if curve.samples.map(|sample| sample.iterations) != budgets
            || curve.samples.iter().any(|sample| {
                !sample.first_player_value.is_finite()
                    || !sample.nash_conv.is_finite()
                    || sample.nash_conv < 0.0
            })
            || curve.samples[1].nash_conv >= curve.samples[0].nash_conv
        {
            return Err("independent Leduc budgets or training values".to_owned());
        }
        if curve
            .samples
            .map(|sample| (sample.first_player_value, sample.nash_conv))
            != recorded
        {
            return Err("independent Leduc checkpoints differ from the recorded run".to_owned());
        }
    }
    Ok(reference)
}

fn fraction<'de, D: Deserializer<'de>>(input: D) -> Result<f64, D::Error> {
    let raw = String::deserialize(input)?;
    let (n, d) = raw
        .split_once('/')
        .ok_or_else(|| D::Error::custom("expected a fraction"))?;
    let n: i32 = n.parse().map_err(D::Error::custom)?;
    let d: u32 = d.parse().map_err(D::Error::custom)?;
    if d == 0 {
        return Err(D::Error::custom("zero denominator"));
    }
    Ok(f64::from(n) / f64::from(d))
}

fn same(actual: f64, expected: f64) -> bool {
    actual.to_bits() == expected.to_bits()
}

pub(super) fn fixture() -> Fixture {
    qualify(SOURCE).expect("invalid CFR reference fixture")
}

fn qualify(input: &str) -> Result<Fixture, String> {
    let f: Fixture = serde_json::from_str(input).map_err(|err| err.to_string())?;
    let k = &f.kuhn;
    let l = &f.leduc;
    let check = &l.published_value_check;
    let checks = [
        (
            f.schema == "paignia/cfr-reference/1.0.0"
                && f.method_version == CFR_VERSION
                && f.diagnostic == "nash_conv",
            "reference identity or diagnostic",
        ),
        (f.methods[0] != f.methods[1], "method inventory"),
        ((k.ante, k.bet, k.deals) == (1, 1, 6), "Kuhn rules"),
        (
            k.iteration_counts == [100, 1_000, 12_000] && l.iteration_counts == [1_000, 4_000],
            "iteration counts",
        ),
        (
            same(k.player_one_value, -1.0 / 18.0)
                && same(k.player_one_value_decimal, k.player_one_value),
            "Kuhn value",
        ),
        (
            same(k.value_absolute_tolerance, 0.005) && same(k.final_nash_conv_upper_bound, 0.02),
            "Kuhn tolerances",
        ),
        (
            l.information_states == 936
                && check.method.0 == CfrVariant::CfrPlus
                && check.iterations == l.iteration_counts[1],
            "Leduc published-value scope",
        ),
        // Schauenberg (2006), Fig. 5.3, first to act.
        (
            same(l.player_one_value, -85_606.0 / 1_000_000.0),
            "Leduc player_one_value",
        ),
        (
            same(l.all_call_best_response_per_seat, 22.0 / 15.0)
                && same(l.all_call_nash_conv, 44.0 / 15.0),
            "Leduc all-call values",
        ),
        (
            same(check.absolute_tolerance, 0.02) && same(check.nash_conv_upper_bound, 0.05),
            "Leduc tolerances",
        ),
    ];
    for (passes, field) in checks {
        if !passes {
            return Err(field.to_owned());
        }
    }
    Ok(f)
}

// Construct the mutated document using a JSON Pointer, preserving the literal
// spelling (or duplicate key) that the actual qualification must read.
fn with_literal(pointer: &str, literal: &str) -> String {
    let mut doc: serde_json::Value = serde_json::from_str(SOURCE).unwrap();
    *doc.pointer_mut(pointer).expect("mutation field exists") =
        serde_json::json!("__replace_field__");
    doc.to_string()
        .replacen(r#""__replace_field__""#, literal, 1)
}

#[test]
fn fixture_qualifies_numeric_targets_methods_and_budgets() {
    fixture();
}

#[test]
fn independent_leduc_fixture_rejects_the_unmatched_cfr_plus_schedule() {
    let mut input: serde_json::Value = serde_json::from_str(INDEPENDENT_LEDUC).unwrap();
    input["cfr_plus_update_order"] = serde_json::json!([0, 1]);
    assert_eq!(
        qualify_independent_leduc(&input.to_string(), [1_000, 4_000]).unwrap_err(),
        "independent Leduc source or schedule"
    );
}

#[test]
fn independent_leduc_fixture_rejects_values_from_the_wrong_schedule() {
    let mut input: serde_json::Value = serde_json::from_str(INDEPENDENT_LEDUC).unwrap();
    let early_cfr_plus = &mut input["curves"][1]["samples"][0];
    early_cfr_plus["first_player_value"] = serde_json::json!(-0.086_337_731_296_827_06);
    early_cfr_plus["nash_conv"] = serde_json::json!(0.013_886_401_842_352_361);
    assert_eq!(
        qualify_independent_leduc(&input.to_string(), [1_000, 4_000]).unwrap_err(),
        "independent Leduc checkpoints differ from the recorded run"
    );
}

#[test]
fn fixture_rejects_a_changed_numeric_target_even_when_the_old_text_survives() {
    let path = "/leduc/player_one_value";
    assert_eq!(
        qualify(&with_literal(path, "-0.085606e3")).unwrap_err(),
        "Leduc player_one_value"
    );
    qualify(&with_literal(path, "-8.5606e-2")).expect("equivalent decimal");
}

#[test]
fn fixture_rejects_duplicate_fields_at_each_scope() {
    for (path, duplicate, key) in [
        (
            "/method_version",
            r#""1.0.0","method_version":"1.0.0""#,
            "method_version",
        ),
        (
            "/leduc/player_one_value",
            r#"-0.085606,"player_one_value":-0.085606"#,
            "player_one_value",
        ),
        (
            "/leduc/published_value_check/iterations",
            r#"4000,"iterations":4000"#,
            "iterations",
        ),
        ("/kuhn/deals", r#"6,"deals":6"#, "deals"),
    ] {
        let err = qualify(&with_literal(path, duplicate)).unwrap_err();
        assert!(err.contains(&format!("duplicate field `{key}`")), "{err}");
    }
}

#[test]
fn fixture_rejects_wrong_types_methods_units_and_published_budgets() {
    for (path, replacement, expected) in [
        ("/leduc/player_one_value", r#""-0.085606""#, "invalid type"),
        (
            "/leduc/published_value_check/method",
            r#""paignia/vanilla-cfr""#,
            "Leduc published-value scope",
        ),
        (
            "/leduc/published_value_check/iterations",
            "1000",
            "Leduc published-value scope",
        ),
        (
            "/diagnostic",
            r#""exploitability""#,
            "reference identity or diagnostic",
        ),
        (
            "/method_version",
            r#""9.0.0""#,
            "reference identity or diagnostic",
        ),
        (
            "/kuhn/iteration_counts",
            "[100,1000,12001]",
            "iteration counts",
        ),
        ("/leduc/iteration_counts", "[1000,5000]", "iteration counts"),
        (
            "/methods",
            r#"["paignia/cfr-plus","paignia/cfr-plus"]"#,
            "method inventory",
        ),
        (
            "/methods",
            r#"["paignia/vanilla-cfr","paignia/unknown"]"#,
            "unknown CFR method",
        ),
    ] {
        let err = qualify(&with_literal(path, replacement)).unwrap_err();
        assert!(err.contains(expected), "{path}: {err}");
    }
}
