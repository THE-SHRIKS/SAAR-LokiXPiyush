use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Income,
    Expense,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("Cannot perform arithmetic on differing currencies: {0} vs {1}")]
    CurrencyMismatch(String, String),
    #[error("Arithmetic overflow or underflow occurred")]
    Overflow,
    #[error("Invalid refund: refund amount ({refund}) exceeds original amount ({original})")]
    RefundExceedsOriginal { original: i64, refund: i64 },
    #[error("Currency code cannot be empty")]
    InvalidCurrency,
}

/// Exact integer minor-unit money representation.
/// Eliminates IEEE-754 binary floating point rounding errors completely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub amount_minor: i64,
    pub currency: String,
}

impl Money {
    pub fn new(amount_minor: i64, currency: &str) -> Result<Self, MoneyError> {
        let clean_curr = currency.trim().to_uppercase();
        if clean_curr.is_empty() {
            return Err(MoneyError::InvalidCurrency);
        }
        Ok(Self {
            amount_minor,
            currency: clean_curr,
        })
    }

    pub fn zero(currency: &str) -> Result<Self, MoneyError> {
        Self::new(0, currency)
    }

    pub fn direction(&self) -> Direction {
        if self.amount_minor >= 0 {
            Direction::Income
        } else {
            Direction::Expense
        }
    }

    /// Adds another money instance. Currencies MUST match identically.
    pub fn add(&self, other: &Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch(
                self.currency.clone(),
                other.currency.clone(),
            ));
        }
        let res = self
            .amount_minor
            .checked_add(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self {
            amount_minor: res,
            currency: self.currency.clone(),
        })
    }

    /// Subtracts another money instance. Currencies MUST match identically.
    pub fn sub(&self, other: &Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch(
                self.currency.clone(),
                other.currency.clone(),
            ));
        }
        let res = self
            .amount_minor
            .checked_sub(other.amount_minor)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self {
            amount_minor: res,
            currency: self.currency.clone(),
        })
    }

    /// Multiplies money by an integer multiplier.
    pub fn checked_mul(&self, factor: i64) -> Result<Self, MoneyError> {
        let res = self
            .amount_minor
            .checked_mul(factor)
            .ok_or(MoneyError::Overflow)?;
        Ok(Self {
            amount_minor: res,
            currency: self.currency.clone(),
        })
    }

    /// Applies a refund or credit note against an expense.
    pub fn apply_refund(&self, refund_amount_minor: i64) -> Result<Self, MoneyError> {
        if refund_amount_minor < 0 {
            return Err(MoneyError::Overflow);
        }
        let abs_orig = self.amount_minor.abs();
        if refund_amount_minor > abs_orig {
            return Err(MoneyError::RefundExceedsOriginal {
                original: abs_orig,
                refund: refund_amount_minor,
            });
        }

        // For an expense (e.g. -10000), a refund of 2000 yields -8000
        let new_amount = if self.amount_minor < 0 {
            self.amount_minor
                .checked_add(refund_amount_minor)
                .ok_or(MoneyError::Overflow)?
        } else {
            self.amount_minor
                .checked_sub(refund_amount_minor)
                .ok_or(MoneyError::Overflow)?
        };

        Ok(Self {
            amount_minor: new_amount,
            currency: self.currency.clone(),
        })
    }

    /// Formats amount into standard string with symbol and 2 decimal places (or 0 for JPY).
    pub fn format_display(&self) -> String {
        let is_negative = self.amount_minor < 0;
        let abs_minor = self.amount_minor.abs();
        let sign = if is_negative { "-" } else { "" };

        if self.currency == "JPY" {
            return format!("{}{}{}", sign, "¥", abs_minor);
        }

        let symbol = match self.currency.as_str() {
            "USD" => "$",
            "EUR" => "€",
            "GBP" => "£",
            "INR" => "₹",
            "CAD" => "CA$",
            "AUD" => "A$",
            _ => "",
        };

        let major = abs_minor / 100;
        let minor = abs_minor % 100;

        if symbol.is_empty() {
            format!("{}{} {}.{:02}", sign, self.currency, major, minor)
        } else {
            format!("{}{}{}.{:02}", sign, symbol, major, minor)
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_addition_same_currency() {
        let m1 = Money::new(1500, "USD").unwrap(); // $15.00
        let m2 = Money::new(1050, "USD").unwrap(); // $10.50
        let sum = m1.add(&m2).unwrap();
        assert_eq!(sum.amount_minor, 2550);
        assert_eq!(sum.currency, "USD");
    }

    #[test]
    fn test_money_currency_mismatch_rejected() {
        let usd = Money::new(1000, "USD").unwrap();
        let eur = Money::new(1000, "EUR").unwrap();
        let res = usd.add(&eur);
        assert_eq!(
            res,
            Err(MoneyError::CurrencyMismatch("USD".to_string(), "EUR".to_string()))
        );
    }

    #[test]
    fn test_money_refund_application() {
        let expense = Money::new(-4550, "USD").unwrap(); // -$45.50
        let refunded = expense.apply_refund(1050).unwrap(); // refund $10.50
        assert_eq!(refunded.amount_minor, -3500); // -$35.00

        // Refund exceeding original must be rejected
        let invalid = expense.apply_refund(5000);
        assert!(matches!(invalid, Err(MoneyError::RefundExceedsOriginal { .. })));
    }

    #[test]
    fn test_money_overflow_prevention() {
        let max = Money::new(i64::MAX, "USD").unwrap();
        let one = Money::new(1, "USD").unwrap();
        let res = max.add(&one);
        assert_eq!(res, Err(MoneyError::Overflow));
    }

    #[test]
    fn test_jpy_zero_decimal_formatting() {
        let jpy = Money::new(50000, "JPY").unwrap();
        assert_eq!(jpy.format_display(), "¥50000");
    }
}
