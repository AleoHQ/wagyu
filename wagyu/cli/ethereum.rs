use crate::cli::{flag, option, subcommand, types::*, CLIError, CLI};
use crate::ethereum::{
    wordlist::*, EthereumAddress, EthereumDerivationPath, EthereumExtendedPrivateKey, EthereumExtendedPublicKey,
    EthereumMnemonic, EthereumPrivateKey, EthereumPublicKey,
};
use crate::model::{ExtendedPrivateKey, ExtendedPublicKey, Mnemonic, MnemonicExtended, PrivateKey, PublicKey};

use clap::ArgMatches;
use colored::*;
use rand::{rngs::StdRng, Rng};
use rand_core::SeedableRng;
use serde::Serialize;
use std::{fmt, fmt::Display, marker::PhantomData, str::FromStr};

/// Represents a generic wallet to output
#[derive(Serialize, Debug, Default)]
struct EthereumWallet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    pub address: String,
}

impl EthereumWallet {
    pub fn new<R: Rng>(rng: &mut R) -> Result<Self, CLIError> {
        let private_key = EthereumPrivateKey::new(rng)?;
        let public_key = private_key.to_public_key();
        let address = public_key.to_address(&PhantomData)?;
        Ok(Self {
            private_key: Some(private_key.to_string()),
            public_key: Some(public_key.to_string()),
            address: address.to_string(),
            ..Default::default()
        })
    }

    pub fn new_hd<W: EthereumWordlist, R: Rng>(
        rng: &mut R,
        word_count: u8,
        password: Option<&str>,
        path: &str,
    ) -> Result<Self, CLIError> {
        let mnemonic = EthereumMnemonic::<W>::new(word_count, rng)?;
        let master_extended_private_key = mnemonic.to_extended_private_key(password)?;
        let derivation_path = EthereumDerivationPath::from_str(path)?;
        let extended_private_key = master_extended_private_key.derive(&derivation_path)?;
        let extended_public_key = extended_private_key.to_extended_public_key();
        let private_key = extended_private_key.to_private_key();
        let public_key = extended_public_key.to_public_key();
        let address = public_key.to_address(&PhantomData)?;
        Ok(Self {
            path: Some(path.to_string()),
            password: password.map(String::from),
            mnemonic: Some(mnemonic.to_string()),
            extended_private_key: Some(extended_private_key.to_string()),
            extended_public_key: Some(extended_public_key.to_string()),
            private_key: Some(private_key.to_string()),
            public_key: Some(public_key.to_string()),
            address: address.to_string(),
        })
    }

    pub fn from_mnemonic<W: EthereumWordlist>(
        mnemonic: &str,
        password: Option<&str>,
        path: &str,
    ) -> Result<Self, CLIError> {
        let mnemonic = EthereumMnemonic::<W>::from_phrase(&mnemonic)?;
        let master_extended_private_key = mnemonic.to_extended_private_key(password)?;
        let derivation_path = EthereumDerivationPath::from_str(path)?;
        let extended_private_key = master_extended_private_key.derive(&derivation_path)?;
        let extended_public_key = extended_private_key.to_extended_public_key();
        let private_key = extended_private_key.to_private_key();
        let public_key = extended_public_key.to_public_key();
        let address = public_key.to_address(&PhantomData)?;
        Ok(Self {
            path: Some(path.to_string()),
            password: password.map(String::from),
            mnemonic: Some(mnemonic.to_string()),
            extended_private_key: Some(extended_private_key.to_string()),
            extended_public_key: Some(extended_public_key.to_string()),
            private_key: Some(private_key.to_string()),
            public_key: Some(public_key.to_string()),
            address: address.to_string(),
        })
    }

