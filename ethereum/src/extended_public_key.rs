use crate::address::EthereumAddress;
use crate::extended_private_key::EthereumExtendedPrivateKey;
use crate::public_key::EthereumPublicKey;
use wagu_model::{
    AddressError,
    ExtendedPublicKey,
    ExtendedPublicKeyError,
    PublicKey,
    crypto::{checksum, hash160}};

use base58::{ToBase58, FromBase58};
use byteorder::{BigEndian, ByteOrder, ReadBytesExt};
use hex;
use hmac::{Hmac, Mac};
use secp256k1::{Secp256k1, SecretKey, PublicKey as Secp256k1_PublicKey};
use sha2::Sha512;
use std::fmt;
use std::io::Cursor;
use std::str::FromStr;
use std::ops::AddAssign;
use serde::export::PhantomData;

type HmacSha512 = Hmac<Sha512>;

/// Represents a Ethereum extended public key
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EthereumExtendedPublicKey {
    /// The Secp256k1 public key associated with a EthereumExtendedPrivateKey's private_key
    pub public_key: EthereumPublicKey,
    /// The chain code associated with a EthereumExtendedPrivateKey
    pub chain_code: [u8; 32],
    /// 0x00 for master nodes, 0x01 for level-1 derived keys, ....
    pub depth: u8,
    /// The first 32 bits of the key identifier (hash160(ECDSA_public_key))
    pub parent_fingerprint: [u8; 4],
    /// This is ser32(i) for i in xi = xpar/i, with xi the key being serialized. (0x00000000 if master key)
    pub child_number: u32,
}

impl ExtendedPublicKey for EthereumExtendedPublicKey {
    type Address = EthereumAddress;
    type ExtendedPrivateKey = EthereumExtendedPrivateKey;
    type Format = PhantomData<u8>;
    type Network = PhantomData<u8>;
    type PublicKey = EthereumPublicKey;

    /// Returns extended public key given extended private key
    fn from_extended_private_key(private_key: &EthereumExtendedPrivateKey) -> Self {
        Self {
            public_key: EthereumPublicKey::from_private_key(&private_key.private_key),
            chain_code: private_key.chain_code,
            depth: private_key.depth,
            parent_fingerprint: private_key.parent_fingerprint,
            child_number: private_key.child_number,
        }
    }

    /// Returns the public key of the corresponding extended public key.
    fn to_public_key(&self) -> Self::PublicKey {
        self.public_key
    }

    /// Returns the address of the corresponding extended public key.
    fn to_address(&self, _: &Self::Format) -> Result<Self::Address, AddressError> {
        self.public_key.to_address(&PhantomData, &PhantomData)
    }
}

impl EthereumExtendedPublicKey {
    /// Returns the extended public key for the given derivation path.
    pub fn derivation_path(&self, path: &str) -> Result<Self, ExtendedPublicKeyError> {
        let mut path_vec: Vec<&str> = path.split("/").collect();

        if path_vec[0] != "m" {
            return Err(ExtendedPublicKeyError::InvalidDerivationPath("m".into(), path_vec[0].into()))
        }

        if path_vec.len() == 1 {
            return Ok(self.clone())
        }

        let mut extended_public_key = self.clone();
        for (i, child_str) in path_vec[1..].iter_mut().enumerate() {
            let mut child_num = 0u32;

            // if hardened path return failure
            if child_str.contains("'") {
                return Err(ExtendedPublicKeyError::InvalidDerivationPath("".into(), "'".into()))
            } else {
                let child_num_u32: u32 = match child_str.parse() {
                    Ok(num) => num,
                    Err(_) => return Err(ExtendedPublicKeyError::InvalidDerivationPath("number".into(), path_vec[i + 1].into()))
                };
                child_num.add_assign(child_num_u32);
            }
            extended_public_key = extended_public_key.ckd_pub(child_num)?;
        }

        Ok(extended_public_key)
    }


