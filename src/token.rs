use std::{fmt::Display, iter::from_fn, ops::Range};

use itertools::Itertools;

struct SourceLocation(Range<usize>);

trait TokenString {
    fn consume<'a, T>(self, options: impl IntoIterator<Item = (&'a str, T)>) -> Option<(usize, T)>;
    fn char_lengths(self) -> impl Iterator<Item = (usize, char)>;
}

impl TokenString for &str {
    fn consume<'a, T>(self, options: impl IntoIterator<Item = (&'a str, T)>) -> Option<(usize, T)> {
        for (prefix, t) in options {
            if self.starts_with(prefix) {
                return Some((prefix.len(), t));
            }
        }
        None
    }

    fn char_lengths(self) -> impl Iterator<Item = (usize, char)> {
        self.char_indices()
            .chain([(self.len(), '\0')])
            .tuple_windows()
            .map(|((_, c), (i, _))| (i, c))
    }
}

fn locate(source: &str, i: usize) -> String {
    let mut line_no = 0;
    let mut col_no = 0;
    for (j, c) in source.char_indices() {
        if j == i {
            return format!("{line_no}:{col_no}");
        }
        col_no += 1;
        if c == '\n' {
            line_no += 1;
            col_no = 0;
        }
    }
    panic!("invalid source offset {i}");
}

enum Kind {
    BraceClose,
    BraceOpen,
    Equal,
    FatArrow,
    Integer(i64),
    Name(String),
    Newline,
    ParenClose,
    ParenOpen,
    Plus,
}

struct Token {
    kind: Kind,
    location: SourceLocation,
}

fn tokenise(source: &str) -> impl Iterator<Item = Result<Token, String>> + '_ {
    type KindResult = (Result<Option<Kind>, String>, usize);
    fn find_offset(s: &str, mut f: impl FnMut(char) -> bool) -> usize {
        s.char_indices()
            .find(move |(_, c)| f(*c))
            .map(|(i, _)| i)
            .unwrap_or(s.len())
    }
    fn next_integer(s: &str) -> KindResult {
        let len = find_offset(s, |c| !c.is_ascii_digit());
        let s = &s[..len];
        let kind = s
            .parse()
            .map(Kind::Integer)
            .map(Some)
            .map_err(|_| format!("integer token too large {}", s));
        (kind, len)
    }
    fn next_name(s: &str) -> KindResult {
        let len = find_offset(s, |c| !c.is_alphanumeric() && c != '_');
        (Ok(Some(Kind::Name(s[..len].into()))), len)
    }
    fn next_token(s: &str) -> KindResult {
        if let Some((length, maybe_kind)) = s.consume([
            (" ", None),
            ("(", Some(Kind::ParenOpen)),
            (")", Some(Kind::ParenClose)),
            ("{", Some(Kind::BraceOpen)),
            ("}", Some(Kind::BraceClose)),
            ("\n", Some(Kind::Newline)),
            ("+", Some(Kind::Plus)),
            ("=>", Some(Kind::FatArrow)),
            ("=", Some(Kind::Equal)),
        ]) {
            return (Ok(maybe_kind), length);
        }
        let (j, c) = s.char_lengths().next().unwrap();
        match c {
            '0'..='9' => next_integer(s),
            c if c.is_alphabetic() => next_name(s),
            c => (Err(format!("unexpected token '{c}'")), j),
        }
    }
    let mut next_i = 0;
    from_fn(move || loop {
        if next_i >= source.len() {
            return None;
        }
        let s = &source[next_i..];
        let (result, length) = next_token(s);
        let prev_i = next_i;
        next_i += length;
        let result = match result {
            Err(msg) => Err(format!("{msg} at {}", locate(source, prev_i))),
            Ok(Some(kind)) => {
                let location = SourceLocation(prev_i..next_i);
                Ok(Token { kind, location })
            }
            Ok(None) => continue,
        };
        return Some(result);
    })
}

pub struct Tokens<'source> {
    source: &'source str,
    tokens: Vec<Token>,
}

impl<'source> Tokens<'source> {
    pub fn from_source(source: &'source str) -> Result<Self, String> {
        let mut errors = Vec::new();
        let mut tokens = Vec::new();
        for token in tokenise(source) {
            match token {
                Ok(token) => tokens.push(token),
                Err(msg) => errors.push(msg),
            }
        }
        if errors.is_empty() {
            Ok(Self { source, tokens })
        } else {
            Err(errors.join("\n"))
        }
    }
}

impl Display for Tokens<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, token) in self.tokens.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            match &token.kind {
                Kind::BraceClose => write!(f, "<BraceClose>")?,
                Kind::BraceOpen => write!(f, "<BraceOpen>")?,
                Kind::Equal => write!(f, "<Equal>")?,
                Kind::FatArrow => write!(f, "<FatArrow>")?,
                Kind::Integer(int) => write!(f, "<Integer {int}>")?,
                Kind::Name(name) => write!(f, "<Name {name}>")?,
                Kind::Newline => write!(f, "<Newline>")?,
                Kind::ParenClose => write!(f, "<ParenClose>")?,
                Kind::ParenOpen => write!(f, "<ParenOpen>")?,
                Kind::Plus => write!(f, "<Plus>")?,
            }
        }
        Ok(())
    }
}
