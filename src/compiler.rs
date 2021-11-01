use logos::{Logos, Span, SpannedIter};
use std::collections::HashMap;
use std::io::Write;
use std::num::ParseFloatError;

#[derive(Logos, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Token {
    // this regex for number don't make much sense, but it is this way in my reference:
    // https://github.com/ColinEberhardt/chasm/blob/master/src/tokenizer.ts#L41
    #[regex(r"-?[.0-9]+([eE]-?[0-9][0-9])?")]
    Number,
    #[token("print")]
    Print,
    #[token("var")]
    Var,
    #[token("while")]
    While,
    #[token("endwhile")]
    EndWhile,
    #[regex(r"(\+|-|\*|/|==|<|>|&&|,)")]
    Operator,
    #[regex(r"[a-zA-Z]+")]
    Identifier,
    #[token("=")]
    Assignment,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[error]
    #[regex(r"\s+", logos::skip)]
    Error,
}
impl Token {
    /// Return a reference to a static value with the same variant that self
    fn to_static(self) -> &'static Self {
        match self {
            Token::Number => &Token::Number,
            Token::Print => &Token::Print,
            Token::Var => &Token::Var,
            Token::While => &Token::While,
            Token::EndWhile => &Token::EndWhile,
            Token::Operator => &Token::Operator,
            Token::Identifier => &Token::Identifier,
            Token::Assignment => &Token::Assignment,
            Token::LeftParen => &Token::LeftParen,
            Token::RightParen => &Token::RightParen,
            Token::Error => &Token::Error,
        }
    }
}

use thiserror::Error;

use crate::wasm_macro::wasm;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unexpected token value, expected {expected:?}, received {received:?}")]
    UnexpectedToken {
        expected: &'static [Token],
        received: Token,
    },
    #[error("Failed to parse float number ({0})")]
    ParseFloatError(ParseFloatError),
}

type Res = Result<(), Error>;

type LocalIdx = u32;

/// Compile the source code to webassembly code.
pub struct Parser<'source, 'bin> {
    source: &'source str,
    lexer: SpannedIter<'source, Token>,
    w: &'bin mut Vec<u8>,
    current: (Token, Span),
    symbols: HashMap<String, LocalIdx>
}
impl<'s, 'b> Parser<'s, 'b> {
    pub fn parse(source: &'s str, w: &'b mut Vec<u8>) -> Result<(), Error> {
        let lexer = Token::lexer(source).spanned();
        let mut parser = Self {
            source,
            current: (Token::Error, 0..0),
            lexer,
            w,
            symbols: HashMap::new(),
        };
        parser.eat_token();

        // record the index of the vector
        let start_index = parser.w.len();

        parser.statements()?;
        
        let locals_index = parser.w.len();

        // write the vector of locals of the function
        leb128::write::unsigned(parser.w, 1).unwrap();
        leb128::write::unsigned(parser.w, parser.symbols.len() as u64).unwrap();
        wasm!(parser.w, f32);

        // move locals to the start
        let len = parser.w.len();
        parser.w[start_index..].rotate_right(len - locals_index);

        Ok(())
    }

    fn eat_token(&mut self) {
        self.current = self.lexer.next().unwrap_or((Token::Error, 0..0));
    }

    fn match_token(&mut self, token: Token) -> Res {
        if self.current.0 != token {
            Err(Error::UnexpectedToken {
                expected: std::slice::from_ref(token.to_static()),
                received: self.current.0.clone(),
            })
        } else {
            self.eat_token();
            Ok(())
        }
    }

    fn local_index_for_symbol(&mut self, symbol: &str) -> LocalIdx {
        if let Some(idx) = self.symbols.get(symbol) {
            *idx
        } else {
            let len = self.symbols.len() as u32;
            self.symbols.insert(symbol.to_string(), len);
            len
        }
    }

    fn statements(&mut self) -> Res {
        while self.current.0 != Token::Error {
            self.statement()?;
        }
        Ok(())
    }

    // parse "<statement>*"
    fn statement(&mut self) -> Res {
        match self.current.0 {
            Token::Print => self.print_statement()?,
            Token::Var => self.variable_declaration()?,
            Token::Identifier => self.variable_assignment()?,
            Token::While => self.while_statement()?,
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: &[Token::Print, Token::Var, Token::Identifier, Token::While],
                    received: self.current.0.clone(),
                })
            }
        }
        Ok(())
    }

    /// Parse "print <expression>"
    fn print_statement(&mut self) -> Res {
        self.match_token(Token::Print)?;
        self.expression()?;
        wasm!(self.w, (call 0x0));
        Ok(())
    }

    /// Parse "var <ident> = <expression>"
    fn variable_declaration(&mut self) -> Res {
        // the "var" keyword is purely aesthetic
        self.match_token(Token::Var)?;

        self.variable_assignment()
    }

    /// Parse "<ident> = <expression>"
    fn variable_assignment(&mut self) -> Res {
        let ident = self.current.clone();
        self.match_token(Token::Identifier)?;
        let idx = self.local_index_for_symbol(&self.source[ident.1]);

        self.match_token(Token::Assignment)?;

        self.expression()?;
        wasm!(self.w, local.set idx);
        Ok(())
    }

    /// Parse "while <expression> <statements>* endwhile"
    fn while_statement(&mut self) -> Res {
        self.match_token(Token::While)?;

        wasm!(self.w, block);
        wasm!(self.w, loop);
        
        self.expression()?;

        wasm!(self.w, i32.eqz);
        wasm!(self.w, br_if 1);

        while self.current.0 != Token::EndWhile {
            self.statement()?;
        }

        self.match_token(Token::EndWhile)?;

        wasm!(self.w, br 0);
        wasm!(self.w, (end) (end));

        Ok(())
    }

    /// Parse "<number>" or "<ident>" or "( <expression> <op> <expression> )"
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
            Token::Identifier => {
                let ident = self.current.clone();
                self.match_token(Token::Identifier)?;

                let symbol = &self.source[ident.1];
                let idx = self.local_index_for_symbol(symbol);

                wasm!(self.w, local.get idx);
            }
            Token::LeftParen => {
                self.match_token(Token::LeftParen)?;

                // left
                self.expression()?;

                let op = self.current.clone();
                self.match_token(Token::Operator)?;

                // right
                self.expression()?;

                // op
                match &self.source[op.1] {
                    "+" => wasm!(self.w, f32.add),
                    "-" => wasm!(self.w, f32.sub),
                    "*" => wasm!(self.w, f32.mul),
                    "/" => wasm!(self.w, f32.div),
                    "==" => wasm!(self.w, f32.eq),
                    "<" => wasm!(self.w, f32.lt),
                    ">" => wasm!(self.w, f32.gt),
                    "&&" => wasm!(self.w, i32.and),
                    _ => unreachable!("I already match the token operator"),
                }

                self.match_token(Token::RightParen)?;
            }
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: &[Token::Number, Token::LeftParen],
                    received: self.current.0.clone(),
                })
            }
        }
        Ok(())
    }
}