    /// Returns the child extended public key for the given child number.
    pub fn ckd_pub(&self, child_number: u32) -> Result<Self, ExtendedPublicKeyError> {
        if self.depth >= 255 {
            return Err(ExtendedPublicKeyError::MaximumChildDepthReached(self.depth))
        }

        let mut mac = HmacSha512::new_varkey(&self.chain_code)?;
        let public_key_serialized = &self.public_key.0.serialize()[..];

        // Check whether i ≥ 2^31 (whether the child is a hardened key).
        //
        // If so (hardened child): return failure
        // If not (normal child): let I = HMAC-SHA512(Key = cpar, Data = serP(Kpar) || ser32(i)).
        //
        if child_number >= 2_u32.pow(31) {
            return Err(ExtendedPublicKeyError::InvalidChildNumber(2_u32.pow(31), child_number))
        } else {
            mac.input(public_key_serialized);
        }

        let mut child_num_big_endian = [0; 4];
        BigEndian::write_u32(&mut child_num_big_endian, child_number);
        mac.input(&child_num_big_endian);

        let result = mac.result().code();

        let mut chain_code = [0u8; 32];
        chain_code[0..32].copy_from_slice(&result[32..]);

        let secret_key = SecretKey::from_slice(&Secp256k1::without_caps(), &result[..32])?;
        let mut public_key = self.public_key.clone();
        public_key.0.add_exp_assign(&Secp256k1::new(), &secret_key)?;

        let mut parent_fingerprint = [0u8; 4];
        parent_fingerprint.copy_from_slice(&hash160(public_key_serialized)[0..4]);

        Ok(Self {
            public_key,
            chain_code,
            depth: self.depth + 1,
            parent_fingerprint,
            child_number,
        })
    }
}

impl FromStr for EthereumExtendedPublicKey {
    type Err = ExtendedPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = s.from_base58()?;
        if data.len() != 82 {
            return Err(ExtendedPublicKeyError::InvalidByteLength(data.len()))
        }

        if &data[0..4] != [0x04u8, 0x88, 0xB2, 0x1E] {
            return Err(ExtendedPublicKeyError::InvalidNetworkBytes(data[0..4].to_vec()))
        };

        let depth = data[4] as u8;

        let mut parent_fingerprint = [0u8; 4];
        parent_fingerprint.copy_from_slice(&data[5..9]);

        let child_number: u32 = Cursor::new(&data[9..13]).read_u32::<BigEndian>().unwrap();

        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&data[13..45]);

        let secp = Secp256k1::new();
        let secp256k1_public_key = Secp256k1_PublicKey::from_slice(&secp, &data[45..78])?;

        let public_key = EthereumPublicKey::from_str(
            &hex::encode(&secp256k1_public_key.serialize_uncompressed()[1..]))?;

        let expected = &data[78..82];
        let checksum = &checksum(&data[0..78])[0..4];
        if *expected != *checksum {
            let expected = expected.to_base58();
            let found = checksum.to_base58();
            return Err(ExtendedPublicKeyError::InvalidChecksum(expected, found))
        }

        Ok(Self {
            public_key,
            chain_code,
            depth,
            parent_fingerprint,
            child_number,
        })
    }
}

impl fmt::Display for EthereumExtendedPublicKey {
    /// BIP32 serialization format
    /// https://github.com/ethereum/bips/blob/master/bip-0032.mediawiki#serialization-format
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut result = [0u8; 82];
        result[0..4].copy_from_slice(&[0x04u8, 0x88, 0xB2, 0x1E][..]);
        result[4] = self.depth as u8;
        result[5..9].copy_from_slice(&self.parent_fingerprint[..]);

        BigEndian::write_u32(&mut result[9..13], u32::from(self.child_number));

        result[13..45].copy_from_slice(&self.chain_code[..]);
        result[45..78].copy_from_slice(&self.public_key.0.serialize()[..]);

        let sum = &checksum(&result[0..78])[0..4];
        result[78..82].copy_from_slice(sum);

