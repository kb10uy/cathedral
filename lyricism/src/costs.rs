const SIGN_CHARS: &[char] = &[
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=',
    '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
];

const LEET_PAIRS: &[(char, char)] = &[('e', '3'), ('o', '0'), ('a', 'V')];

pub fn query_insert(c: char) -> usize {
    if c.is_ascii_whitespace() {
        1
    } else if SIGN_CHARS.contains(&c) {
        2
    } else {
        7
    }
}

pub fn query_delete(c: char) -> usize {
    if c.is_ascii_whitespace() {
        2
    } else {
        10
    }
}

pub fn query_replace(qc: char, tc: char) -> usize {
    if qc == tc {
        0
    } else if qc.to_ascii_uppercase() == tc {
        1
    } else if qc.to_ascii_lowercase() == tc {
        2
    } else if LEET_PAIRS.contains(&(qc, tc)) {
        3
    } else {
        4
    }
}

pub fn query_substring(s: &str, position: usize) -> isize {
    s.chars().count() as isize * -20 + (position as isize / 2)
}
