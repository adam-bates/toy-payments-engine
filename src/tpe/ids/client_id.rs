use std::fmt;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientId(pub u16);

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        assert_eq!(ClientId(0).to_string(), "0");
        assert_eq!(ClientId(1).to_string(), "1");
        assert_eq!(ClientId(123).to_string(), "123");
        assert_eq!(ClientId(std::u16::MAX).to_string(), "65535");
    }
}