        fmt.write_str(&result.to_base58())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    fn test_from_str(
        expected_public_key: &str,
        expected_chain_code: &str,
        expected_depth: u8,
        expected_parent_fingerprint: &str,
        expected_child_number: u32,
        expected_extended_public_key_serialized: &str,
    ) {
        let extended_public_key = EthereumExtendedPublicKey::from_str(&expected_extended_public_key_serialized).expect("Error generating extended_public_key from string");
        assert_eq!(expected_public_key, extended_public_key.public_key.to_string());
        assert_eq!(expected_chain_code, hex::encode(extended_public_key.chain_code));
        assert_eq!(expected_depth, extended_public_key.depth);
        assert_eq!(expected_parent_fingerprint, hex::encode(extended_public_key.parent_fingerprint));
        assert_eq!(expected_child_number, extended_public_key.child_number);
        assert_eq!(expected_extended_public_key_serialized, extended_public_key.to_string());
    }

    fn test_from_private(
        expected_public_key: &str,
        expected_chain_code: &str,
        expected_parent_fingerprint: &str,
        expected_extended_public_key_serialized: &str,
        extended_private_key_serialized: &str,
    ) {
        let extended_private_key = EthereumExtendedPrivateKey::from_str(extended_private_key_serialized).unwrap();
        let extended_public_key = EthereumExtendedPublicKey::from_extended_private_key(&extended_private_key);
        assert_eq!(expected_public_key, extended_public_key.public_key.to_string());
        assert_eq!(expected_chain_code, hex::encode(extended_public_key.chain_code));
        assert_eq!(expected_parent_fingerprint, hex::encode(extended_public_key.parent_fingerprint));
        assert_eq!(expected_extended_public_key_serialized, extended_public_key.to_string());
    }

    fn test_ckd_pub(
        expected_public_key: &str,
        expected_chain_code: &str,
        expected_parent_fingerprint: &str,
        expected_extended_public_key_serialized: &str,
        parent_extended_public_key: &EthereumExtendedPublicKey,
        child_number: u32,
    ) -> EthereumExtendedPublicKey {
        let child_extended_public_key = parent_extended_public_key.ckd_pub(child_number).expect("error generating child key");
        assert_eq!(expected_public_key, child_extended_public_key.public_key.to_string());
        assert_eq!(expected_chain_code, hex::encode(child_extended_public_key.chain_code));
        assert_eq!(expected_parent_fingerprint, hex::encode(child_extended_public_key.parent_fingerprint));
        assert_eq!(expected_extended_public_key_serialized, child_extended_public_key.to_string());
        assert_eq!(child_number, child_extended_public_key.child_number);

        child_extended_public_key
    }

    fn test_derivation_path(
        expected_public_key: &str,
        expected_chain_code: &str,
        expected_parent_fingerprint: &str,
        expected_extended_public_key_serialized: &str,
        parent_extended_public_key: &EthereumExtendedPublicKey,
        path: &str,
    ) {
        let derived_extended_public_key = parent_extended_public_key.derivation_path(path).expect("error deriving key from path");
        assert_eq!(expected_public_key, derived_extended_public_key.public_key.to_string());
        assert_eq!(expected_chain_code, hex::encode(derived_extended_public_key.chain_code));
        assert_eq!(expected_parent_fingerprint, hex::encode(derived_extended_public_key.parent_fingerprint));
        assert_eq!(expected_extended_public_key_serialized, derived_extended_public_key.to_string());
    }

    mod bip32_default {
        use super::*;
        use wagu_model::ExtendedPrivateKey;

