#[allow(warnings)]
mod bindings;
use bindings::exports::workbench::component::computation::{ComponentError, Guest, Message};
use num_bigint::BigUint;
use serde::Deserialize;

struct Mortgage;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    principal: String,
    annual_rate_percent: String,
    term_months: serde_json::Number,
}
fn invalid() -> ComponentError {
    ComponentError {
        code: "INVALID_MORTGAGE_INPUT".into(),
        message: "Expected bounded canonical decimal mortgage terms".into(),
    }
}
fn scaled(text: &str, places: u32, maximum: u64) -> Option<u64> {
    if text.len() > 20 {
        return None;
    }
    let mut parts = text.split('.');
    let whole = parts.next()?;
    let fraction = parts.next();
    if parts.next().is_some()
        || whole.is_empty()
        || (whole.len() > 1 && whole.starts_with('0'))
        || !whole.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let fraction = match fraction {
        Some(s)
            if !s.is_empty()
                && s.len() <= places as usize
                && s.bytes().all(|b| b.is_ascii_digit()) =>
        {
            s
        }
        Some(_) => return None,
        None => "",
    };
    let mut value = whole.parse::<u64>().ok()?.checked_mul(10u64.pow(places))?;
    if !fraction.is_empty() {
        value = value.checked_add(
            fraction
                .parse::<u64>()
                .ok()?
                .checked_mul(10u64.pow(places - fraction.len() as u32))?,
        )?;
    }
    (value <= maximum).then_some(value)
}
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let rem = a % b;
        a = b;
        b = rem;
    }
    a
}
fn calculate(input: Input) -> Result<String, ComponentError> {
    let cents = scaled(&input.principal, 2, 100_000_000_000)
        .filter(|v| *v > 0)
        .ok_or_else(invalid)?;
    let rate = scaled(&input.annual_rate_percent, 6, 100_000_000).ok_or_else(invalid)?;
    // JSON Schema's integer type also permits 12.0. Only this bounded month count
    // is converted through f64; all monetary/rate arithmetic below is exact integer.
    let months = input
        .term_months
        .as_u64()
        .or_else(|| {
            input
                .term_months
                .as_f64()
                .filter(|v| v.is_finite() && v.fract() == 0.0 && *v >= 1.0 && *v <= 600.0)
                .map(|v| v as u64)
        })
        .filter(|v| (1..=600).contains(v))
        .ok_or_else(invalid)? as u32;
    let (numerator, denominator) = if rate == 0 {
        (BigUint::from(cents), BigUint::from(months))
    } else {
        let factor = gcd(rate, 1_200_000_000);
        let a = rate / factor;
        let b = 1_200_000_000 / factor;
        let power = BigUint::from(a + b).pow(months);
        let base = BigUint::from(b).pow(months);
        (
            BigUint::from(cents) * a * &power,
            BigUint::from(b) * (power - base),
        )
    };
    let mut rounded = &numerator / &denominator;
    if (&numerator % &denominator) * 2u32 >= denominator {
        rounded += 1u32;
    }
    let digits = rounded.to_str_radix(10);
    let padded = if digits.len() < 3 {
        format!("{:0>3}", digits)
    } else {
        digits
    };
    let (whole, fraction) = padded.split_at(padded.len() - 2);
    Ok(format!("{whole}.{fraction}"))
}
impl Guest for Mortgage {
    fn compute(message: Message) -> Result<Vec<Message>, ComponentError> {
        if message.port != "calculate" || message.payload_json.len() > 4096 {
            return Err(invalid());
        }
        let input: Input = serde_json::from_str(&message.payload_json).map_err(|_| invalid())?;
        let payment = calculate(input)?;
        Ok(vec![Message {
            port: "payment".into(),
            payload_json: serde_json::json!({"monthly_payment":payment}).to_string(),
        }])
    }
}
bindings::export!(Mortgage with_types_in bindings);