    pub fn from_extended_private_key(extended_private_key: &str, path: &Option<String>) -> Result<Self, CLIError> {
        let mut extended_private_key = EthereumExtendedPrivateKey::from_str(extended_private_key)?;
        if let Some(derivation_path) = path {
            let derivation_path = EthereumDerivationPath::from_str(&derivation_path)?;
            extended_private_key = extended_private_key.derive(&derivation_path)?;
        }
        let extended_public_key = extended_private_key.to_extended_public_key();
        let private_key = extended_private_key.to_private_key();
        let public_key = extended_public_key.to_public_key();
        let address = public_key.to_address(&PhantomData)?;
        Ok(Self {
            path: path.clone(),
            extended_private_key: Some(extended_private_key.to_string()),
            extended_public_key: Some(extended_public_key.to_string()),
            private_key: Some(private_key.to_string()),
            public_key: Some(public_key.to_string()),
            address: address.to_string(),
            ..Default::default()
        })
    }

    pub fn from_extended_public_key(extended_public_key: &str, path: &Option<String>) -> Result<Self, CLIError> {
        let mut extended_public_key = EthereumExtendedPublicKey::from_str(extended_public_key)?;
        if let Some(derivation_path) = path {
            let derivation_path = EthereumDerivationPath::from_str(&derivation_path)?;
            extended_public_key = extended_public_key.derive(&derivation_path)?;
        }
        let public_key = extended_public_key.to_public_key();
        let address = public_key.to_address(&PhantomData)?;
        Ok(Self {
            path: path.clone(),
            extended_public_key: Some(extended_public_key.to_string()),
            public_key: Some(public_key.to_string()),
            address: address.to_string(),
            ..Default::default()
        })
    }

    pub fn from_private_key(private_key: &str) -> Result<Self, CLIError> {
        let private_key = EthereumPrivateKey::from_str(private_key)?;
        let public_key = private_key.to_public_key();
        let address = public_key.to_address(&PhantomData)?;
        Ok(Self {
            private_key: Some(private_key.to_string()),
            public_key: Some(public_key.to_string()),
            address: address.to_string(),
            ..Default::default()
        })
    }

    pub fn from_public_key(public_key: &str) -> Result<Self, CLIError> {
        let public_key = EthereumPublicKey::from_str(public_key)?;
        let address = public_key.to_address(&PhantomData)?;
        Ok(Self {
            public_key: Some(public_key.to_string()),
            address: address.to_string(),
            ..Default::default()
        })
    }

    pub fn from_address(address: &str) -> Result<Self, CLIError> {
        let address = EthereumAddress::from_str(address)?;
        Ok(Self {
            address: address.to_string(),
            ..Default::default()
        })
    }
}

#[cfg_attr(tarpaulin, skip)]
impl Display for EthereumWallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = [
            match &self.path {
                Some(path) => format!("      {}                 {}\n", "Path".cyan().bold(), path),
                _ => "".to_owned(),
            },
            match &self.password {
                Some(password) => format!("      {}             {}\n", "Password".cyan().bold(), password),
                _ => "".to_owned(),
            },
            match &self.mnemonic {
                Some(mnemonic) => format!("      {}             {}\n", "Mnemonic".cyan().bold(), mnemonic),
                _ => "".to_owned(),
            },
            match &self.extended_private_key {
                Some(extended_private_key) => format!(
                    "      {} {}\n",
                    "Extended Private Key".cyan().bold(),
                    extended_private_key
                ),
                _ => "".to_owned(),
            },
            match &self.extended_public_key {
                Some(extended_public_key) => format!(
                    "      {}  {}\n",
                    "Extended Public Key".cyan().bold(),
                    extended_public_key
                ),
                _ => "".to_owned(),
            },
            match &self.private_key {
                Some(private_key) => format!("      {}          {}\n", "Private Key".cyan().bold(), private_key),
                _ => "".to_owned(),
            },
            match &self.public_key {
                Some(public_key) => format!("      {}           {}\n", "Public Key".cyan().bold(), public_key),
                _ => "".to_owned(),
            },
            format!("      {}              {}\n", "Address".cyan().bold(), self.address),
        ]
        .concat();

        // Removes final new line character
        let output = output[..output.len() - 1].to_owned();
        write!(f, "\n{}", output)
    }
}

