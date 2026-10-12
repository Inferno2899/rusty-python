#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Integer(i64),
    Str(String),

    Identifier(String),
    Keyword(Keyword),

    Plus,
    Minus,
    Star,
    Slash,

    Greater,
    Less,
    EqualEqual,

    Equal,

    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,

    Comma,
    Colon,
    Dot,

    Newline,

    Indent,
    Dedent,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Keyword {
    If,
    While,
    Def,
    Class,
    Return,
    None,
    Pass,
}
