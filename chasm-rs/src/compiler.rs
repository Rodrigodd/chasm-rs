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
    #[token("if")]
    If,
    #[token("endif")]
    EndIf,
    #[token("else")]
    Else,
    #[token("proc")]
    Proc,
    #[token("endproc")]
    EndProc,
    #[token(",")]
    Comma,
    #[regex(r"(\+|-|\*|/|==|<|>|&&)")]
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
    EOF,
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
            Token::If => &Token::If,
            Token::EndIf => &Token::EndIf,
            Token::Else => &Token::Else,
            Token::Proc => &Token::Proc,
            Token::EndProc => &Token::EndProc,
            Token::Comma => &Token::Comma,
            Token::Operator => &Token::Operator,
            Token::Identifier => &Token::Identifier,
            Token::Assignment => &Token::Assignment,
            Token::LeftParen => &Token::LeftParen,
            Token::RightParen => &Token::RightParen,
            Token::Error => &Token::Error,
            Token::EOF => &Token::EOF,
        }
    }
}

use thiserror::Error;

use crate::wasm_macro::wasm;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Unexpected token value, expected {expected:?}, received {received:?}")]
    UnexpectedToken {
        expected: &'static [Token],
        received: (Token, Span),
    },
    #[error("Failed to parse float number ({0})")]
    ParseFloatError(ParseFloatError),
    #[error("Number of arguments mismatch, expected {expected:?}, received {received:?}")]
    ArgumentNumberMismatch { expected: u32, received: u32 },
    #[error("Unexpected number type, expected {expected:?}, received {received:?}")]
    UnexpectedType {
        expected: &'static [Type],
        received: Vec<Type>,
    },
    #[error("Undeclared procedural {name:?}")]
    UndeclaredProc { name: String },
}

type Res<T = ()> = Result<T, Error>;

type LocalIdx = u32;
type FuncIdx = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    I32,
    F32,
}
impl Type {
    fn expect_type(self, other: Self) -> Res {
        if self != other {
            Err(Error::UnexpectedType {
                expected: std::slice::from_ref(match other {
                    Self::I32 => &Self::I32,
                    Self::F32 => &Self::F32,
                }),
                received: vec![self],
            })
        } else {
            Ok(())
        }
    }
}

pub struct Procedure {
    pub idx: FuncIdx,
    pub num_param: u32,
    pub code: Vec<u8>,
}

struct Context {
    code: Vec<u8>,
    symbols: HashMap<String, LocalIdx>,
}
impl Context {
    fn local_index_for_symbol(&mut self, symbol: &str) -> LocalIdx {
        if let Some(idx) = self.symbols.get(symbol) {
            *idx
        } else {
            let len = self.symbols.len() as u32;
            self.symbols.insert(symbol.to_string(), len);
            len
        }
    }
}

/// Compile the source code to webassembly code.
pub struct Parser<'source> {
    source: &'source str,
    lexer: SpannedIter<'source, Token>,
    current: (Token, Span),
    next: (Token, Span),
    procedures: HashMap<String, Procedure>,
}
impl<'s> Parser<'s> {
    pub fn parse(source: &'s str) -> Result<Vec<Procedure>, Error> {
        let lexer = Token::lexer(source).spanned();
        let mut parser = Self {
            source,
            current: (Token::Error, 0..0),
            next: (Token::Error, 0..0),
            lexer,
            procedures: HashMap::new(),
        };
        parser.eat_token();
        parser.eat_token();

        let main_proc = Procedure {
            idx: 1,
            num_param: 0,
            code: Vec::new(),
        };
        parser.procedures.insert("main".to_string(), main_proc);

        let mut ctx = Context {
            code: Vec::new(),
            symbols: HashMap::new(),
        };

        // compile statements
        let ref mut this = parser;
        while this.current.0 != Token::EOF {
            this.statement(&mut ctx)?;
        }
        parser.match_token(Token::EOF)?;
        wasm!(&mut ctx.code, end);

        let locals_index = ctx.code.len();

        // write the vector of locals of the function
        leb128::write::unsigned(&mut ctx.code, 1).unwrap();
        leb128::write::unsigned(&mut ctx.code, ctx.symbols.len() as u64).unwrap();
        wasm!(&mut ctx.code, f32);

        // move locals to the start
        let len = ctx.code.len();
        ctx.code.rotate_right(len - locals_index);

        parser.procedures.get_mut("main").unwrap().code = ctx.code;

        let mut procedures: Vec<_> = Vec::with_capacity(parser.procedures.len());
        for (name, p) in parser.procedures.into_iter() {
            if p.code.is_empty() {
                return Err(Error::UndeclaredProc { name });
            }
            procedures.push(p);
        }
        procedures.sort_by_key(|x| x.idx);
        Ok(procedures)
    }

