use model::{Address, crypto::{checksum, hash160}, PrivateKey};
use network::{Network, MAINNET_ADDRESS_BYTES, TESTNET_ADDRESS_BYTES};
use private_key::ZcashPrivateKey;
use public_key::ZcashPublicKey;

use base58::ToBase58;
use serde::Serialize;
use std::fmt;

/// Represents the format of a Zcash address
#[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Format {

    /// Unshielded Zcash Address
    Unshielded,

    /// Shielded Zcash Address
    Shielded,
}

/// Represents a Zcash t-address
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZcashAddress {

    /// The Zcash address
    pub address: String,

    /// The format of the address
    pub format: Format,

    /// The network on which this address is usable
    pub network: Network,
}

impl Address for ZcashAddress{
    type Format = (Format, Network);
    type PrivateKey = ZcashPrivateKey;
    type PublicKey = ZcashPublicKey;

    /// Returns the address corresponding to the given Zcash private key.
    fn from_private_key(private_key: &Self::PrivateKey, format: Option<Self::Format>) -> Self {
        match format {
            Some((Format::Unshielded, _)) => Self::unshielded(&private_key.to_public_key(), &private_key.network),
            Some((Format::Shielded, _)) => Self::shielded(&private_key.to_public_key(), &private_key.network),
            None => Self::unshielded(&private_key.to_public_key(), &private_key.network)
        }
    }

    /// Returns the address corresponding to the given Zcash public key.
    fn from_public_key(public_key: &Self::PublicKey, format: Option<Self::Format>) -> Self {
        match format {
            Some((Format::Unshielded, network)) => Self::unshielded(public_key, &network),
            Some((Format::Shielded, network)) => Self::shielded(public_key, &network),
            None => Self::unshielded(public_key, &Network::Mainnet)
        }
    }
}

impl ZcashAddress {

    /// Returns an unshielded address from a given Zcash public key
    fn unshielded(public_key: &ZcashPublicKey, network: &Network) -> Self {
        let public_key = match public_key.compressed {
            true => public_key.public_key.serialize().to_vec(),
            false => public_key.public_key.serialize_uncompressed().to_vec()
        };

        let network_bytes = match network {
            Network::Mainnet => MAINNET_ADDRESS_BYTES,
            Network::Testnet => TESTNET_ADDRESS_BYTES,
            _ => MAINNET_ADDRESS_BYTES,
        };

        let mut address_bytes = [0u8; 26];
        let ripemd160_hash = hash160(&public_key); // Ripemd160 Hash


        address_bytes[0] = network_bytes[0];
        address_bytes[1] = network_bytes[1];
        address_bytes[2..22].copy_from_slice(&ripemd160_hash);

        let checksum_bytes = checksum(&address_bytes[0..22]); // Calculate Checksum
        address_bytes[22..26].copy_from_slice(&checksum_bytes[0..4]); // Append Checksum Bytes

        Self {
            address: address_bytes.to_base58(),
            format: Format::Unshielded,
            network: network.clone(),
        }
    }

    /// TODO Returns a shielded address from a given Zcash public key
    fn shielded(_public_key: &ZcashPublicKey, _network: &Network) -> Self {
        panic!("shieled addresses not implemented");
    }
}

impl fmt::Display for ZcashAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_private_key_address_pairs(private_keys: [&str; 5], addresses: [&str; 5]) {
        let key_address_pairs = private_keys.iter().zip(addresses.iter());
        key_address_pairs.for_each(|(&private_key, &expected_address)| {
            let private_key = ZcashPrivateKey::from_wif(private_key).unwrap();
            let address = ZcashAddress::from_private_key(&private_key, Some((Format::Unshielded, private_key.network)));
            assert_eq!(address.address, expected_address);
        });
    }

    fn test_private_key_wif(private_key: &str, expected_address: &str) {
        let private_key = ZcashPrivateKey::from_wif(private_key).unwrap();
        let address = ZcashAddress::from_private_key(&private_key, Some((Format::Unshielded, private_key.network)));
        assert_eq!(address.address, expected_address);
    }

    #[test]
    fn test_mainnet_uncompressed_private_key_to_address() {
        let private_keys = [
            "5HwduFgmNrhcgXpD7TH2ZbqBzfET3FzRLwapJdZYUNyxPz6MYQU",
            "5KFcAbDaap4ZqF1pCTq6rKWU6bUZg3bnqHJYaCEh6NUu8aVTszm",
            "5KXotG2j5THVbdf2Uf87HPZFRNaVqZYrrBVnZzczyDVza39q94f",
            "5KPN7LeX6uzBpTYdC28xjgHkN5XbCKZVJiu9QquSCEFJcD7ndnv",
            "5JewwWXmgcdk9P762F3Pdr8RBcWfWVAAotq9mjSNBcEvZsQBJ32",
        ];
        let addresses = [
            "t1gxf6ykX23Ha3Bf1bKhjJzdxtCPratotJK",
            "t1QnYYpiVpmwHPtrRSJqypnDxG77284NUtj",
            "t1XEXEt3KeEYzycPTzn3invLivktYifWuXJ",
            "t1VxdN6a4T6RiSwgkNURkHhjLuoThvZWaHC",
            "t1XraTEGoX5QjnhAqDs9F8AqvDEh4zohhUQ",
        ];

        test_private_key_address_pairs(private_keys, addresses);
    }

    #[test]
    fn test_mainnet_compressed_private_key_to_address() {
        let private_keys = [
            "KxYzZuBPkE3rnEEGCdsB6dCzxN1D4xoY5ogKoxbdUdkxbRzvgbij",
            "KyuC6qNxMiuPEF4wp6eLsJuczLKqHsdsdSx5c3a1boY81mpahuR6",
            "KxNLHESzCRfzTfF9KGsF68QtV9fT9qFRAH5UKpVUdMvc4TTcBmhJ",
            "L5XgV3xUnqcqJyJm3JZmtZyj5i8FmUbuj9LCz9n3FA87Ertn2Qod",
            "L17dC6ZcGfKu84FGastka34sB8yV9fzgbKJaafVWi4zKs6ETnF2x",
        ];
        let addresses = [
            "t1MoMR1XdnPqLBWf5XkchWTkGNrveYLCaiM",
            "t1cnUnLfXZsb7gM7h9zD6QXm1wEDi4NxvTi",
            "t1VenYPx8HCiq6YFbuh1HbLGwtDZxQ5hQCr",
            "t1U9A7fh864FCzePbrXeUdjvuMfuCYKijbr",
            "t1J8w8EMM1Rs26zJFu3Deo6ougWhNhPXUZt",
        ];

        test_private_key_address_pairs(private_keys, addresses);
    }

    #[test]
    #[should_panic(expected = "Error deriving PrivateKey from WIF")]
    fn test_invalid_private_key_wif() {
        test_private_key_wif(
            "xYzZuBPkE3rnEEGCdsB6dCzxN1D4xoY5ogKoxbdUdkxbRzvgbij",
            ""
        )
    }
}
