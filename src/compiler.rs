use logos::{Logos, Span, SpannedIter};
use std::io::Write;
use std::num::ParseFloatError;

#[derive(Logos, PartialEq, Eq, Debug, Clone)]
pub enum Token {
    // this regex for number don't make much sense, but it is this way in my reference:
    // https://github.com/ColinEberhardt/chasm/blob/master/src/tokenizer.ts#L41
    #[regex("-?[.0-9]+([eE]-?[0-9][0-9])?")]
    Number,
    #[token("print")]
    Print,
    #[error]
    #[regex("\\s+", logos::skip)]
    Error,
}

use thiserror::Error;

use crate::wasm_macro::wasm;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unexpected token value, expected {expected:?}, received {received:?}")]
    ParseError { expected: Token, received: Token },
    #[error("Failed to parse float number ({0})")]
    ParseFloatError(ParseFloatError),
}

type Res = Result<(), Error>;

/// Compile the source code to webassembly code.
pub struct Parser<'source, 'bin> {
    source: &'source str,
    lexer: SpannedIter<'source, Token>,
    w: &'bin mut Vec<u8>,
    current: (Token, Span),
}
impl<'s, 'b> Parser<'s, 'b> {
    pub fn parse(source: &'s str, w: &'b mut Vec<u8>) -> Result<(), Error> {
        let lexer = Token::lexer(source).spanned();
        let mut parser = Self {
            source,
            current: (Token::Error, 0..0),
            lexer,
            w 
        };
        parser.eat_token();
        parser.statements()?;
        Ok(())
    }

    fn eat_token(&mut self) {
        self.current = self.lexer.next().unwrap_or((Token::Error, 0..0));
    }

    fn match_token(&mut self, token: Token) -> Res {
        if self.current.0 != token {
            Err(Error::ParseError { expected: token, received: self.current.0.clone() })
        } else {
            self.eat_token();
            Ok(())
        }
    }

    fn statements(&mut self) -> Res {
        while self.current.0 != Token::Error {
            self.statement()?;
        }
        Ok(())
    }

    fn statement(&mut self) -> Res {
        match self.current.0  {
            Token::Print => self.print_statement()?,
            _ => return Err(Error::ParseError { expected: Token::Print, received: self.current.0.clone() })
        }
        Ok(())
    }

    fn print_statement(&mut self) -> Res {
        self.match_token(Token::Print)?;
        self.expression()?;
        wasm!(self.w, (call 0x0));
        Ok(())
    }

    fn expression(&mut self) -> Res {
        match self.current.0 {
            Token::Number => {
                let number = match self.source[self.current.1.clone()].parse::<f32>() {
                    Ok(x) => x,
                    Err(err) => return Err(Error::ParseFloatError(err)),
                };
                self.match_token(Token::Number)?;
                wasm!(self.w, (f32.const number));
            }
            _ => return Err(Error::ParseError { expected: Token::Number, received: self.current.0.clone() }),
        }
        Ok(())
    }
}
