use crate::address::{Address, AddressError};
use crate::derivation_path::{DerivationPath, DerivationPathError};
use crate::extended_public_key::ExtendedPublicKey;
use crate::network::NetworkError;
use crate::private_key::PrivateKey;
use crate::public_key::PublicKey;

use crypto_mac;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

/// The interface for a generic extended private key.
pub trait ExtendedPrivateKey: Clone + Debug + Display + FromStr + Send + Sync + 'static + Eq + Sized {
    type Address: Address;
    type DerivationPath: DerivationPath;
    type ExtendedPublicKey: ExtendedPublicKey;
    type Format;
    type PrivateKey: PrivateKey;
    type PublicKey: PublicKey;

    /// Returns a new extended private key.
    fn new(seed: &[u8], format: &Self::Format, path: &Self::DerivationPath) -> Result<Self, ExtendedPrivateKeyError>;

    /// Returns a new extended private key.
    fn new_master(seed: &[u8], format: &Self::Format) -> Result<Self, ExtendedPrivateKeyError>;

    /// Returns the extended private key of the given derivation path.
    fn derive(&self, path: &Self::DerivationPath) -> Result<Self, ExtendedPrivateKeyError>;

    /// Returns the extended public key of the corresponding extended private key.
    fn to_extended_public_key(&self) -> Self::ExtendedPublicKey;

    /// Returns the private key of the corresponding extended private key.
    fn to_private_key(&self) -> Self::PrivateKey;

    /// Returns the public key of the corresponding extended private key.
    fn to_public_key(&self) -> Self::PublicKey;

    /// Returns the address of the corresponding extended private key.
    fn to_address(&self, format: &Self::Format) -> Result<Self::Address, AddressError>;
}

#[derive(Debug, Fail)]
pub enum ExtendedPrivateKeyError {
    #[fail(display = "{}: {}", _0, _1)]
    Crate(&'static str, String),

    #[fail(display = "{}", _0)]
    DerivationPathError(DerivationPathError),

    #[fail(display = "invalid byte length: {}", _0)]
    InvalidByteLength(usize),

    #[fail(
        display = "invalid extended private key checksum: {{ expected: {:?}, found: {:?} }}",
        _0, _1
    )]
    InvalidChecksum(String, String),

    #[fail(display = "invalid version bytes: {:?}", _0)]
    InvalidVersionBytes(Vec<u8>),

    #[fail(display = "maximum child depth reached: {}", _0)]
    MaximumChildDepthReached(u8),

    #[fail(display = "{}", _0)]
    Message(String),

    #[fail(display = "{}", _0)]
    NetworkError(NetworkError),

    #[fail(display = "unsupported format: {}", _0)]
    UnsupportedFormat(String),
}

impl From<DerivationPathError> for ExtendedPrivateKeyError {
    fn from(error: DerivationPathError) -> Self {
        ExtendedPrivateKeyError::DerivationPathError(error)
    }
}

impl From<NetworkError> for ExtendedPrivateKeyError {
    fn from(error: NetworkError) -> Self {
        ExtendedPrivateKeyError::NetworkError(error)
    }
}

impl From<base58::FromBase58Error> for ExtendedPrivateKeyError {
    fn from(error: base58::FromBase58Error) -> Self {
        ExtendedPrivateKeyError::Crate("base58", format!("{:?}", error))
    }
}

impl From<bech32::Error> for ExtendedPrivateKeyError {
    fn from(error: bech32::Error) -> Self {
        ExtendedPrivateKeyError::Crate("bech32", format!("{:?}", error))
    }
}

impl From<crypto_mac::InvalidKeyLength> for ExtendedPrivateKeyError {
    fn from(error: crypto_mac::InvalidKeyLength) -> Self {
        ExtendedPrivateKeyError::Crate("crypto-mac", format!("{:?}", error))
    }
}

impl From<secp256k1::Error> for ExtendedPrivateKeyError {
    fn from(error: secp256k1::Error) -> Self {
        ExtendedPrivateKeyError::Crate("secp256k1", format!("{:?}", error))
    }
}

impl From<std::array::TryFromSliceError> for ExtendedPrivateKeyError {
    fn from(error: std::array::TryFromSliceError) -> Self {
        ExtendedPrivateKeyError::Crate("std::array", format!("{:?}", error))
    }
}

impl From<std::io::Error> for ExtendedPrivateKeyError {
    fn from(error: std::io::Error) -> Self {
        ExtendedPrivateKeyError::Crate("std::io", format!("{:?}", error))
    }
}

impl From<std::num::ParseIntError> for ExtendedPrivateKeyError {
    fn from(error: std::num::ParseIntError) -> Self {
        ExtendedPrivateKeyError::Crate("std::num", format!("{:?}", error))
    }
}
