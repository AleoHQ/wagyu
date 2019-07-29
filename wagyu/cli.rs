//! # Wagu CLI
//!
//! A command-line tool to generate cryptocurrency wallets.

use bitcoin::address::Format as BitcoinFormat;
use bitcoin::{BitcoinAddress, BitcoinPrivateKey, Mainnet as BitcoinMainnet, Testnet as BitcoinTestnet};
use ethereum::{EthereumAddress, EthereumPrivateKey};
use monero::address::Format as MoneroFormat;
use monero::{Mainnet as MoneroMainnet, MoneroAddress, MoneroPrivateKey, Testnet as MoneroTestnet};
use wagyu_model::{Address, PrivateKey};
use zcash::address::Format as ZcashFormat;
use zcash::{Mainnet as ZcashMainnet, Testnet as ZcashTestnet, ZcashAddress, ZcashPrivateKey};

use clap::{App, Arg};
use rand::rngs::StdRng;
use rand_core::SeedableRng;
use serde::Serialize;
use std::marker::PhantomData;

fn main() {
    let network_vals = ["mainnet", "testnet"];
    let matches = App::new("wagyu")
        .version("v0.6.0")
        .about("Generate a wallet for Bitcoin, Ethereum, Monero, and Zcash")
        .author("Argus <team@argus.dev>")
        .arg(
            Arg::with_name("currency")
                .required(true)
                .help("Name of the currency to generate a wallet for (e.g. bitcoin, ethereum, monero, zcash)"),
        )
        .arg(
            Arg::with_name("network")
                .short("N")
                .long("network")
                .takes_value(true)
                .possible_values(&network_vals)
                .help("Network of wallet(s) to generate (e.g. mainnet, testnet)"),
        )
        .arg(
            Arg::with_name("count")
                .short("n")
                .long("count")
                .takes_value(true)
                .help("Number of wallets to generate"),
        )
        .arg(
            Arg::with_name("compressed")
                .short("c")
                .long("compressed")
                .help("Enabling this flag generates a wallet which corresponds to a compressed public key"),
        )
        .arg(
            Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Enabling this flag prints the wallet in JSON format"),
        )
        .arg(
            Arg::with_name("segwit")
                .long("segwit")
                .conflicts_with("network")
                .help("Enabling this flag generates a wallet with a SegWit address"),
        )
        .arg(
            Arg::with_name("bech32")
                .long("bech32")
                .conflicts_with("segwit")
                .help("Enabling this flag generates a wallet with a Bech32 (SegWit enabled) address"),
        )
        .get_matches();

    let currency = matches.value_of("currency").unwrap();
    //    let mut compressed = matches.is_present("compressed");
    let json = matches.is_present("json");
    let count = clap::value_t!(matches.value_of("count"), usize).unwrap_or_else(|_e| 1);
    let bitcoin_address_type = if matches.is_present("segwit") {
        //        compressed = true;
        BitcoinFormat::P2SH_P2WPKH
    } else if matches.is_present("bech32") {
        BitcoinFormat::Bech32
    } else {
        BitcoinFormat::P2PKH
    };
    let zcash_address_type = if matches.is_present("shielded") {
        ZcashFormat::Sprout
    } else {
        ZcashFormat::P2PKH
    };
    let testnet = match matches.value_of("network") {
        Some("mainnet") => false,
        Some("testnet") => true,
        _ => false,
    };

    match currency {
        "bitcoin" => print_bitcoin_wallet(count, testnet, &bitcoin_address_type, json),
        "ethereum" => print_ethereum_wallet(count, json),
        "monero" => print_monero_wallet(count, testnet, json),
        "zcash" => print_zcash_wallet(count, testnet, &zcash_address_type, json),
        _ => panic!("Unsupported currency"),
    };
}

