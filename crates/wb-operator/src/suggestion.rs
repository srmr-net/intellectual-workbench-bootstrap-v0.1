use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorError {
    InvalidSuggestion,
    RequestTooLarge,
    ClarificationRequired,
    Ambiguous,
    Unavailable,
    Cancelled,
    Timeout,
    QueueFull,
}
pub type Result<T> = std::result::Result<T, OperatorError>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Suggestion {
    pub suggestion_version: String,
    pub candidate_key: String,
    pub input_values: BTreeMap<String, Value>,
    pub explanation: String,
}

// Parsing through Value first would discard duplicate keys. Reject them at every
// nesting level before schema validation or typed deserialization.
struct UniqueValue;
impl<'de> serde::de::DeserializeSeed<'de> for UniqueValue {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> serde::de::Visitor<'de> for UniqueValue {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("JSON with unique keys")
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut a: A,
    ) -> std::result::Result<(), A::Error> {
        let mut keys = std::collections::BTreeSet::new();
        while let Some(k) = a.next_key::<String>()? {
            if !keys.insert(k) {
                return Err(serde::de::Error::custom("duplicate key"));
            }
            a.next_value_seed(UniqueValue)?;
        }
        Ok(())
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut a: A,
    ) -> std::result::Result<(), A::Error> {
        while a.next_element_seed(UniqueValue)?.is_some() {}
        Ok(())
    }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
}
impl Suggestion {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        use serde::de::DeserializeSeed;
        let invalid = || OperatorError::InvalidSuggestion;
        if bytes.len() > 16 * 1024 {
            return Err(invalid());
        }
        let mut d = serde_json::Deserializer::from_slice(bytes);
        UniqueValue.deserialize(&mut d).map_err(|_| invalid())?;
        d.end().map_err(|_| invalid())?;
        let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        let schema: Value = serde_json::from_str(include_str!(
            "../../../docs/implementation/adr-005/schemas/operator-suggestion.schema.json"
        ))
        .map_err(|_| invalid())?;
        if !jsonschema::is_valid(&schema, &value) {
            return Err(invalid());
        }
        serde_json::from_value(value).map_err(|_| invalid())
    }
}

/// Explicit GBP mortgage domain policy; no package identifiers, pins, paths or
/// floating point arithmetic. Missing or extra prose requires clarification.
pub fn mortgage_terms(utterance: &str) -> Result<BTreeMap<String, Value>> {
    if utterance.len() > 4096 {
        return Err(OperatorError::RequestTooLarge);
    }
    let grammar = regex::Regex::new(r"^I need a mortgage calculator for a £([1-9][0-9]*|[1-9][0-9]{0,2}(?:,[0-9]{3})+)(?:\.([0-9]{1,2}))? loan over ([1-9][0-9]*) years at ([0-9]+(?:\.[0-9]+)?)%\.$").expect("static grammar");
    let c = grammar
        .captures(utterance)
        .ok_or(OperatorError::ClarificationRequired)?;
    let amount = c[1].replace(',', "");
    let principal = match c.get(2) {
        None => amount,
        Some(f) => format!("{amount}.{:0<2}", f.as_str()),
    };
    let months = c[3]
        .parse::<u64>()
        .ok()
        .and_then(|y| y.checked_mul(12))
        .ok_or(OperatorError::ClarificationRequired)?;
    let rate = c[4].trim_end_matches('0').trim_end_matches('.');
    // Preserve integer trailing zeroes (10% must never become 1%).
    let rate = if c[4].contains('.') {
        if rate.is_empty() {
            "0"
        } else {
            rate
        }
    } else {
        &c[4]
    };
    Ok(BTreeMap::from([
        ("principal".into(), Value::String(principal)),
        ("annual_rate_percent".into(), Value::String(rate.into())),
        ("term_months".into(), Value::from(months)),
    ]))
}
