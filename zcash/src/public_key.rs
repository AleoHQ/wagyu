use address::{ZcashAddress, Format};
use model::{Address, PublicKey};
use network::Network;
use private_key::ZcashPrivateKey;

use secp256k1;
use std::{fmt, fmt::Display};

///Represents a Zcash public key
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZcashPublicKey {
    /// The ECDSA public key
    pub public_key: secp256k1::PublicKey,

    /// If true, the public key is serialized in compressed form
    pub compressed: bool,
}

impl PublicKey for ZcashPublicKey {
    type Address = ZcashAddress;
    type Format = (Format, Network);
    type PrivateKey = ZcashPrivateKey;

    /// Returns the address corresponding to the given public key.
    fn from_private_key(private_key: &Self::PrivateKey) -> Self {
        let secp = secp256k1::Secp256k1::new();
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &private_key.secret_key);
        Self { public_key, compressed: private_key.compressed }
    }

    /// Returns the address of the corresponding private key.
    fn to_address(&self, format: Option<Self::Format>) -> Self::Address {
        ZcashAddress::from_public_key(self, format)
    }
}

impl Display for ZcashPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.compressed {
            for s in &self.public_key.serialize()[..] {
                write!(f, "{:02x}", s)?;
            }
        } else {
            for s in &self.public_key.serialize_uncompressed()[..] {
                write!(f, "{:02x}", s)?;
            }
        }
        Ok(())
    }
}
