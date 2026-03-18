use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    House,
    Site,
    Style,
    Floor,
    Room,
    Roof,
    Facade,
    Landscape,
    Furniture,

    // Literals
    Ident(String),
    StringLit(String),
    Number(f64),

    // Symbols
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Colon,
    Comma,
    Dot,
    DotDot,
    Tilde,
    X, // dimension separator in "12m x 9m"

    // Units (attached to numbers during parsing)
    Unit(String),

    // End of file
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::House => write!(f, "house"),
            Self::Site => write!(f, "site"),
            Self::Style => write!(f, "style"),
            Self::Floor => write!(f, "floor"),
            Self::Room => write!(f, "room"),
            Self::Roof => write!(f, "roof"),
            Self::Facade => write!(f, "facade"),
            Self::Landscape => write!(f, "landscape"),
            Self::Furniture => write!(f, "furniture"),
            Self::Ident(s) => write!(f, "{s}"),
            Self::StringLit(s) => write!(f, "\"{s}\""),
            Self::Number(n) => write!(f, "{n}"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::DotDot => write!(f, ".."),
            Self::Tilde => write!(f, "~"),
            Self::X => write!(f, "x"),
            Self::Unit(u) => write!(f, "{u}"),
            Self::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

const UNITS: &[&str] = &["mm", "cm", "sqm", "sqft", "ft", "in", "m"];
const KEYWORDS: &[(&str, TokenKind)] = &[
    ("house", TokenKind::House),
    ("site", TokenKind::Site),
    ("style", TokenKind::Style),
    ("floor", TokenKind::Floor),
    ("room", TokenKind::Room),
    ("roof", TokenKind::Roof),
    ("facade", TokenKind::Facade),
    ("landscape", TokenKind::Landscape),
    ("furniture", TokenKind::Furniture),
];

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut line = 1;
    let mut col = 1;

    while pos < chars.len() {
        let ch = chars[pos];

        // Whitespace
        if ch.is_whitespace() {
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            pos += 1;
            continue;
        }

        // Line comment
        if ch == '/' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
            while pos < chars.len() && chars[pos] != '\n' {
                pos += 1;
            }
            continue;
        }

        // Block comment
        if ch == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
            pos += 2;
            col += 2;
            while pos + 1 < chars.len() && !(chars[pos] == '*' && chars[pos + 1] == '/') {
                if chars[pos] == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                pos += 1;
            }
            pos += 2; // skip */
            col += 2;
            continue;
        }

        let span = Span {
            line,
            col,
            offset: pos,
        };

        // String literal
        if ch == '"' {
            pos += 1;
            col += 1;
            let mut s = String::new();
            while pos < chars.len() && chars[pos] != '"' {
                s.push(chars[pos]);
                pos += 1;
                col += 1;
            }
            if pos < chars.len() {
                pos += 1; // closing quote
                col += 1;
            }
            tokens.push(Token {
                kind: TokenKind::StringLit(s),
                span,
            });
            continue;
        }

        // Number (possibly followed by unit)
        if ch.is_ascii_digit() || (ch == '-' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit()) {
            let start = pos;
            if ch == '-' {
                pos += 1;
            }
            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                pos += 1;
            }
            let num_str: String = chars[start..pos].iter().collect();
            let num: f64 = num_str
                .parse()
                .map_err(|_| LexError::new(&format!("invalid number: {num_str}"), line, col))?;

            tokens.push(Token {
                kind: TokenKind::Number(num),
                span: span.clone(),
            });
            col += pos - start;

            // Check for immediately-following unit
            let remaining: String = chars[pos..].iter().collect();
            // Sort units by length descending to match longest first
            let mut sorted_units: Vec<&&str> = UNITS.iter().collect();
            sorted_units.sort_by(|a, b| b.len().cmp(&a.len()));
            for unit in sorted_units {
                if remaining.starts_with(*unit) {
                    // Make sure the unit isn't part of a longer identifier
                    let after = pos + unit.len();
                    if after >= chars.len() || !chars[after].is_alphanumeric() {
                        tokens.push(Token {
                            kind: TokenKind::Unit(unit.to_string()),
                            span: Span {
                                line,
                                col,
                                offset: pos,
                            },
                        });
                        pos += unit.len();
                        col += unit.len();
                        break;
                    }
                }
            }
            continue;
        }

        // Symbols
        match ch {
            '{' => {
                tokens.push(Token {
                    kind: TokenKind::LBrace,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            '}' => {
                tokens.push(Token {
                    kind: TokenKind::RBrace,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            '[' => {
                tokens.push(Token {
                    kind: TokenKind::LBracket,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            ']' => {
                tokens.push(Token {
                    kind: TokenKind::RBracket,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            '(' => {
                tokens.push(Token {
                    kind: TokenKind::LParen,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            ')' => {
                tokens.push(Token {
                    kind: TokenKind::RParen,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            ':' => {
                tokens.push(Token {
                    kind: TokenKind::Colon,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            ',' => {
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            '.' => {
                if pos + 1 < chars.len() && chars[pos + 1] == '.' {
                    tokens.push(Token {
                        kind: TokenKind::DotDot,
                        span,
                    });
                    pos += 2;
                    col += 2;
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Dot,
                        span,
                    });
                    pos += 1;
                    col += 1;
                }
                continue;
            }
            '~' => {
                tokens.push(Token {
                    kind: TokenKind::Tilde,
                    span,
                });
                pos += 1;
                col += 1;
                continue;
            }
            _ => {}
        }

        // Identifier or keyword
        // Allow hyphens in identifiers (e.g., "east-west", "mid-century")
        if ch.is_alphabetic() || ch == '_' {
            let start = pos;
            while pos < chars.len()
                && (chars[pos].is_alphanumeric()
                    || chars[pos] == '_'
                    || (chars[pos] == '-'
                        && pos + 1 < chars.len()
                        && chars[pos + 1].is_alphabetic()))
            {
                pos += 1;
            }
            let word: String = chars[start..pos].iter().collect();
            col += pos - start;

            // Check for keyword
            if let Some((_, kind)) = KEYWORDS.iter().find(|(kw, _)| *kw == word.as_str()) {
                tokens.push(Token {
                    kind: kind.clone(),
                    span,
                });
            } else if word == "x" {
                // Could be dimension separator
                tokens.push(Token {
                    kind: TokenKind::X,
                    span,
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Ident(word),
                    span,
                });
            }
            continue;
        }

        return Err(LexError::new(
            &format!("unexpected character: '{ch}'"),
            line,
            col,
        ));
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            line,
            col,
            offset: pos,
        },
    });
    Ok(tokens)
}

#[derive(Debug, thiserror::Error)]
#[error("line {line}, col {col}: {message}")]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl LexError {
    fn new(message: &str, line: usize, col: usize) -> Self {
        Self {
            message: message.to_string(),
            line,
            col,
        }
    }
}
