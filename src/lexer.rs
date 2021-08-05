use logos::{Logos, Source};
use crate::new_parser::Operator;

#[inline]
fn trim<'a>(lex: &mut logos::Lexer<'a, Token<'a>>, begin: usize, end: usize) -> &'a str {
    let s = lex.slice();
    &s[begin..s.len()-end]
}

#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub enum Integer<'a> {
    Binary(&'a str),
    Decimal(&'a str),
    Hex(&'a str),
}

impl<'a> Integer<'a> {
    pub fn from_str(string: &'a str) -> Self {
        if string.starts_with("0x") || string.starts_with("0X") {
            Integer::Hex(string)
        } else if string.starts_with("0b") || string.starts_with("0B") {
            Integer::Binary(string)
        } else {
            Integer::Decimal(string)
        }
    }
    
    pub fn slice(&self) -> &'a str {
        match *self {
            Integer::Binary(s) => s,
            Integer::Decimal(s) => s,
            Integer::Hex(s) => s,
        }
    }
    
    pub fn as_int<T: num_traits::int::PrimInt>(&self) -> Option<T> {
        match *self {
            Self::Binary(b) => T::from_str_radix(&b[2..], 2).ok(),
            Self::Decimal(d) => T::from_str_radix(d, 10).ok(),
            Self::Hex(h) => T::from_str_radix(&h[2..], 16).ok(),
        }
    }
    
    pub fn width(&self) -> usize {
        match *self {
            Integer::Binary(b) => b.len() - 2,
            Integer::Decimal(d) => d.len() * 4,
            Integer::Hex(h) => (h.len() - 2) * 4,
        }
    }
}

#[derive(Debug, Logos, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub enum Token<'a> {
    #[regex("[_a-zA-Z]\\w*")]
    Identifier(&'a str),
    
    #[regex("0[bB][01]+",        |lex| Integer::Binary(lex.slice()))]
    #[regex("\\d+",              |lex| Integer::Decimal(lex.slice()))]
    #[regex("0[xX][0-9a-fA-F]+", |lex| Integer::Hex(lex.slice()))]
    Integer(Integer<'a>),
    
    #[token("->", |_| Operator::Arrow)]
    Opterator(Operator),
    
    #[regex("(/\\*([^*]|\\*[^/])+\\*/)|//.*", logos::skip)]
    Comment,
    
    #[error]
    #[regex("[ \t\r\n]*", logos::skip)]
    Error,
}

#[repr(transparent)]
pub struct Lexer<'a, T: Logos<'a>>(logos::Lexer<'a, T>);
pub struct Lexeme<'a, T: logos::Logos<'a>> {
    pub token: T,
    pub slice: &'a <<T as Logos<'a>>::Source as Source>::Slice,
}

impl<'a> Lexer<'a, Token<'a>> {
    pub fn new(source: &'a str) -> Self {
        Self(Token::lexer(source))
    }
}

impl<'a, T: Logos<'a>> Iterator for Lexer<'a, T> {
    type Item = Lexeme<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next();
        next.map(|t| {
            Lexeme {
                token: t,
                slice: self.0.slice(),
            }
        })
    }
}
