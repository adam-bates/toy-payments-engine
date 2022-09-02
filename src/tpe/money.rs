use crate::Result;

use thiserror::Error;

use serde::Deserialize;

#[derive(Error, Debug)]
pub enum MoneyError {
    #[error("Overflow error while applying {0} operation on {1:?} and {2:?}")]
    Overflow(&'static str, Money, Money),

    #[error("Underflow error while applying {0} operation on {1:?} and {2:?}")]
    Underflow(&'static str, Money, Money),

    #[error("Money parse error: {0}, {1}")]
    Parse(&'static str, String),
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Money(pub i64);

impl Money {
    pub const MAX: Self = Self(std::i64::MAX);
    pub const MIN: Self = Self(std::i64::MIN);

    pub fn parse(string: String) -> Result<Self> {
        let str_to_split = string.clone();
        let mut parts = str_to_split.split(".");

        if parts.clone().count() > 2 {
            Err(MoneyError::Parse("Too many decimal points", string))?
        }

        let dollars = match parts.next() {
            None => return Ok(Money(0)),
            Some(dollars) => dollars,
        };

        let cents = match parts.next() {
            None => "0000".to_string(),
            Some(cents) => format!("{:0<4}", cents)[..4].to_string(), 
        };

        dbg!(&dollars);
        dbg!(&cents);

        let dollars: i64 = dollars.parse()?;
        let cents: i64 = cents.parse()?;

        return Ok(Money((dollars * 10000) + cents));
    }

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

