use crate::frontend::{Keyword, Token};

pub struct Lexer {}

impl Lexer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tokenize(&self, source: String) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let mut chars = source.char_indices().peekable();
        let mut indentations = vec![0];
        let mut at_line_start = true;
        while let Some((index, ch)) = chars.next() {
            let start = index;
            if at_line_start {
                // Empty line: indentation does not change.
                if ch == '\n' {
                    tokens.push(Token::Newline);
                    at_line_start = true;
                    continue;
                }

                let mut indentation = 0;

                if ch == ' ' {
                    indentation = 1;

                    while let Some(&(_, ' ')) = chars.peek() {
                        chars.next();
                        indentation += 1;
                    }

                    // A line containing only spaces is also blank.
                    if let Some(&(_, '\n')) = chars.peek() {
                        continue;
                    }
                }

                if indentation % 4 != 0 {
                    return Err("indentation must be divisible by 4".to_string());
                }

                let current = *indentations.last().unwrap();

                if indentation > current {
                    tokens.push(Token::Indent);
                    indentations.push(indentation);
                }

                while indentation < *indentations.last().unwrap() {
                    tokens.push(Token::Dedent);
                    indentations.pop();
                }

                at_line_start = false;

                if ch == ' ' {
                    continue;
                }
            }

            match ch {
                ' ' => continue,
                '=' => {
                    if let Some(&(_, '=')) = chars.peek() {
                        tokens.push(Token::EqualEqual);
                        chars.next();
                    } else {
                        tokens.push(Token::Equal);
                    }
                }
                '+' => tokens.push(Token::Plus),
                '-' => tokens.push(Token::Minus),
                '*' => tokens.push(Token::Star),
                '/' => tokens.push(Token::Slash),
                '\n' => {
                    tokens.push(Token::Newline);
                    at_line_start = true;
                }
                '(' => tokens.push(Token::LeftParenthesis),
                ')' => tokens.push(Token::RightParenthesis),
                '[' => tokens.push(Token::LeftBracket),
                ']' => tokens.push(Token::RightBracket),
                ',' => tokens.push(Token::Comma),
                '>' => tokens.push(Token::Greater),
                '<' => tokens.push(Token::Less),
                ':' => tokens.push(Token::Colon),
                '.' => tokens.push(Token::Dot),
                '"' => {
                    let mut finish = index;
                    while let Some(&(_, next)) = chars.peek() {
                        if next.is_ascii_alphabetic() {
                            chars.next();
                            finish += 1;
                        } else {
                            break;
                        }
                    }
                    finish += 1;
                    chars.next();
                    let text = &source[start + 1..finish];
                    tokens.push(Token::Str(text.to_string()));
                }
                ch if ch.is_ascii_alphabetic() || ch == '_' => {
                    let mut finish = index;
                    while let Some(&(_, next)) = chars.peek() {
                        if next.is_ascii_alphanumeric() || next == '_' {
                            chars.next();
                            finish += 1;
                        } else {
                            break;
                        }
                    }
                    let text = &source[start..finish + 1];
                    match text {
                        "if" => tokens.push(Token::Keyword(Keyword::If)),
                        "while" => tokens.push(Token::Keyword(Keyword::While)),
                        "def" => tokens.push(Token::Keyword(Keyword::Def)),
                        "return" => tokens.push(Token::Keyword(Keyword::Return)),
                        "class" => tokens.push(Token::Keyword(Keyword::Class)),
                        "None" => tokens.push(Token::Keyword(Keyword::None)),
                        "pass" => tokens.push(Token::Keyword(Keyword::Pass)),
                        _ => tokens.push(Token::Identifier(text.to_string())),
                    }
                }
                ch if ch.is_ascii_digit() => {
                    let mut finish = index;
                    while let Some(&(_, next)) = chars.peek() {
                        if next.is_ascii_digit() {
                            chars.next();
                            finish += 1;
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Integer(source[start..finish + 1].parse().unwrap()));
                }
                '#' => {
                    while let Some(&(_, next)) = chars.peek() {
                        if next == '\n' {
                            break;
                        }
                        chars.next();
                    }
                }
                _ => return Err(format!("unexpected token: {}", ch)),
            }
        }

        if tokens.last() != Some(&Token::Newline) {
            tokens.push(Token::Newline);
        }

        while indentations.len() > 1 {
            indentations.pop();
            tokens.push(Token::Dedent);
        }

        Ok(tokens)
    }
}
