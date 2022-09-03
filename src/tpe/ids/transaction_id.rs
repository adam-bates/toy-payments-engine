use std::fmt;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(pub u32);

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        assert_eq!(TransactionId(0).to_string(), "0");
        assert_eq!(TransactionId(1).to_string(), "1");
        assert_eq!(TransactionId(123).to_string(), "123");
        assert_eq!(TransactionId(std::u32::MAX).to_string(), "4294967295");
    }
}
