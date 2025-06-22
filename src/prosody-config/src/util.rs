// prosody-config
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// `"10MB"` -> `("10", "MB")`
pub(crate) fn split_leading_digits(s: &str) -> (&str, &str) {
    let split_index = (s.char_indices())
        .find(|&(_, c)| !c.is_ascii_digit())
        .map(|(i, _)| i)
        .unwrap_or(s.len()); // If all are digits

    s.split_at(split_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_leading_digits() {
        assert_eq!(split_leading_digits("10MB"), ("10", "MB"));
        assert_eq!(split_leading_digits("10"), ("10", ""));
        assert_eq!(split_leading_digits("MB"), ("", "MB"));
        assert_eq!(split_leading_digits(""), ("", ""));
        assert_eq!(split_leading_digits("10Mb/s"), ("10", "Mb/s"));
        assert_eq!(split_leading_digits("10MB123"), ("10", "MB123"));
    }
}