/// Represents options for an Ethereum wallet
#[derive(Clone, Debug, Serialize)]
pub struct EthereumOptions {
    // Standard command
    count: usize,
    json: bool,
    subcommand: Option<String>,
    // HD and Import HD subcommands
    derivation: String,
    extended_private_key: Option<String>,
    extended_public_key: Option<String>,
    index: u32,
    language: String,
    mnemonic: Option<String>,
    password: Option<String>,
    path: Option<String>,
    word_count: u8,
    // Import subcommand
    address: Option<String>,
    private: Option<String>,
    public: Option<String>,
}

impl Default for EthereumOptions {
    fn default() -> Self {
        Self {
            // Standard command
            count: 1,
            json: false,
            subcommand: None,
            // HD and Import HD subcommands
            derivation: "ethereum".into(),
            extended_private_key: None,
            extended_public_key: None,
            index: 0,
            language: "english".into(),
            mnemonic: None,
            password: None,
            path: None,
            word_count: 12,
            // Import subcommand
            address: None,
            private: None,
            public: None,
        }
    }
}

impl EthereumOptions {
    fn parse(&mut self, arguments: &ArgMatches, options: &[&str]) {
        options.iter().for_each(|option| match *option {
            "address" => self.address(arguments.value_of(option)),
            "count" => self.count(clap::value_t!(arguments.value_of(*option), usize).ok()),
            "derivation" => self.derivation(arguments.value_of(option)),
            "extended private" => self.extended_private(arguments.value_of(option)),
            "extended public" => self.extended_public(arguments.value_of(option)),
            "json" => self.json(arguments.is_present(option)),
            "index" => self.index(clap::value_t!(arguments.value_of(*option), u32).ok()),
            "language" => self.language(arguments.value_of(option)),
            "mnemonic" => self.mnemonic(arguments.value_of(option)),
            "password" => self.password(arguments.value_of(option)),
            "private" => self.private(arguments.value_of(option)),
            "public" => self.public(arguments.value_of(option)),
            "word count" => self.word_count(clap::value_t!(arguments.value_of(*option), u8).ok()),
            _ => (),
        });
    }

