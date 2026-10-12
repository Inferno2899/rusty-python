use super::ast::{BinaryOperator, Expression, Statement};
use super::token::{Keyword, Token};

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();

        while self.current().is_some() {
            if self.current() == Some(&Token::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            Some(Token::Keyword(Keyword::If)) => self.parse_if(),

            Some(Token::Keyword(Keyword::While)) => self.parse_while(),

            Some(Token::Keyword(Keyword::Def)) => self.parse_function(),

            Some(Token::Keyword(Keyword::Class)) => self.parse_class(),

            Some(Token::Keyword(Keyword::Return)) => self.parse_simple_statement(),

            Some(Token::Keyword(Keyword::Pass)) => self.parse_simple_statement(),

            _ => self.parse_simple_statement(),
        }
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect(Token::Keyword(Keyword::If))?;

        let colon_index = self.find_token(Token::Colon);
        let condition = self.parse_expression(&self.tokens[self.current..colon_index])?;

        self.current = colon_index + 1;
        self.expect(Token::Newline)?;

        let body = self.parse_block()?;

        Ok(Statement::If { condition, body })
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        self.expect(Token::Keyword(Keyword::While))?;

        let colon_index = self.find_token(Token::Colon);
        let condition = self.parse_expression(&self.tokens[self.current..colon_index])?;

        self.current = colon_index + 1;
        self.expect(Token::Newline)?;

        let body = self.parse_block()?;

        Ok(Statement::While { condition, body })
    }

    fn parse_function(&mut self) -> Result<Statement, String> {
        self.expect(Token::Keyword(Keyword::Def))?;

        let name = self.expect_identifier()?;

        self.expect(Token::LeftParenthesis)?;
        let mut parameters = Vec::new();
        while self.current() != Some(&Token::RightParenthesis) {
            if self.current() != Some(&Token::Comma)
                && let Some(Token::Identifier(s)) = self.current()
            {
                parameters.push(s.clone());
            }
            self.advance();
        }
        self.expect(Token::RightParenthesis)?;
        self.expect(Token::Colon)?;
        self.expect(Token::Newline)?;

        let body = self.parse_block()?;

        Ok(Statement::Function {
            name,
            parameters,
            body,
        })
    }

    fn parse_class(&mut self) -> Result<Statement, String> {
        self.expect(Token::Keyword(Keyword::Class))?;

        let name = self.expect_identifier()?;

        self.expect(Token::Colon)?;

        self.expect(Token::Newline)?;

        let body = self.parse_block()?;

        Ok(Statement::Class { name, body })
    }

    fn parse_simple_statement(&mut self) -> Result<Statement, String> {
        let end = self.find_token(Token::Newline);

        let result = {
            let statement_tokens = &self.tokens[self.current..end];

            if let Some(index) = self.find_operator(statement_tokens, Token::Equal) {
                let name = self.parse_expression(&statement_tokens[..index])?;
                let value = self.parse_expression(&statement_tokens[index + 1..])?;
                Statement::Assignment { name, value }
            } else {
                match statement_tokens {
                    [Token::Keyword(Keyword::Return)] => Statement::Return(Expression::None),
                    [Token::Keyword(Keyword::Return), rest @ ..] => {
                        Statement::Return(self.parse_expression(rest)?)
                    }
                    [Token::Keyword(Keyword::Pass)] => Statement::Pass,
                    _ => Statement::Expression(self.parse_expression(statement_tokens)?),
                }
            }
        };

        self.current = end;

        if self.current() == Some(&Token::Newline) {
            self.advance();
        }

        Ok(result)
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        self.expect(Token::Indent)?;

        let mut body = Vec::new();

        while self.current() != Some(&Token::Dedent) {
            if self.current() == Some(&Token::Newline) {
                self.advance();
                continue;
            }
            body.push(self.parse_statement()?);
        }

        self.expect(Token::Dedent)?;

        Ok(body)
    }

    fn parse_expression(&self, tokens: &[Token]) -> Result<Expression, String> {
        if tokens.len() == 1 {
            return self.parse_primary(&tokens[0]);
        }
        if let Some(index) = self.find_operator(tokens, Token::Greater) {
            return self.parse_binary(tokens, index, BinaryOperator::Greater);
        }
        if let Some(index) = self.find_operator(tokens, Token::Less) {
            return self.parse_binary(tokens, index, BinaryOperator::Less);
        }
        if let Some(index) = self.find_operator(tokens, Token::EqualEqual) {
            return self.parse_binary(tokens, index, BinaryOperator::Equal);
        }
        if let Some(index) = self.find_operator(tokens, Token::Plus) {
            return self.parse_binary(tokens, index, BinaryOperator::Plus);
        }
        if let Some(index) = self.find_operator(tokens, Token::Minus) {
            return self.parse_binary(tokens, index, BinaryOperator::Minus);
        }
        if let Some(index) = self.find_operator(tokens, Token::Star) {
            return self.parse_binary(tokens, index, BinaryOperator::Multiply);
        }
        if let Some(index) = self.find_operator(tokens, Token::Slash) {
            return self.parse_binary(tokens, index, BinaryOperator::Divide);
        }
        if tokens.first() == Some(&Token::LeftParenthesis)
            && tokens.last() == Some(&Token::RightParenthesis)
        {
            return self.parse_expression(&tokens[1..tokens.len() - 1]);
        }
        if tokens.first() == Some(&Token::LeftBracket)
            && tokens.last() == Some(&Token::RightBracket)
        {
            return self.parse_list(&tokens[1..tokens.len() - 1]);
        }
        if tokens.contains(&Token::LeftParenthesis) {
            return self.parse_function_call(tokens);
        }
        if tokens.contains(&Token::LeftBracket) {
            return self.parse_indexing(tokens);
        }
        if tokens.contains(&Token::Dot) {
            return self.parse_attribute_access(tokens);
        }
        Err(format!("couldn't parse expression: {:?}", tokens))
    }

    fn parse_primary(&self, token: &Token) -> Result<Expression, String> {
        match token {
            Token::Integer(value) => Ok(Expression::Integer(*value)),
            Token::Identifier(s) => Ok(Expression::Variable(s.clone())),
            Token::Str(value) => Ok(Expression::Str(value.clone())),
            Token::Keyword(Keyword::None) => Ok(Expression::None),
            _ => Err(format!("unknown primary token: {:?}", token)),
        }
    }

    fn parse_binary(
        &self,
        tokens: &[Token],
        index: usize,
        operator: BinaryOperator,
    ) -> Result<Expression, String> {
        let left = Box::new(self.parse_expression(&tokens[..index])?);
        let right = Box::new(self.parse_expression(&tokens[index + 1..])?);
        Ok(Expression::Binary {
            left,
            operator,
            right,
        })
    }

    fn parse_function_call(&self, tokens: &[Token]) -> Result<Expression, String> {
        let index = tokens
            .iter()
            .position(|t| *t == Token::LeftParenthesis)
            .unwrap();
        let function = Box::new(self.parse_expression(&tokens[..index])?);
        let mut arguments = Vec::new();
        let argument_tokens = &tokens[index + 1..tokens.len() - 1];

        if !argument_tokens.is_empty() {
            let commas_indices = self.find_commas(argument_tokens);

            let mut start = 0;
            for &end in &commas_indices {
                arguments.push(self.parse_expression(&argument_tokens[start..end])?);
                start = end + 1;
            }
            arguments.push(self.parse_expression(&argument_tokens[start..])?);
        }
        Ok(Expression::Call {
            function,
            arguments,
        })
    }

    fn parse_attribute_access(&self, tokens: &[Token]) -> Result<Expression, String> {
        let dot_index = tokens.iter().position(|t| *t == Token::Dot).unwrap();

        let object = Box::new(self.parse_expression(&tokens[..dot_index])?);
        let name = if let Some(Token::Identifier(s)) = tokens[dot_index + 1..].first() {
            s
        } else {
            return Err("name of attribute must be a string".to_string());
        };

        Ok(Expression::Attribute {
            object,
            name: name.clone(),
        })
    }

    fn parse_list(&self, tokens: &[Token]) -> Result<Expression, String> {
        let mut result = Vec::new();

        if !tokens.is_empty() {
            for expression in tokens.split(|token| *token == Token::Comma) {
                result.push(self.parse_expression(expression)?);
            }
        }

        Ok(Expression::List(result))
    }

    fn parse_indexing(&self, tokens: &[Token]) -> Result<Expression, String> {
        let opening_bracket_index = tokens
            .iter()
            .position(|t| *t == Token::LeftBracket)
            .unwrap();
        let object = Box::new(self.parse_expression(&tokens[..opening_bracket_index])?);
        let index =
            Box::new(self.parse_expression(&tokens[opening_bracket_index + 1..tokens.len() - 1])?);
        Ok(Expression::Index { object, index })
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) {
        self.current += 1
    }

    fn find_token(&self, token: Token) -> usize {
        self.tokens[self.current..]
            .iter()
            .position(|tkn| *tkn == token)
            .map(|offset| self.current + offset)
            .unwrap_or(self.tokens.len())
    }

    fn find_operator(&self, tokens: &[Token], operator: Token) -> Option<usize> {
        let mut depth = 0;

        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::LeftParenthesis => {
                    depth += 1;
                }

                Token::RightParenthesis => {
                    depth -= 1;
                }

                _ if depth == 0 && *token == operator => {
                    return Some(i);
                }

                _ => {}
            }
        }

        None
    }

    fn find_commas(&self, tokens: &[Token]) -> Vec<usize> {
        let mut depth = 0;
        let mut ans = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::LeftParenthesis => {
                    depth += 1;
                }

                Token::RightParenthesis => {
                    depth -= 1;
                }

                _ if depth == 0 && *token == Token::Comma => {
                    ans.push(i);
                }

                _ => {}
            }
        }

        ans
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.current() {
            Some(token) if *token == expected => {
                self.advance();
                Ok(())
            }

            actual => Err(format!("expected {expected:?}, got {actual:?}")),
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        let name = match self.current() {
            Some(Token::Identifier(name)) => name.clone(),

            actual => {
                return Err(format!("expected identifier, got {actual:?}"));
            }
        };

        self.advance();

        Ok(name)
    }
}
