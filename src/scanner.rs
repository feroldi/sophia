use std::iter::Peekable;
use std::str::Chars;

use crate::compiler_context::CompilerContext;

pub(crate) struct Scanner<'ctx> {
    ctx: &'ctx CompilerContext,
    char_stream: Peekable<Chars<'ctx>>,
    current_peek_pos: BytePos,
}

impl Scanner<'_> {
    const EOF_CHAR: char = '\0';

    pub(crate) fn new<'ctx>(ctx: &'ctx CompilerContext) -> Scanner {
        Scanner {
            ctx,
            char_stream: ctx.get_source_code().chars().peekable(),
            current_peek_pos: BytePos(0),
        }
    }

    pub(crate) fn scan_all_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while let Some(token) = self.scan_next_token() {
            tokens.push(token)
        }

        tokens
    }

    fn scan_next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let span_start = self.current_peek_pos;

        let token_kind = match self.bump() {
            Scanner::EOF_CHAR => return None,
            ';' => TokenKind::Semi,
            ':' => {
                if self.peek() == ':' {
                    self.bump();

                    TokenKind::ColonColon
                } else if self.peek() == '=' {
                    self.bump();

                    TokenKind::ColonEqual
                } else {
                    TokenKind::Colon
                }
            }
            '(' => TokenKind::Open(Delim::Paren),
            ')' => TokenKind::Closed(Delim::Paren),
            '{' => TokenKind::Open(Delim::Curly),
            '}' => TokenKind::Closed(Delim::Curly),
            '-' if self.peek() == '>' => {
                self.bump();

                TokenKind::DashGreater
            }
            '.' if self.peek() == '.' => {
                self.bump();

                if self.peek() == '=' {
                    self.bump();

                    TokenKind::PeriodPeriodEqual
                } else {
                    TokenKind::PeriodPeriod
                }
            }
            '0'..='9' => self.scan_integer_constant(),
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(span_start),
            ch => todo!("char not recognized: '{}'", ch),
        };

        let token_span = Span {
            start: span_start,
            end: self.current_peek_pos,
        };

        Some(Token {
            kind: token_kind,
            span: token_span,
        })
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_ascii_whitespace() {
            self.bump();
        }
    }

    fn scan_integer_constant(&mut self) -> TokenKind {
        while self.peek().is_ascii_digit() {
            self.bump();
        }

        TokenKind::IntegerConstant
    }

    fn scan_identifier(&mut self, ident_span_start: BytePos) -> TokenKind {
        while matches!(self.peek(), 'a'..='z' | 'A'..='Z' | '_' | '0'..='9') {
            self.bump();
        }

        let ident_text = &self.ctx.get_source_code()[ident_span_start.0..self.current_peek_pos.0];

        match ident_text {
            "i32" => TokenKind::Keyword(Keyword::I32),
            "if" => TokenKind::Keyword(Keyword::If),
            "else" => TokenKind::Keyword(Keyword::Else),
            "for" => TokenKind::Keyword(Keyword::For),
            "break" => TokenKind::Keyword(Keyword::Break),
            "continue" => TokenKind::Keyword(Keyword::Continue),
            _ => TokenKind::Identifier,
        }
    }

    fn peek(&mut self) -> char {
        self.char_stream
            .peek()
            .cloned()
            .unwrap_or(Scanner::EOF_CHAR)
    }

    fn bump(&mut self) -> char {
        let peeked = self.peek();

        if peeked != Scanner::EOF_CHAR {
            self.current_peek_pos = BytePos(self.current_peek_pos.0 + peeked.len_utf8());
            self.char_stream.next();
        }

        peeked
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum TokenKind {
    UnitConstant,
    IntegerConstant,
    Identifier,
    Comma,
    Excla,
    Star,
    Slash,
    Plus,
    Dash,
    Less,
    Greater,
    LessLess,
    GreaterGreater,
    LessEqual,
    GreaterEqual,
    Colon,
    ColonColon,
    ColonEqual,
    Semi,
    DashGreater,
    PeriodPeriod,
    PeriodPeriodEqual,
    Keyword(Keyword),
    Open(Delim),
    Closed(Delim),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Keyword {
    I32,
    If,
    Else,
    For,
    Break,
    Continue,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Delim {
    Paren,
    Curly,
}

#[derive(Clone, Copy)]
pub(crate) struct Span {
    pub(crate) start: BytePos,
    pub(crate) end: BytePos,
}

#[derive(Clone, Copy)]
pub(crate) struct BytePos(pub(crate) usize);