        // (depth, master_seed, public_key, chain_code, parent_fingerprint, extended_private_key_serialized, extended_public_key_serialized)
        const KEYPAIR_TREE_HARDENED: [(&str, &str, &str, &str, &str, &str, &str); 2] = [
            (
                "0x00",
                "000102030405060708090a0b0c0d0e0f",
                "39a36013301597daef41fbe593a02cc513d0b55527ec2df1050e2e8ff49c85c23cbe7ded0e7ce6a594896b8f62888fdbc5c8821305e2ea42bf01e37300116281",
                "873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508",
                "00000000",
                "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi",
                "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8"
            ),
            (
                "0x01",
                "m/0'",
                "035a784662a4a20a65bf6aab9ae98a6c068a81c52e4b032c0fb5400c706cfccc56",
                "47fdacbd0f1097043b78c63c20c34ef4ed9a111d980047ad16282c7ae6236141",
                "3442193e",
                "xprv9uHRZZhk6KAJC1avXpDAp4MDc3sQKNxDiPvvkX8Br5ngLNv1TxvUxt4cV1rGL5hj6KCesnDYUhd7oWgT11eZG7XnxHrnYeSvkzY7d2bhkJ7",
                "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw"
            )
        ];

        // (depth, master_seed, secret_key, chain_code, parent_fingerprint, extended_private_key_serialized, extended_public_key_serialized)
        const KEYPAIR_TREE_NORMAL: [(&str, &str, &str, &str, &str, &str, &str); 2] = [
            (
                "0x00",
                "fffcf9f6f3f0edeae7e4e1dedbd8d5d2cfccc9c6c3c0bdbab7b4b1aeaba8a5a29f9c999693908d8a8784817e7b7875726f6c696663605d5a5754514e4b484542",
                "cbcaa9c98c877a26977d00825c956a238e8dddfbd322cce4f74b0b5bd6ace4a77bd3305d363c26f82c1e41c667e4b3561c06c60a2104d2b548e6dd059056aa51",
                "60499f801b896d83179a4374aeb7822aaeaceaa0db1f85ee3e904c4defbd9689",
                "00000000",
                "xprv9s21ZrQH143K31xYSDQpPDxsXRTUcvj2iNHm5NUtrGiGG5e2DtALGdso3pGz6ssrdK4PFmM8NSpSBHNqPqm55Qn3LqFtT2emdEXVYsCzC2U",
                "xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB"
            ),
            (
                "0x01",
                "m/0",
                "fc9e5af0ac8d9b3cecfe2a888e2117ba3d089d8585886c9c826b6b22a98d12ea67a50538b6f7d8b5f7a1cc657efd267cde8cc1d8c0451d1340a0fb3642777544",
                "f0909affaa7ee7abe5dd4e100598d4dc53cd709d5a5c2cac40e7412f232f7c9c",
                "bd16bee5",
                "xprv9vHkqa6EV4sPZHYqZznhT2NPtPCjKuDKGY38FBWLvgaDx45zo9WQRUT3dKYnjwih2yJD9mkrocEZXo1ex8G81dwSM1fwqWpWkeS3v86pgKt",
                "xpub69H7F5d8KSRgmmdJg2KhpAK8SR3DjMwAdkxj3ZuxV27CprR9LgpeyGmXUbC6wb7ERfvrnKZjXoUmmDznezpbZb7ap6r1D3tgFxHmwMkQTPH"
            )
        ];

        #[test]
        fn test_from_str_hardened() {
            let (
                _,
                _,
                public_key,
                chain_code,
                parent_fingerprint,
                _,
                extended_public_key
            ) = KEYPAIR_TREE_HARDENED[0];
            test_from_str(
                public_key,
                chain_code,
                0,
                parent_fingerprint,
                0,
                extended_public_key,
            );
        }

        #[test]
        fn test_from_str_normal() {
            let (
                _,
                _,
                public_key,
                chain_code,
                parent_fingerprint,
                _,
                extended_public_key
            ) = KEYPAIR_TREE_NORMAL[0];
            test_from_str(
                public_key,
                chain_code,
                0,
                parent_fingerprint,
                0,
                extended_public_key,
            );
        }

