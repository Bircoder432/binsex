use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Whitespace,

    #[regex(r"[0-9]+", |lex| lex.slice().to_string())]
    Number(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len() - 1].to_string()
    })]
    Text(String),
    #[regex(r"#.*?\n", logos::skip)]
    Comment,
    #[regex(r"@[a-zA-Z]+", |lex| lex.slice().to_string())]
    MetaOperator(String),

    #[regex(r"[a-zA-Z]+", |lex| lex.slice().to_string())]
    Operator(String),
    #[token(";")]
    Semicolon,
}
