use crate::Result;

use std::fmt;

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

/// Money type stores money as 1/100 of a cent. This prevents issues with floating-point rounding.
/// ie. Money(123456) represents a monetary value of 12.3456
/// Note: Money is stored as an i64, so the inner value must fit within the bounds of an i64.
#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Money(pub i64);

impl Money {
    pub const MAX: Self = Self(std::i64::MAX);
    pub const MIN: Self = Self(std::i64::MIN);

    pub fn parse(string: String) -> Result<Self> {
        let str_to_split = string.clone();
        let mut parts = str_to_split.split('.');

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

        let dollars: i64 = dollars.parse()?;
        let cents: i64 = cents.parse()?;

        Ok(Money((dollars * 10000) + cents))
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

        Ok(())
    }

    pub fn sub(&mut self, other: &Self) -> Result {
        let other = Self(-other.0);
        self.add(&other)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{:0>5}", self.0);

        let pivot = string.len() - 4;

        let dollars = &string[..pivot];
        let cents = &string[pivot..];

        write!(f, "{dollars}.{cents}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(Money::parse("0".to_string()).unwrap(), Money(0));
        assert_eq!(Money::parse("0.".to_string()).unwrap(), Money(0));
        assert_eq!(Money::parse("0.0".to_string()).unwrap(), Money(0));
        assert_eq!(Money::parse("1.0".to_string()).unwrap(), Money(10000));
        assert_eq!(
            Money::parse("123456".to_string()).unwrap(),
            Money(1234560000)
        );
        assert_eq!(
            Money::parse("123456.001".to_string()).unwrap(),
            Money(1234560010)
        );
        assert_eq!(Money::parse("0.1234".to_string()).unwrap(), Money(1234));
        assert_eq!(
            Money::parse("1234.5678".to_string()).unwrap(),
            Money(12345678)
        );
        assert_eq!(
            Money::parse("922337203685477.5807".to_string()).unwrap(),
            Money(std::i64::MAX)
        );
    }

    #[test]
    fn parse_only_reads_4_digits() {
        assert_eq!(Money::parse("0.123456".to_string()).unwrap(), Money(1234));
        assert_eq!(
            Money::parse("9000.00001".to_string()).unwrap(),
            Money(90000000)
        );
    }

    #[test]
    fn fail_to_parse_invalid_str() {
        assert!(Money::parse("".to_string()).is_err());
    }

    #[test]
    fn fail_to_parse_too_big() {
        assert!(Money::parse(std::u128::MAX.to_string()).is_err());
    }

    #[test]
    fn fail_to_parse_too_small() {
        assert!(Money::parse(std::i128::MIN.to_string()).is_err());
    }

    #[test]
    fn serialize() {
        assert_eq!(&Money(0).to_string(), "0.0000");
        assert_eq!(&Money(1).to_string(), "0.0001");
        assert_eq!(&Money(10010).to_string(), "1.0010");
        assert_eq!(&Money(12345678).to_string(), "1234.5678");
        assert_eq!(&Money(543210000).to_string(), "54321.0000");
        assert_eq!(&Money::MAX.to_string(), "922337203685477.5807");
    }

    #[test]
    fn add() {
        let mut a = Money(123);
        let b = Money(456);

        a.add(&b).unwrap();

        assert_eq!(a, Money(123 + 456));
    }

    #[test]
    fn fail_to_add_overflow() {
        let mut a = Money::MAX;
        let b = Money(1);

        assert!(a.add(&b).is_err());
    }

    #[test]
    fn fail_to_add_underflow() {
        let mut a = Money::MIN;
        let b = Money(-1);

        assert!(a.add(&b).is_err());
    }

    #[test]
    fn sub() {
        let mut a = Money(456);
        let b = Money(123);

        a.sub(&b).unwrap();

        assert_eq!(a, Money(456 - 123));
    }

    #[test]
    fn fail_to_sub_overflow() {
        let mut a = Money::MAX;
        let b = Money(-1);

        assert!(a.sub(&b).is_err());
    }

    #[test]
    fn fail_to_sub_underflow() {
        let mut a = Money::MIN;
        let b = Money(1);

        assert!(a.sub(&b).is_err());
    }
}