        #[test]
        fn test_from_private_hardened() {
            let (
                _,
                _,
                public_key,
                chain_code,
                parent_fingerprint,
                extended_private_key,
                extended_public_key
            ) = KEYPAIR_TREE_HARDENED[0];
            test_from_private(
                public_key,
                chain_code,
                parent_fingerprint,
                extended_public_key,
                extended_private_key,
            );
        }

        #[test]
        fn test_from_private_normal() {
            let (
                _,
                _,
                public_key,
                chain_code,
                parent_fingerprint,
                extended_private_key,
                extended_public_key
            ) = KEYPAIR_TREE_NORMAL[0];
            test_from_private(
                public_key,
                chain_code,
                parent_fingerprint,
                extended_public_key,
                extended_private_key,
            );
        }

        #[test]
        fn test_ckd_pub_normal() {
            let (_, _, _, _, _, extended_private_key_serialized, _) = KEYPAIR_TREE_NORMAL[0];
            let parent_extended_private_key = EthereumExtendedPrivateKey::from_str(extended_private_key_serialized).unwrap();
            let mut parent_extended_public_key = parent_extended_private_key.to_extended_public_key();
            for (i,
                (
                    _,
                    _,
                    public_key,
                    chain_code,
                    parent_fingerprint,
                    _,
                    extended_public_key
                )
            ) in KEYPAIR_TREE_NORMAL[1..].iter_mut().enumerate() {
                parent_extended_public_key = test_ckd_pub(
                    public_key,
                    chain_code,
                    parent_fingerprint,
                    extended_public_key,
                    &parent_extended_public_key,
                    i as u32,
                );
            }
        }

        #[test]
        #[should_panic(expected = "InvalidChildNumber(2147483648, 2147483648)")]
        fn test_ckd_pub_hardened_panic() {
            let (_, _, _, _, _, extended_private_key_serialized, _) = KEYPAIR_TREE_HARDENED[0];
            let parent_extended_private_key = EthereumExtendedPrivateKey::from_str(&extended_private_key_serialized).unwrap();
            let parent_extended_public_key = parent_extended_private_key.to_extended_public_key();
            let _result = parent_extended_public_key.ckd_pub(2_u32.pow(31)).unwrap();
        }

        #[test]
        fn test_derivation_path_normal() {
            let (_, _, _, _, _, extended_private_key_serialized, _) = KEYPAIR_TREE_NORMAL[0];
            let parent_extended_private_key = EthereumExtendedPrivateKey::from_str(extended_private_key_serialized).unwrap();
            let parent_extended_public_key = parent_extended_private_key.to_extended_public_key();
            for (_,
                (
                    _,
                    path,
                    public_key,
                    chain_code,
                    parent_fingerprint,
                    _,
                    extended_public_key
                )
            ) in KEYPAIR_TREE_NORMAL[1..].iter_mut().enumerate() {
                test_derivation_path(
                    public_key,
                    chain_code,
                    parent_fingerprint,
                    extended_public_key,
                    &parent_extended_public_key,
                    path,
                );
            }
        }

        #[test]
        #[should_panic(expected = "InvalidDerivationPath(\"\", \"\\'\")")]
        fn test_derivation_path_hardened_panic() {
            let (_, _, _, _, _, extended_private_key_serialized, _) = KEYPAIR_TREE_HARDENED[0];
            let parent_extended_private_key = EthereumExtendedPrivateKey::from_str(&extended_private_key_serialized).unwrap();
            let parent_extended_public_key = parent_extended_private_key.to_extended_public_key();
            let _result = parent_extended_public_key.derivation_path("m/0'").unwrap();
        }
    }

    mod test_invalid {
        use super::*;

