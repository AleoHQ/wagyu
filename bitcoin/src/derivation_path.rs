use wagyu_model::derivation_path::{ChildIndex, DerivationPath, DerivationPathError};

use std::{fmt, str::FromStr};

/// Represents a Bitcoin derivation path
#[derive(Clone, PartialEq, Eq)]
pub struct BitcoinDerivationPath(Vec<ChildIndex>);

impl DerivationPath for BitcoinDerivationPath {}

impl FromStr for BitcoinDerivationPath {
    type Err = DerivationPathError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let mut parts = path.split("/");

        if parts.next().unwrap() != "m" {
            return Err(DerivationPathError::InvalidDerivationPath(path.to_string()));
        }

        let path: Result<Vec<ChildIndex>, Self::Err> = parts.map(str::parse).collect();
        Ok(Self(path?))
    }
}

impl From<Vec<ChildIndex>> for BitcoinDerivationPath {
    fn from(path: Vec<ChildIndex>) -> Self {
        Self(path)
    }
}

impl Into<Vec<ChildIndex>> for BitcoinDerivationPath {
    fn into(self) -> Vec<ChildIndex> {
        self.0
    }
}

impl<'a> From<&'a [ChildIndex]> for BitcoinDerivationPath {
    fn from(path: &'a [ChildIndex]) -> Self {
        Self(path.to_vec())
    }
}

impl<'a> ::std::iter::IntoIterator for &'a BitcoinDerivationPath {
    type Item = &'a ChildIndex;
    type IntoIter = ::std::slice::Iter<'a, ChildIndex>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Debug for BitcoinDerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl fmt::Display for BitcoinDerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("m")?;
        for index in self.0.iter() {
            f.write_str("/")?;
            fmt::Display::fmt(index, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wagyu_model::derivation_path::{ChildIndex, DerivationPathError};

    use std::str::FromStr;

    #[test]
    fn valid_path() {
        assert_eq!(BitcoinDerivationPath::from_str("m"), Ok(vec![].into()));
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0"),
            Ok(vec![ChildIndex::from_normal(0).unwrap()].into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0/1"),
            Ok(vec![ChildIndex::from_normal(0).unwrap(), ChildIndex::from_normal(1).unwrap()].into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0/1/2"),
            Ok(vec![
                ChildIndex::from_normal(0).unwrap(),
                ChildIndex::from_normal(1).unwrap(),
                ChildIndex::from_normal(2).unwrap()
            ]
            .into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0/1/2/3"),
            Ok(vec![
                ChildIndex::from_normal(0).unwrap(),
                ChildIndex::from_normal(1).unwrap(),
                ChildIndex::from_normal(2).unwrap(),
                ChildIndex::from_normal(3).unwrap()
            ]
            .into())
        );

        assert_eq!(BitcoinDerivationPath::from_str("m"), Ok(vec![].into()));
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0'"),
            Ok(vec![ChildIndex::from_hardened(0).unwrap()].into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0'/1"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_normal(1).unwrap()
            ]
            .into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0'/1/2'"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_normal(1).unwrap(),
                ChildIndex::from_hardened(2).unwrap(),
            ]
            .into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0'/1/2'/3"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_normal(1).unwrap(),
                ChildIndex::from_hardened(2).unwrap(),
                ChildIndex::from_normal(3).unwrap(),
            ]
            .into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0'/1/2'/3/4'"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_normal(1).unwrap(),
                ChildIndex::from_hardened(2).unwrap(),
                ChildIndex::from_normal(3).unwrap(),
                ChildIndex::from_hardened(4).unwrap(),
            ]
            .into())
        );

        assert_eq!(BitcoinDerivationPath::from_str("m"), Ok(vec![].into()));
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0h"),
            Ok(vec![ChildIndex::from_hardened(0).unwrap()].into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0h/1'"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_hardened(1).unwrap()
            ]
            .into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0'/1h/2'"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_hardened(1).unwrap(),
                ChildIndex::from_hardened(2).unwrap(),
            ]
            .into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0h/1'/2h/3'"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_hardened(1).unwrap(),
                ChildIndex::from_hardened(2).unwrap(),
                ChildIndex::from_hardened(3).unwrap(),
            ]
            .into())
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0'/1h/2'/3h/4'"),
            Ok(vec![
                ChildIndex::from_hardened(0).unwrap(),
                ChildIndex::from_hardened(1).unwrap(),
                ChildIndex::from_hardened(2).unwrap(),
                ChildIndex::from_hardened(3).unwrap(),
                ChildIndex::from_hardened(4).unwrap(),
            ]
            .into())
        );
    }

    #[test]
    fn invalid_path() {
        assert_eq!(
            BitcoinDerivationPath::from_str("n"),
            Err(DerivationPathError::InvalidDerivationPath("n".into()))
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("n/0"),
            Err(DerivationPathError::InvalidDerivationPath("n/0".into()))
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("n/0/0"),
            Err(DerivationPathError::InvalidDerivationPath("n/0/0".into()))
        );

        assert_eq!(
            BitcoinDerivationPath::from_str("1"),
            Err(DerivationPathError::InvalidDerivationPath("1".into()))
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("1/0"),
            Err(DerivationPathError::InvalidDerivationPath("1/0".into()))
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("1/0/0"),
            Err(DerivationPathError::InvalidDerivationPath("1/0/0".into()))
        );

        assert_eq!(
            BitcoinDerivationPath::from_str("m/0x"),
            Err(DerivationPathError::InvalidChildNumberFormat)
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0x0"),
            Err(DerivationPathError::InvalidChildNumberFormat)
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/0x00"),
            Err(DerivationPathError::InvalidChildNumberFormat)
        );

        assert_eq!(
            BitcoinDerivationPath::from_str("0/m"),
            Err(DerivationPathError::InvalidDerivationPath("0/m".into()))
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m//0"),
            Err(DerivationPathError::InvalidChildNumberFormat)
        );
        assert_eq!(
            BitcoinDerivationPath::from_str("m/2147483648"),
            Err(DerivationPathError::InvalidChildNumber(2147483648))
        );
    }
}
