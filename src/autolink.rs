// Copied from https://git.cypr.io/oz/autolink-rust
// Copyright (c) 2015 Arnaud Berthomier
// Updated to use regex from https://mathiasbynens.be/demo/url-regex

use regex::Regex;
use std::borrow::Cow;

/// Wrap URLs in text with HTML A tags.
///
/// # Examples
///
/// ```
///   use phroxy::autolink::auto_link;
///
///   let before = "Share code on https://crates.io";
///   let after = "Share code on <a href=\"https://crates.io\">https://crates.io</a>";
///   assert!(auto_link(before) == after)
/// ```
pub fn auto_link(text: &str) -> Cow<str> {
    if text.len() == 0 {
        return Cow::from("");
    }

    let re =
        Regex::new(r"\b(([\w-]+://?|www[.])[^\s()<>]+(?:\([\w\d]+\)|([^[:punct:]\s]|/)))").unwrap();

    let replace_str = "<a href=\"$0\">$0</a>";
    re.replace_all(text, &replace_str as &str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert!(auto_link("") == "")
    }

    #[test]
    fn test_string_without_urls() {
        let src = "<p>Some HTML</p>";
        assert!(auto_link(src) == src)
    }

    #[test]
    fn test_string_with_http_urls() {
        let src = "Check this out: https://doc.rust-lang.org/\n
               https://fr.wikipedia.org/wiki/Caf%C3%A9ine";
        let linked = "Check this out: <a href=\"https://doc.rust-lang.org/\">https://doc.rust-lang.org/</a>\n
               <a href=\"https://fr.wikipedia.org/wiki/Caf%C3%A9ine\">https://fr.wikipedia.org/wiki/Caf%C3%A9ine</a>";
        assert!(auto_link(src) == linked)
    }

    #[test]
    fn test_trailing_characters() {
        let src = "I miss the old http://google.com. Don't you?";
        let linked =
            "I miss the old <a href=\"http://google.com\">http://google.com</a>. Don't you?";
        assert!(auto_link(src) == linked)
    }
}