    fn eat_token(&mut self) {
        self.current = self.next.clone();
        self.next = self.lexer.next().unwrap_or_else(|| {
            let end = self.source.len();
            (Token::EOF, end..end)
        });
    }

    fn match_token(&mut self, token: Token) -> Res {
        if self.current.0 != token {
            Err(Error::UnexpectedToken {
                expected: std::slice::from_ref(token.to_static()),
                received: self.current.clone(),
            })
        } else {
            self.eat_token();
            Ok(())
        }
    }

    fn procedure_from_symbol<'a>(
        &'a mut self,
        symbol: &str,
        num_param: u32,
    ) -> Res<&'a mut Procedure> {
        if let Some(_) = self.procedures.get_mut(symbol) {
            // need to borrow twice, because of borrow checker
            let proc = self.procedures.get_mut(symbol).unwrap();

            if proc.num_param != num_param {
                return Err(Error::ArgumentNumberMismatch {
                    expected: proc.num_param,
                    received: num_param,
                });
            }

            return Ok(proc);
        } else {
            let idx = (self.procedures.len() + 1) as FuncIdx;
            let proc = Procedure {
                idx,
                num_param,
                code: Vec::new(),
            };
            self.procedures.insert(symbol.to_string(), proc);
            let proc = self.procedures.get_mut(symbol).unwrap();
            Ok(proc)
        }
    }

    // parse "<statement>*"
    fn statement(&mut self, ctx: &mut Context) -> Res {
        match self.current.0 {
            Token::Print => self.print_statement(ctx)?,
            Token::Var => self.variable_declaration(ctx)?,
            Token::Identifier => match self.next.0 {
                Token::Assignment => self.variable_assignment(ctx)?,
                Token::LeftParen => self.proc_call(ctx)?,
                _ => {
                    return Err(Error::UnexpectedToken {
                        expected: &[Token::Assignment, Token::LeftParen],
                        received: self.next.clone(),
                    })
                }
            },
            Token::While => self.while_statement(ctx)?,
            Token::If => self.if_statement(ctx)?,
            Token::Proc => self.proc_statement()?,
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: &[Token::Print, Token::Var, Token::Identifier, Token::While],
                    received: self.current.clone(),
                })
            }
        }
        Ok(())
    }

    /// Parse "print <expression>"
    fn print_statement(&mut self, ctx: &mut Context) -> Res {
        self.match_token(Token::Print)?;
        self.expression(ctx)?.expect_type(Type::F32)?;
        wasm!(&mut ctx.code, (call 0x0));
        Ok(())
    }

    /// Parse "var <ident> = <expression>"
    fn variable_declaration(&mut self, ctx: &mut Context) -> Res {
        // the "var" keyword is purely aesthetic
        self.match_token(Token::Var)?;

        self.variable_assignment(ctx)
    }

    /// Parse "<ident> = <expression>"
    fn variable_assignment(&mut self, ctx: &mut Context) -> Res {
        let ident = self.current.clone();
        self.match_token(Token::Identifier)?;
        let idx = ctx.local_index_for_symbol(&self.source[ident.1]);

        self.match_token(Token::Assignment)?;

        self.expression(ctx)?.expect_type(Type::F32)?;
        wasm!(&mut ctx.code, local.set idx);
        Ok(())
    }

    /// Parse "<ident> ( <args>,* )"
    fn proc_call(&mut self, ctx: &mut Context) -> Res {
        let symbol = self.current.clone();
        self.match_token(Token::Identifier)?;
        let ident = &self.source[symbol.1];

        // setpixel calls are hardcoded in the compiler
        if ident == "setpixel" {
            self.match_token(Token::LeftParen)?;

            // yes, setpixel calls cause side effects in variables

            self.expression(ctx)?.expect_type(Type::F32)?;
            let x_idx = ctx.local_index_for_symbol("x");
            wasm!(&mut ctx.code, local.set x_idx);

            self.match_token(Token::Comma)?;

            self.expression(ctx)?.expect_type(Type::F32)?;
            let y_idx = ctx.local_index_for_symbol("y");
            wasm!(&mut ctx.code, local.set y_idx);

            self.match_token(Token::Comma)?;

            self.expression(ctx)?.expect_type(Type::F32)?;
            let color_idx = ctx.local_index_for_symbol("color");
            wasm!(&mut ctx.code, local.set color_idx);

            wasm!(&mut ctx.code,
                // compute ((y*100) + x)
                (local.get y_idx)
                (f32.const 100.0)
                (f32.mul)
                (local.get x_idx)
                (f32.add)
                // convert to integer
                (i32.trunc_f32_s)
                // fetch color
                (local.get color_idx)
                (i32.trunc_f32_s)
                // write to memory
                (i32.store8 0 0)
            );

            self.match_token(Token::RightParen)?;
        } else {
            self.match_token(Token::LeftParen)?;

            let mut n = 0;
            while self.current.0 != Token::RightParen {
                self.expression(ctx)?.expect_type(Type::F32)?;
                n += 1;
                if self.current.0 != Token::RightParen {
                    self.match_token(Token::Comma)?;
                } else {
                    break;
                }
            }
            self.match_token(Token::RightParen)?;

            let idx = self.procedure_from_symbol(ident, n)?.idx;

            wasm!(&mut ctx.code, call idx);
        }
        Ok(())
    }

    /// Parse "while <expression> <statements>* endwhile"
    fn while_statement(&mut self, ctx: &mut Context) -> Res {
        self.match_token(Token::While)?;

        // start a block, and a loop block
        wasm!(&mut ctx.code, (block) (loop));

        // if the expression is false, jump to the end of the block
        self.expression(ctx)?.expect_type(Type::I32)?;
        wasm!(&mut ctx.code, (i32.eqz) (br_if 1));

        while self.current.0 != Token::EndWhile {
            self.statement(ctx)?;
        }

        self.match_token(Token::EndWhile)?;

        // jump to the start of the loop block
        wasm!(&mut ctx.code, (br 0) (end) (end));

        Ok(())
    }

    /// Parse "if <expresion> <expression>* endif" or "if <expression> <expression>* else
    /// <expression>* endif"
    fn if_statement(&mut self, ctx: &mut Context) -> Res {
        self.match_token(Token::If)?;

        // condition
        self.expression(ctx)?.expect_type(Type::I32)?;

        wasm!(&mut ctx.code, if);

        while !(self.current.0 == Token::EndIf || self.current.0 == Token::Else) {
            self.statement(ctx)?;
        }
        if self.current.0 == Token::Else {
            self.match_token(Token::Else)?;
            wasm!(&mut ctx.code, else);
            while self.current.0 != Token::EndIf {
                self.statement(ctx)?;
            }
        }

        self.match_token(Token::EndIf)?;
        wasm!(&mut ctx.code, end);

        Ok(())
    }

    /// Parse "proc <ident> ( <args>,* ) <statement>* endproc"
    fn proc_statement(&mut self) -> Res {
        self.match_token(Token::Proc)?;

        let name = self.current.clone();
        self.match_token(Token::Identifier)?;
        let name = &self.source[name.1];

        let mut args = Vec::new();

        self.match_token(Token::LeftParen)?;
        while self.current.0 != Token::RightParen {
            let arg = self.current.clone();
            self.match_token(Token::Identifier)?;

            let arg = &self.source[arg.1];
            args.push(arg.to_string());

            if self.current.0 != Token::RightParen {
                self.match_token(Token::Comma)?;
            } else {
                break;
            }
        }
        self.match_token(Token::RightParen)?;

        let num_param = args.len() as u32;
        self.procedure_from_symbol(name, num_param)?;

        let mut ctx = Context {
            code: Vec::new(),
            // function arguments are the starting locals index
            symbols: args.into_iter().zip(0..).collect(),
        };

        while self.current.0 != Token::EndProc {
            self.statement(&mut ctx)?;
        }
        self.match_token(Token::EndProc)?;
        wasm!(&mut ctx.code, end);

        let locals_index = ctx.code.len();

        // write the vector of locals of the function
        leb128::write::unsigned(&mut ctx.code, 1).unwrap();
        leb128::write::unsigned(
            &mut ctx.code,
            // don't need to add locals for the argumentes
            (ctx.symbols.len() - num_param as usize) as u64,
        )
        .unwrap();
        wasm!(&mut ctx.code, f32);

        // move locals to the start
        let len = ctx.code.len();
        ctx.code.rotate_right(len - locals_index);

        self.procedure_from_symbol(name, num_param).unwrap().code = ctx.code;

        Ok(())
    }

    /// Parse "<number>" or "<ident>" or "( <expression> <op> <expression> )"
    fn expression(&mut self, ctx: &mut Context) -> Res<Type> {
        match self.current.0 {
            Token::Number => {
                let number = match self.source[self.current.1.clone()].parse::<f32>() {
                    Ok(x) => x,
                    Err(err) => return Err(Error::ParseFloatError(err)),
                };
                self.match_token(Token::Number)?;
                wasm!(&mut ctx.code, (f32.const number));
                Ok(Type::F32)
            }
            Token::Identifier => {
                let ident = self.current.clone();
                self.match_token(Token::Identifier)?;

                let symbol = &self.source[ident.1];
                let idx = ctx.local_index_for_symbol(symbol);

                wasm!(&mut ctx.code, local.get idx);
                Ok(Type::F32)
            }
            Token::LeftParen => {
                self.match_token(Token::LeftParen)?;

                // left
                let type_a = self.expression(ctx)?;

                let op = self.current.clone();
                self.match_token(Token::Operator)?;
                let op = &self.source[op.1];

                // right
                let type_b = self.expression(ctx)?;

                // op
                match op {
                    "+" | "-" | "*" | "/" | "<" | ">" | "==" => {
                        if type_a != Type::F32 || type_b != Type::F32 {
                            return Err(Error::UnexpectedType {
                                expected: &[Type::F32, Type::F32],
                                received: vec![type_a, type_b],
                            });
                        }
                    }
                    "&&" => {
                        if type_a != Type::I32 || type_b != Type::I32 {
                            return Err(Error::UnexpectedType {
                                expected: &[Type::I32, Type::I32],
                                received: vec![type_a, type_b],
                            });
                        }
                    }
                    _ => unreachable!("I already match the token operator"),
                }
                match op {
                    "+" => wasm!(&mut ctx.code, f32.add),
                    "-" => wasm!(&mut ctx.code, f32.sub),
                    "*" => wasm!(&mut ctx.code, f32.mul),
                    "/" => wasm!(&mut ctx.code, f32.div),
                    "==" => wasm!(&mut ctx.code, f32.eq),
                    "<" => wasm!(&mut ctx.code, f32.lt),
                    ">" => wasm!(&mut ctx.code, f32.gt),
                    "&&" => wasm!(&mut ctx.code, i32.and),
                    _ => unreachable!("I already match the token operator"),
                }

                self.match_token(Token::RightParen)?;

                match op {
                    "+" | "-" | "*" | "/" => Ok(Type::F32),
                    "==" | "<" | ">" | "&&" => Ok(Type::I32),
                    _ => unreachable!("I already match the token operator"),
                }
            }
            _ => {
                return Err(Error::UnexpectedToken {
                    expected: &[Token::Number, Token::LeftParen],
                    received: self.current.clone(),
                })
            }
        }
    }
}