    /// Imports a wallet for the specified address, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn address(&mut self, argument: Option<&str>) {
        if let Some(address) = argument {
            self.address = Some(address.to_string());
        }
    }

    /// Sets `count` to the specified count, overriding its previous state.
    fn count(&mut self, argument: Option<usize>) {
        if let Some(count) = argument {
            self.count = count;
        }
    }

    /// Sets `derivation` to the specified derivation, overriding its previous state.
    /// If `derivation` is `\"custom\"`, then `path` is set to the specified path.
    /// If the specified argument is `None`, then no change occurs.
    fn derivation(&mut self, argument: Option<&str>) {
        match argument {
            Some("ethereum") => self.derivation = "ethereum".into(),
            Some("keepkey") => self.derivation = "keepkey".into(),
            Some("ledger-legacy") => self.derivation = "ledger-legacy".into(),
            Some("ledger-live") => self.derivation = "ledger-legacy".into(),
            Some("trezor") => self.derivation = "trezor".into(),
            Some(custom) => {
                self.derivation = "custom".into();
                self.path = Some(custom.to_string());
            }
            _ => (),
        };
    }

    /// Sets `extended_private_key` to the specified extended private key, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn extended_private(&mut self, argument: Option<&str>) {
        if let Some(extended_private_key) = argument {
            self.extended_private_key = Some(extended_private_key.to_string());
        }
    }

    /// Sets `extended_public_key` to the specified extended public key, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn extended_public(&mut self, argument: Option<&str>) {
        if let Some(extended_public_key) = argument {
            self.extended_public_key = Some(extended_public_key.to_string());
        }
    }

    /// Sets `index` to the specified index, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn index(&mut self, argument: Option<u32>) {
        if let Some(index) = argument {
            self.index = index;
        }
    }

    /// Sets `json` to the specified boolean value, overriding its previous state.
    fn json(&mut self, argument: bool) {
        self.json = argument;
    }

    /// Sets `language` to the specified language, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn language(&mut self, argument: Option<&str>) {
        match argument {
            Some("chinese_simplified") => self.language = "chinese_simplified".into(),
            Some("chinese_traditional") => self.language = "chinese_traditional".into(),
            Some("english") => self.language = "english".into(),
            Some("french") => self.language = "french".into(),
            Some("italian") => self.language = "italian".into(),
            Some("japanese") => self.language = "japanese".into(),
            Some("korean") => self.language = "korean".into(),
            Some("spanish") => self.language = "spanish".into(),
            _ => (),
        };
    }

    /// Sets `mnemonic` to the specified mnemonic, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn mnemonic(&mut self, argument: Option<&str>) {
        if let Some(mnemonic) = argument {
            self.mnemonic = Some(mnemonic.to_string());
        }
    }

    /// Sets `password` to the specified password, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn password(&mut self, argument: Option<&str>) {
        if let Some(password) = argument {
            self.password = Some(password.to_string());
        }
    }

    /// Imports a wallet for the specified private key, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn private(&mut self, argument: Option<&str>) {
        if let Some(private_key) = argument {
            self.private = Some(private_key.to_string());
        }
    }

    /// Imports a wallet for the specified public key, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn public(&mut self, argument: Option<&str>) {
        if let Some(public_key) = argument {
            self.public = Some(public_key.to_string())
        }
    }

    /// Sets `word_count` to the specified word count, overriding its previous state.
    /// If the specified argument is `None`, then no change occurs.
    fn word_count(&mut self, argument: Option<u8>) {
        if let Some(word_count) = argument {
            self.word_count = word_count;
        }
    }

    /// Returns the derivation path with the specified account, chain, derivation, index, and path.
    /// If `default` is enabled, then return the default path if no derivation was provided.
    fn to_derivation_path(&self, default: bool) -> Option<String> {
        match self.derivation.as_str() {
            "ethereum" => Some(format!("m/44'/60'/0'/{}", self.index)),
            "keepkey" => Some(format!("m/44'/60'/{}'/0", self.index)),
            "ledger-legacy" => Some(format!("m/44'/60'/0'/{}", self.index)),
            "ledger-live" => Some(format!("m/44'/60'/{}'/0/0", self.index)),
            "trezor" => Some(format!("m/44'/60'/0'/{}", self.index)),
            "custom" => self.path.clone(),
            _ => match default {
                true => Some(format!("m/44'/60'/0'/{}", self.index)),
                false => None,
            },
        }
    }
}

pub struct EthereumCLI;

impl CLI for EthereumCLI {
    type Options = EthereumOptions;

