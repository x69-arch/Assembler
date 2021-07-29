use logos::Logos;

#[inline]
fn trim<'a>(lex: &mut logos::Lexer<'a, Token<'a>>, begin: usize, end: usize) -> &'a str {
    let s = lex.slice();
    &s[begin..s.len()-end]
}

#[inline]
fn parse_int<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> Result<u64, std::num::ParseIntError> {
    let slice = lex.slice();
    if slice.starts_with("0x") || slice.starts_with("0X") {
        u64::from_str_radix(&slice[2..], 16)
    } else if slice.starts_with("0b") || slice.starts_with("0B") {
        u64::from_str_radix(&slice[2..], 2)
    } else {
        slice.parse()
    }
}

#[derive(Debug, Logos, PartialEq, Clone)]
pub enum Token<'a> {
    #[regex("[_a-zA-Z]\\w*")]
    Ident(&'a str),
    
    #[regex("[rR]\\d+", |lex| trim(lex, 1, 0).parse())]
    Register(usize),
    
    #[regex("[_a-zA-Z]\\w*:")]
    Label(&'a str),
    
    #[regex("\\.[_a-zA-Z0-9]\\w*")]
    Directive(&'a str),
    
    #[regex("\"[^\"]*\"", |lex| trim(lex, 1, 1))]
    String(&'a str),
    
    #[regex("(0[xX][\\da-fA-F]+)|(0[bB][01]+)|\\d+", parse_int)]
    Integer(u64),
    
    #[token("->")]
    Arrow,
    
    #[token(",")]
    Comma,
    
    #[token("|")]
    Or,
    
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    
    #[regex("(/\\*([^*]|\\*[^/])+\\*/)|//.*", logos::skip)]
    Comment,
    
    #[error]
    #[regex("[ \t\r]*", logos::skip)]
    Error,
}

#[repr(transparent)]
pub struct Lexer<'a, T: Logos<'a, Source = str>>(logos::Lexer<'a, T>);
pub struct Lexeme<'a, T> {
    pub token: T,
    pub slice: &'a str,
}

impl<'a, T: Logos<'a, Source = str>> Lexer<'a, T> {
    pub fn new(source: &'a str) -> Self
        where <T as Logos<'a>>::Extras: Default,
    {
        Self(T::lexer(source))
    }
}

impl<'a, T: Logos<'a, Source = str>> Iterator for Lexer<'a, T> {
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
