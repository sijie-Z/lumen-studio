use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Money value object: precision, non-negative, compile-time type safety.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money(Decimal);

#[derive(Debug, thiserror::Error)]
pub enum MoneyError {
    #[error("Amount must be positive, got {0}")]
    NegativeAmount(Decimal),
    #[error("Amount exceeds maximum: {0}")]
    ExceedsMaximum(Decimal),
}

impl Money {
    pub fn max() -> Decimal {
        Decimal::new(100_000_00, 2)
    }

    pub fn new(amount: Decimal) -> Result<Self, MoneyError> {
        if amount < Decimal::ZERO {
            return Err(MoneyError::NegativeAmount(amount));
        }
        if amount > Self::max() {
            return Err(MoneyError::ExceedsMaximum(amount));
        }
        Ok(Self(amount.round_dp(2)))
    }

    pub fn from_yuan(yuan: f64) -> Result<Self, MoneyError> {
        let amount = Decimal::from_f64_retain(yuan).unwrap_or(Decimal::ZERO);
        Self::new(amount)
    }

    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    pub fn amount(&self) -> Decimal {
        self.0
    }

    pub fn commission(&self) -> Money {
        let commission = self.0 * Decimal::new(1, 1);
        Self(commission.round_dp(2))
    }
}

impl std::ops::Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