fn print_bitcoin_wallet(count: usize, testnet: bool, format: &BitcoinFormat, json: bool) {
    #[derive(Serialize, Debug)]
    pub struct Wallet {
        private_key: String,
        address: String,
        network: String,
        compressed: bool,
    };

    let wallet = if testnet {
        let rng = &mut StdRng::from_entropy();
        let private_key = BitcoinPrivateKey::<BitcoinTestnet>::new(rng).unwrap();
        let address = BitcoinAddress::from_private_key(&private_key, &format).unwrap();

        Wallet {
            private_key: private_key.to_string(),
            address: address.to_string(),
            network: "testnet".into(),
            compressed: private_key.is_compressed(),
        }
    } else {
        let rng = &mut StdRng::from_entropy();
        let private_key = BitcoinPrivateKey::<BitcoinMainnet>::new(rng).unwrap();
        let address = BitcoinAddress::from_private_key(&private_key, &format).unwrap();

        Wallet {
            private_key: private_key.to_string(),
            address: address.to_string(),
            network: "mainnet".into(),
            compressed: private_key.is_compressed(),
        }
    };

    for _ in 0..count {
        if json {
            println!("{}", serde_json::to_string_pretty(&wallet).unwrap())
        } else {
            println!(
                "
        Private Key:    {}
        Address:        {}
        Network:        {}
        Compressed:     {}
        ",
                wallet.private_key, wallet.address, wallet.network, wallet.compressed
            )
        }
    }
}

fn print_ethereum_wallet(count: usize, json: bool) {
    #[derive(Serialize, Debug)]
    pub struct Wallet {
        private_key: String,
        address: String,
    };

    let rng = &mut StdRng::from_entropy();
    let private_key = EthereumPrivateKey::new(rng).unwrap();
    let address = EthereumAddress::from_private_key(&private_key, &PhantomData).unwrap();

    let wallet = Wallet {
        private_key: private_key.to_string(),
        address: address.to_string(),
    };

    for _ in 0..count {
        if json {
            println!("{}", serde_json::to_string_pretty(&wallet).unwrap())
        } else {
            println!(
                "
        Private Key:    {}
        Address:        {}
        ",
                wallet.private_key, wallet.address
            )
        }
    }
}

fn print_monero_wallet(count: usize, testnet: bool, json: bool) {
    #[derive(Serialize, Debug)]
    pub struct Wallet {
        private_key: String,
        address: String,
        network: String,
    };

    // TODO (howardwu): Add support for all Monero formats.
    let wallet = if testnet {
        let rng = &mut StdRng::from_entropy();
        let private_key = MoneroPrivateKey::<MoneroTestnet>::new(rng).unwrap();
        let address = MoneroAddress::from_private_key(&private_key, &MoneroFormat::Standard).unwrap();

        Wallet {
            private_key: private_key.to_string(),
            address: address.to_string(),
            network: "testnet".into(),
        }
    } else {
        let rng = &mut StdRng::from_entropy();
        let private_key = MoneroPrivateKey::<MoneroMainnet>::new(rng).unwrap();
        let address = MoneroAddress::from_private_key(&private_key, &MoneroFormat::Standard).unwrap();

        Wallet {
            private_key: private_key.to_string(),
            address: address.to_string(),
            network: "mainnet".into(),
        }
    };

    for _ in 0..count {
        if json {
            println!("{}", serde_json::to_string_pretty(&wallet).unwrap())
        } else {
            println!(
                "
        Private ( Spend, View ) Key:    {}
        Address:              {}
        ",
                wallet.private_key, wallet.address
            )
        }
    }
}

fn print_zcash_wallet(count: usize, testnet: bool, format: &ZcashFormat, json: bool) {
    #[derive(Serialize, Debug)]
    pub struct Wallet {
        private_key: String,
        address: String,
        network: String,
    };

    let wallet = if testnet {
        let rng = &mut StdRng::from_entropy();
        let private_key = ZcashPrivateKey::<ZcashTestnet>::new(rng).unwrap();
        let address = ZcashAddress::from_private_key(&private_key, &format).unwrap();

        Wallet {
            private_key: private_key.to_string(),
            address: address.to_string(),
            network: "testnet".into(),
        }
    } else {
        let rng = &mut StdRng::from_entropy();
        let private_key = ZcashPrivateKey::<ZcashMainnet>::new(rng).unwrap();
        let address = ZcashAddress::from_private_key(&private_key, &format).unwrap();

        Wallet {
            private_key: private_key.to_string(),
            address: address.to_string(),
            network: "mainnet".into(),
        }
    };

    for _ in 0..count {
        if json {
            println!("{}", serde_json::to_string_pretty(&wallet).unwrap())
        } else {
            println!(
                "
        Private Key:    {}
        Address:        {}
        Network:        {}
        ",
                wallet.private_key, wallet.address, wallet.network
            )
        }
    }
}
