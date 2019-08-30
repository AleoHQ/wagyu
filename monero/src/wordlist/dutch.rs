use crate::wordlist::MoneroWordlist;
use wagyu_model::{monero::DUTCH, wordlist::Wordlist};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dutch;

impl Wordlist for Dutch {}

impl MoneroWordlist for Dutch {
    /// The wordlist in original form.
    const WORDLIST: &'static str = DUTCH;
    /// The prefix length for computing the checksum.
    const PREFIX_LENGTH: usize = 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WORD: &str = "uurcirkel";
    const VALID_WORD_INDEX: usize = 1413;
    const INVALID_WORD: &str = "keuken";
    const INVALID_WORD_INDEX: usize = 3400;

    #[test]
    fn get() {
        // Valid case
        assert_eq!(VALID_WORD, Dutch::get(VALID_WORD_INDEX).unwrap());
        // Invalid case
        assert!(Dutch::get(INVALID_WORD_INDEX).is_err());
    }

    #[test]
    fn get_index() {
        // Valid case
        assert_eq!(VALID_WORD_INDEX, Dutch::get_index(VALID_WORD).unwrap());
        // Invalid case
        assert!(Dutch::get_index(INVALID_WORD).is_err());
    }

    #[test]
    fn get_all() {
        let list = Dutch::get_all();
        assert_eq!(1626, list.len());
        assert_eq!(VALID_WORD, list[VALID_WORD_INDEX]);
    }
}
