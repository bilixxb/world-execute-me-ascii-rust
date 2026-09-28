//! Character-width aware string helpers — port of `player.py`'s `cw`/`width`/
//! `crop`/`wrap`.

use unicode_width::UnicodeWidthChar;

/// Python `cw(ch)`: 0 for combining marks, 2 for East Asian W/F, else 1.
pub fn cw(ch: char) -> usize {
    // `unicodedata.combining(ch)` covers Mn/Me plus a few others; the
    // unicode-width crate reports those as zero width, which is the same
    // outcome we need here.
    match UnicodeWidthChar::width(ch) {
        Some(0) | None => 0,
        Some(w) => w,
    }
}

/// Python `width(s)`.
pub fn width(s: &str) -> usize {
    s.chars().map(cw).sum()
}

/// Python `crop(s, n)` — truncate to at most `n` display columns.
///
/// Note the original appends the whole character and never has to pad, because
/// it breaks as soon as the next character would overflow.
pub fn crop(s: &str, n: usize) -> &str {
    let mut used = 0usize;
    for (idx, ch) in s.char_indices() {
        let k = cw(ch);
        if used + k > n {
            return &s[..idx];
        }
        used += k;
    }
    s
}

fn is_ascii(s: &str) -> bool {
    s.is_ascii()
}

/// Python `wrap(s, n)` — greedy wrap, backing up to the last space for pure
/// ASCII text so words are not split mid-token.
pub fn wrap(s: &str, n: usize) -> Vec<String> {
    if width(s) <= n {
        return vec![s.to_string()];
    }
    let mut parts = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        let mut line = crop(rest, n).to_string();
        if line.len() < rest.len() && line.contains(' ') && is_ascii(rest) {
            if let Some(pos) = line.rfind(' ') {
                line.truncate(pos);
            }
        }
        let consumed = line.len();
        parts.push(line);
        rest = rest[consumed..].trim_start_matches(|c: char| c.is_whitespace());
        // A pathological case: a single wide char wider than `n` consumes
        // nothing, which would spin forever. The original cannot hit this
        // because `crop` always stops before exceeding `n`, but a zero-width
        // first character would. Guard explicitly.
        if consumed == 0 && !rest.is_empty() {
            let mut it = rest.chars();
            it.next();
            rest = it.as_str();
        }
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_match_unicodedata() {
        assert_eq!(width("abc"), 3);
        assert_eq!(width("如果我是一组点"), 14);
        assert_eq!(width(""), 0);
        assert_eq!(cw('A'), 1);
        assert_eq!(cw('我'), 2);
    }

    #[test]
    fn crop_never_splits_a_wide_char() {
        // Expectations verified against the Python original:
        // `crop('如果我是一组点', 3) == '如'` (each CJK char is 2 columns).
        assert_eq!(crop("如果我是一组点", 3), "如");
        assert_eq!(crop("如果我是一组点", 4), "如果");
        assert_eq!(crop("如果我是一组点", 6), "如果我");
        assert_eq!(crop("abc", 2), "ab");
        assert_eq!(crop("abc", 10), "abc");
    }

    #[test]
    fn wrap_prefers_word_boundaries_for_ascii() {
        assert_eq!(wrap("hello world", 20), vec!["hello world"]);
        // The original crops to the column limit first, then backs up to the
        // last space — so 'hello world' at width 11 becomes 'hello', and the
        // remainder 'world again' then fits.
        assert_eq!(wrap("hello world again", 11), vec!["hello", "world again"]);
    }

    #[test]
    fn wrap_cjk_splits_by_columns() {
        assert_eq!(
            wrap("如果我是一组点", 6),
            vec!["如果我", "是一组", "点"]
        );
    }
}
