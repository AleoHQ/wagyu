use crate::wordlist::MoneroWordlist;
use wagyu_model::{monero::ESPERANTO, wordlist::Wordlist};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Esperanto;

impl Wordlist for Esperanto {}

impl MoneroWordlist for Esperanto {
    /// The wordlist in original form.
    const WORDLIST: &'static str = ESPERANTO;
    /// The prefix length for computing the checksum.
    const PREFIX_LENGTH: usize = 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WORD: &str = "speciala";
    const VALID_WORD_INDEX: usize = 1414;
    const INVALID_WORD: &str = "a";
    const INVALID_WORD_INDEX: usize = 3400;

    #[test]
    fn get() {
        // Valid case
        assert_eq!(VALID_WORD, Esperanto::get(VALID_WORD_INDEX).unwrap());
        // Invalid case
        assert!(Esperanto::get(INVALID_WORD_INDEX).is_err());
    }

    #[test]
    fn get_index() {
        // Valid case
        assert_eq!(VALID_WORD_INDEX, Esperanto::get_index(VALID_WORD).unwrap());
        // Invalid case
        assert!(Esperanto::get_index(INVALID_WORD).is_err());
    }

    #[test]
    fn get_all() {
        let list = Esperanto::get_all();
        assert_eq!(1626, list.len());
        assert_eq!(VALID_WORD, list[VALID_WORD_INDEX]);
    }
}