    const NAME: NameType = "ethereum";
    const ABOUT: AboutType = "Generates a Ethereum wallet (include -h for more options)";
    const FLAGS: &'static [FlagType] = &[flag::JSON];
    const OPTIONS: &'static [OptionType] = &[option::COUNT];
    const SUBCOMMANDS: &'static [SubCommandType] = &[
        subcommand::HD_ETHEREUM,
        subcommand::IMPORT_ETHEREUM,
        subcommand::IMPORT_HD_ETHEREUM,
    ];

    /// Handle all CLI arguments and flags for Ethereum
    #[cfg_attr(tarpaulin, skip)]
    fn parse(arguments: &ArgMatches) -> Result<Self::Options, CLIError> {
        let mut options = EthereumOptions::default();
        options.parse(arguments, &["count", "json"]);

        match arguments.subcommand() {
            ("hd", Some(arguments)) => {
                options.subcommand = Some("hd".into());
                options.parse(arguments, &["count", "json"]);
                options.parse(arguments, &["derivation", "language", "password", "word count"]);
            }
            ("import", Some(arguments)) => {
                options.subcommand = Some("import".into());
                options.parse(arguments, &["json"]);
                options.parse(arguments, &["address", "private", "public"]);
            }
            ("import-hd", Some(arguments)) => {
                options.subcommand = Some("import-hd".into());
                options.parse(arguments, &["json"]);
                options.parse(
                    arguments,
                    &[
                        "account",
                        "chain",
                        "derivation",
                        "extended private",
                        "extended public",
                        "index",
                        "mnemonic",
                        "password",
                    ],
                );
            }
            _ => {}
        };

        Ok(options)
    }

    /// Generate the Ethereum wallet and print the relevant fields
    #[cfg_attr(tarpaulin, skip)]
    fn print(options: Self::Options) -> Result<(), CLIError> {
        fn output<W: EthereumWordlist>(options: EthereumOptions) -> Result<(), CLIError> {
            let wallets = match options.subcommand.as_ref().map(String::as_str) {
                Some("hd") => {
                    let path = options.to_derivation_path(true).unwrap();
                    (0..options.count)
                        .flat_map(|_| {
                            match EthereumWallet::new_hd::<W, _>(
                                &mut StdRng::from_entropy(),
                                options.word_count,
                                options.password.as_ref().map(String::as_str),
                                &path,
                            ) {
                                Ok(wallet) => vec![wallet],
                                _ => vec![],
                            }
                        })
                        .collect()
                }
                Some("import") => {
                    if let Some(private_key) = options.private {
                        vec![EthereumWallet::from_private_key(&private_key)?]
                    } else if let Some(public_key) = options.public {
                        vec![EthereumWallet::from_public_key(&public_key)?]
                    } else if let Some(address) = options.address {
                        vec![EthereumWallet::from_address(&address)?]
                    } else {
                        vec![]
                    }
                }
                Some("import-hd") => {
                    if let Some(mnemonic) = options.mnemonic.clone() {
                        fn process_mnemonic<EW: EthereumWordlist>(
                            mnemonic: &String,
                            options: &EthereumOptions,
                        ) -> Result<EthereumWallet, CLIError> {
                            EthereumWallet::from_mnemonic::<EW>(
                                &mnemonic,
                                options.password.as_ref().map(String::as_str),
                                &options.to_derivation_path(true).unwrap(),
                            )
                        }
                        vec![process_mnemonic::<ChineseSimplified>(&mnemonic, &options)
                            .or(process_mnemonic::<ChineseTraditional>(&mnemonic, &options))
                            .or(process_mnemonic::<English>(&mnemonic, &options))
                            .or(process_mnemonic::<French>(&mnemonic, &options))
                            .or(process_mnemonic::<Italian>(&mnemonic, &options))
                            .or(process_mnemonic::<Japanese>(&mnemonic, &options))
                            .or(process_mnemonic::<Korean>(&mnemonic, &options))
                            .or(process_mnemonic::<Spanish>(&mnemonic, &options))?]
                    } else if let Some(extended_private_key) = options.extended_private_key.clone() {
                        vec![EthereumWallet::from_extended_private_key(
                            &extended_private_key,
                            &options.to_derivation_path(false),
                        )?]
                    } else if let Some(extended_public_key) = options.extended_public_key.clone() {
                        vec![EthereumWallet::from_extended_public_key(
                            &extended_public_key,
                            &options.to_derivation_path(false),
                        )?]
                    } else {
                        vec![]
                    }
                }
                _ => (0..options.count)
                    .flat_map(|_| match EthereumWallet::new::<_>(&mut StdRng::from_entropy()) {
                        Ok(wallet) => vec![wallet],
                        _ => vec![],
                    })
                    .collect(),
            };

            match options.json {
                true => println!("{}\n", serde_json::to_string_pretty(&wallets)?),
                false => wallets.iter().for_each(|wallet| println!("{}\n", wallet)),
            };

            Ok(())
        }

        match options.language.as_str() {
            "chinese_simplified" => output::<ChineseSimplified>(options),
            "chinese_traditional" => output::<ChineseTraditional>(options),
            "english" => output::<English>(options),
            "french" => output::<French>(options),
            "italian" => output::<Italian>(options),
            "japanese" => output::<Japanese>(options),
            "korean" => output::<Korean>(options),
            "spanish" => output::<Spanish>(options),
            _ => output::<English>(options),
        }
    }
}