        const INVALID_PATH: &str = "/0";
        const INVALID_PATH_HARDENED: &str = "m/a'";
        const INVALID_PATH_NORMAL: &str = "m/a";
        const INVALID_EXTENDED_PUBLIC_KEY_PUBLIC_KEY: &str = "xpub661MyMwAqRbcftXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
        const INVALID_EXTENDED_PUBLIC_KEY_NETWORK: &str = "xpub561MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
        const INVALID_EXTENDED_PUBLIC_KEY_CHECKSUM: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet7";
        const VALID_EXTENDED_PUBLIC_KEY: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
        const VALID_EXTENDED_PUBLIC_KEY_FINAL: &str = "xpubEND4cWBkwMUcwj3bjw4RNYcpnuvgbEaGSCAujB1XQro3Ptpvs8hDMFsBmk1mhfz9sGc3k4XPpueGAcR66Kb7HMXwfnKKBaV3i7YyMxLuwKh";

        #[test]
        #[should_panic(expected = "Crate(\"secp256k1\", \"InvalidPublicKey\")")]
        fn from_str_invalid_secret_key() {
            let _result = EthereumExtendedPublicKey::from_str(INVALID_EXTENDED_PUBLIC_KEY_PUBLIC_KEY).unwrap();
        }

        #[test]
        #[should_panic(expected = "InvalidNetworkBytes([4, 136, 178, 29])")]
        fn from_str_invalid_network() {
            let _result = EthereumExtendedPublicKey::from_str(INVALID_EXTENDED_PUBLIC_KEY_NETWORK).unwrap();
        }

        #[test]
        #[should_panic(expected = "InvalidChecksum(\"5Nvot3\", \"5Nvot4\")")]
        fn from_str_invalid_checksum() {
            let _result = EthereumExtendedPublicKey::from_str(INVALID_EXTENDED_PUBLIC_KEY_CHECKSUM).unwrap();
        }

        #[test]
        #[should_panic(expected = "InvalidByteLength(81)")]
        fn from_str_short() {
            let _result = EthereumExtendedPublicKey::from_str(&VALID_EXTENDED_PUBLIC_KEY[1..]).unwrap();
        }

        #[test]
        #[should_panic(expected = "InvalidByteLength(83)")]
        fn from_str_long() {
            let mut string = String::from(VALID_EXTENDED_PUBLIC_KEY);
            string.push('a');
            let _result = EthereumExtendedPublicKey::from_str(&string).unwrap();
        }

        #[test]
        #[should_panic(expected = "MaximumChildDepthReached(255)")]
        fn ckd_pub_max_depth() {
            let mut extended_public_key = EthereumExtendedPublicKey::from_str(VALID_EXTENDED_PUBLIC_KEY).unwrap();
            for _ in 0..255 {
                extended_public_key = extended_public_key.ckd_pub(0).expect("Error deriving child key");
            }
            assert_eq!(extended_public_key.to_string(), VALID_EXTENDED_PUBLIC_KEY_FINAL);
            let _result = extended_public_key.ckd_pub(0).unwrap();
        }

        #[test]
        #[should_panic(expected = "InvalidDerivationPath(\"m\", \"\")")]
        fn derivation_path_invalid() {
            let extended_public_key = EthereumExtendedPublicKey::from_str(VALID_EXTENDED_PUBLIC_KEY).unwrap();
            let _result = extended_public_key.derivation_path(INVALID_PATH).unwrap();
        }

        #[test]
        #[should_panic(expected = "InvalidDerivationPath(\"number\", \"a\")")]
        fn derivation_path_invalid_digit_normal() {
            let extended_public_key = EthereumExtendedPublicKey::from_str(VALID_EXTENDED_PUBLIC_KEY).unwrap();
            let _result = extended_public_key.derivation_path(INVALID_PATH_NORMAL).unwrap();
        }

        #[test]
        #[should_panic(expected = "InvalidDerivationPath(\"\", \"\\'\")")]
        fn derivation_path_invalid_digit_hardened() {
            let extended_public_key = EthereumExtendedPublicKey::from_str(VALID_EXTENDED_PUBLIC_KEY).unwrap();
            let _result = extended_public_key.derivation_path(INVALID_PATH_HARDENED).unwrap();
        }
    }
}