//! Currency handling with cents/paise storage for precision.
//!
//! All monetary values are stored as i64 cents/paise to avoid floating-point errors.
//! Display formatting handles both USD and INR conventions.

use serde::{Deserialize, Serialize};

/// Supported currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Currency {
    #[default]
    USD,
    INR,
}

impl Currency {
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "$",
            Currency::INR => "₹",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::INR => "INR",
        }
    }

    /// Number of decimal places for this currency (2 for both USD and INR).
    pub fn decimal_places(&self) -> u8 {
        2
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl std::str::FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" | "$" => Ok(Currency::USD),
            "INR" | "₹" | "RS" => Ok(Currency::INR),
            _ => Err(format!("Unknown currency: {}", s)),
        }
    }
}

/// Type alias for monetary amounts stored as cents/paise.
pub type Cents = i64;

/// Convert dollars to cents.
pub fn dollars_to_cents(dollars: f64) -> Cents {
    (dollars * 100.0).round() as i64
}

/// Convert cents to dollars.
pub fn cents_to_dollars(cents: Cents) -> f64 {
    cents as f64 / 100.0
}

/// Convert rupees to paise.
pub fn rupees_to_paise(rupees: f64) -> Cents {
    (rupees * 100.0).round() as i64
}

/// Convert paise to rupees.
pub fn paise_to_rupees(paise: Cents) -> f64 {
    paise as f64 / 100.0
}

/// Format a cents/paise value as a currency string.
///
/// # Arguments
/// * `cents` - Amount in cents/paise
/// * `currency` - Currency type (USD or INR)
///
/// # Returns
/// Formatted string like "$1,234.56" or "₹1,23,456.78"
pub fn format_currency(cents: Cents, currency: Currency) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let whole = abs_cents / 100;
    let fraction = (abs_cents % 100) as u8;

    let formatted_whole = match currency {
        Currency::USD => format_with_commas(whole),
        Currency::INR => format_with_indian_system(whole),
    };

    let prefix = if is_negative { "-" } else { "" };

    format!(
        "{}{}{}.{:02}",
        prefix,
        currency.symbol(),
        formatted_whole,
        fraction
    )
}

/// Format a number with standard comma separators (1,234,567).
fn format_with_commas(n: i64) -> String {
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        // Insert comma before this digit when (len - i) is divisible by 3
        // and we're not at the very first digit.
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }

    result
}

/// Format a number with Indian lakh/crore system (1,23,45,678).
///
/// Indian numbering: rightmost 3 digits, then groups of 2.
/// Examples:
/// - 1000 -> 1,000
/// - 123456 -> 1,23,456 (1.23 lakhs)
/// - 12345678 -> 1,23,45,678 (1.23 crores)
fn format_with_indian_system(n: i64) -> String {
    let s = n.to_string();
    let len = s.len();

    if len <= 3 {
        return s;
    }

    // boundary = index where the rightmost-3-digits group starts
    let boundary = len - 3;
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if i > 0 {
            if i == boundary {
                // Comma right before the last 3 digits
                result.push(',');
            } else if i < boundary && (boundary - i).is_multiple_of(2) {
                // Every 2 digits within the left portion
                result.push(',');
            }
        }
        result.push(c);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dollars_to_cents() {
        assert_eq!(dollars_to_cents(1.23), 123);
        assert_eq!(dollars_to_cents(1000.0), 100000);
        assert_eq!(dollars_to_cents(0.01), 1);
    }

    #[test]
    fn test_cents_to_dollars() {
        assert_eq!(cents_to_dollars(123), 1.23);
        assert_eq!(cents_to_dollars(100000), 1000.0);
    }

    #[test]
    fn test_format_usd() {
        assert_eq!(format_currency(123456, Currency::USD), "$1,234.56");
        assert_eq!(format_currency(0, Currency::USD), "$0.00");
        assert_eq!(format_currency(-123456, Currency::USD), "-$1,234.56");
    }

    #[test]
    fn test_format_inr() {
        // 1,23,456 rupees = 1.23 lakhs
        assert_eq!(format_currency(12345600, Currency::INR), "₹1,23,456.00");
        // 1,23,45,678 rupees = 1.23 crores
        assert_eq!(format_currency(1234567800, Currency::INR), "₹1,23,45,678.00");
        assert_eq!(format_currency(0, Currency::INR), "₹0.00");
    }

    #[test]
    fn test_format_with_commas() {
        assert_eq!(format_with_commas(0), "0");
        assert_eq!(format_with_commas(100), "100");
        assert_eq!(format_with_commas(1000), "1,000");
        assert_eq!(format_with_commas(1234567), "1,234,567");
    }

    #[test]
    fn test_format_with_indian_system() {
        assert_eq!(format_with_indian_system(0), "0");
        assert_eq!(format_with_indian_system(100), "100");
        assert_eq!(format_with_indian_system(1000), "1,000");
        assert_eq!(format_with_indian_system(12345), "12,345");
        assert_eq!(format_with_indian_system(123456), "1,23,456");
        assert_eq!(format_with_indian_system(1234567), "12,34,567");
        assert_eq!(format_with_indian_system(12345678), "1,23,45,678");
    }
}
