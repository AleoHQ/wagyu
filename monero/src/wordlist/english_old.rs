use crate::wordlist::MoneroWordlist;
use wagyu_model::{monero::ENGLISH_OLD, wordlist::Wordlist};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnglishOld;

impl Wordlist for EnglishOld {}

impl MoneroWordlist for EnglishOld {
    /// The wordlist in original form.
    const WORDLIST: &'static str = ENGLISH_OLD;
    /// The prefix length for computing the checksum.
    const PREFIX_LENGTH: usize = 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WORD: &str = "destination";
    const VALID_WORD_INDEX: usize = 1576;
    const INVALID_WORD: &str = "abracadabra";
    const INVALID_WORD_INDEX: usize = 3400;

    #[test]
    fn get() {
        // Valid case
        assert_eq!(VALID_WORD, EnglishOld::get(VALID_WORD_INDEX).unwrap());
        // Invalid case
        assert!(EnglishOld::get(INVALID_WORD_INDEX).is_err());
    }

    #[test]
    fn get_index() {
        // Valid case
        assert_eq!(VALID_WORD_INDEX, EnglishOld::get_index(VALID_WORD).unwrap());
        // Invalid case
        assert!(EnglishOld::get_index(INVALID_WORD).is_err());
    }

    #[test]
    fn get_all() {
        let list = EnglishOld::get_all();
        assert_eq!(1626, list.len());
        assert_eq!(VALID_WORD, list[VALID_WORD_INDEX]);
    }
}
