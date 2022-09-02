use crate::Result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoneyError {
    #[error("Overflow error while applying {0} operation on {1:?} and {2:?}")]
    Overflow(&'static str, Money, Money),

    #[error("Underflow error while applying {0} operation on {1:?} and {2:?}")]
    Underflow(&'static str, Money, Money),
}

#[derive(Debug, Clone, Copy)]
pub struct Money(pub i64);

impl Money {
    pub const MAX: Self = Self(std::i64::MAX);
    pub const MIN: Self = Self(std::i64::MIN);

    pub fn add(&mut self, other: &Self) -> Result {
        let a = self.0;
        let b = other.0;

        if b > 0 && Money::MAX.0 - b < a {
            *self = Money::MAX;
            Err(MoneyError::Overflow("add", Money(a), *other))?
        }

        if b < 0 && Money::MIN.0 - b > a {
            *self = Money::MAX;
            Err(MoneyError::Underflow("add", Money(a), *other))?
        }

        self.0 += b;

        return Ok(());
    }

    pub fn sub(&mut self, other: &Self) -> Result {
        let other = Self(-1 * other.0);
        return self.add(&other);
    }
}

