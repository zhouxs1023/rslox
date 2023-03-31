use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

use std::{collections::HashMap, convert::TryFrom};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,  // < > <= >=
    Term,        // + -
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary
}

impl Precedence {
    fn next(&self) -> Self {
        use Precedence::*;
        match *self {
            None       =>  Assignment,
            Assignment =>  Or,
            Or         =>  And,
            And        =>  Equality,
            Equality   =>  Comparison,
            Comparison =>  Term,
            Term       =>  Factor,
            Factor     =>  Unary,
            Unary      =>  Call,
            Call       =>  Primary,
            Primary    =>  None,
        }
    }
}

pub type ParseFn<'src> = fn(&mut Parser<'src>) -> ();

struct ParseRule<'src> {
    prefix: Option<ParseFn<'src>>,
    infix:  Option<ParseFn<'src>>,
    precedence: Precedence
}

impl<'src> ParseRule<'src> {
    fn new(
        prefix: Option<ParseFn<'src>>,
        infix: Option<ParseFn<'src>>,
        precedence: Precedence,
    ) -> ParseRule<'src> {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

pub struct Parser<'src> {
    scanner: Scanner<'src>,
    current: Token<'src>,
    previous: Token<'src>,
    rules: HashMap<TokenType, ParseRule<'src>>,
    pub chunk: Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str) -> Parser<'src> {
        let mut rule_map = HashMap::new();
        rule_map.insert(
            TokenType::LeftParen,
            ParseRule::new(Some(Parser::grouping), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::RightParen,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::LeftBrace,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::RightBrace,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Comma,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(TokenType::Dot, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Minus,
            ParseRule::new(Some(Parser::unary), Some(Parser::binary), Precedence::Term),
        );
        rule_map.insert(
            TokenType::Plus,
            ParseRule::new(None, Some(Parser::binary), Precedence::Term),
        );
        rule_map.insert(
            TokenType::Semicolon,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Slash,
            ParseRule::new(None, Some(Parser::binary), Precedence::Factor),
        );
        rule_map.insert(
            TokenType::Star,
            ParseRule::new(None, Some(Parser::binary), Precedence::Factor),
        );
        rule_map.insert(
            TokenType::Bang,
            ParseRule::new(Some(Parser::unary), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::BangEqual,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Equal,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::EqualEqual,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Greater,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::GreaterEqual,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Less,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::LessEqual,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Identifier,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::String,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Number,
            ParseRule::new(Some(Parser::number), None, Precedence::None),
        );
        rule_map.insert(TokenType::And, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Class,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Else,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::False,
            ParseRule::new(Some(Parser::literal), None, Precedence::None),
        );
        rule_map.insert(TokenType::For, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(TokenType::Fun, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(TokenType::If, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(TokenType::Nil, ParseRule::new(Some(Parser::literal), None, Precedence::None));
        rule_map.insert(TokenType::Or, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::Print,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Return,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Super,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::This,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::True,
            ParseRule::new(Some(Parser::literal), None, Precedence::None),
        );
        rule_map.insert(TokenType::Var, ParseRule::new(None, None, Precedence::None));
        rule_map.insert(
            TokenType::While,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::Error,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(TokenType::Eof, ParseRule::new(None, None, Precedence::None));

        let dummy_token = Token::new(TokenType::Eof, 0, "");
        let dummy_token2 = Token::new(TokenType::Eof, 0, "");
        Parser {
            chunk: Chunk::new(),
            current: dummy_token,
            previous: dummy_token2,
            scanner: Scanner::new(src),
            rules: rule_map,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn compile(&mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();
        !self.had_error
    }

    fn advance(&mut self) {
        self.previous = self.current;

        loop {
            self.current = self.scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            };

            self.error_at_current(self.current.lexeme);
        }
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }
        self.error_at_current(msg);
    }

    fn emit_byte(&mut self, byte: OpCode) {

        self.chunk.write_chunk(byte, self.previous.line);
    }

    #[allow(dead_code)]
    fn emit_bytes(&mut self, byte1: OpCode, byte2: OpCode) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_retrun(&mut self) {

        self.emit_byte(OpCode::OpReturn);
    }

    fn make_constant(&mut self, val: Value) -> u8 {
        let idx = self.chunk.add_constant(val);
        match u8::try_from(idx) {
            Ok(idx) => idx,
            Err(_) => {
                self.error("Too many constants in one chunk.");
                0
            }
        }
    }

    fn emit_constant(&mut self, val: f64) {
        let con_idx = self.make_constant(Value::Number(val));
        self.emit_byte(OpCode::OpConstant(con_idx));
    }

    fn end_compiler(&mut self) {
        self.emit_retrun();
        if cfg!(debug_assetions) && !self.had_error {
            (&self.chunk).disassemble_chunk("code");
        }
    }

    fn binary(&mut self) {
        let op_type = self.previous.token_type;
        let rule = self.get_rule(op_type);
        self.parse_precedence(rule.precedence.next());

        match op_type {
            TokenType::Plus  => self.emit_byte(OpCode::OpAdd),
            TokenType::Minus => self.emit_byte(OpCode::OpSubtract),
            TokenType::Star  => self.emit_byte(OpCode::OpMultiply),
            TokenType::Slash => self.emit_byte(OpCode::OpDivide),
            _ => ()   // Unreachable.
        }
    }

    fn literal(&mut self) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::OpFalse),
            TokenType::Nil   => self.emit_byte(OpCode::OpNil),
            TokenType::True  => self.emit_byte(OpCode::OpTrue),
            _ => () // Unreachable.
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self) {
        let val = self.previous.lexeme.parse().expect("Cannot convert str to f64");
        self.emit_constant(val);
    }

    fn unary(&mut self) {
        let op_type = self.previous.token_type;

        // Compile the operand.
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match op_type {
            TokenType::Minus => self.emit_byte(OpCode::OpNegate),
            TokenType::Bang  => self.emit_byte(OpCode::OpNot),
            _ => ()
        }
    }

    fn parse_precedence(&mut self, prec: Precedence ) {
        self.advance();
        let prefix_rule = self.get_rule(self.previous.token_type).prefix;
        match prefix_rule {
            Some(r) => r(self),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        while prec <= self.get_rule(self.current.token_type).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.previous.token_type).infix;
            match infix_rule {
                Some(r) => r(self),
                None => {
                    self.error("Infix rule not found.");
                    return;
                }
            }
        }
    }

    fn get_rule(&self, tk_type: TokenType) -> &ParseRule<'src> {
        return self.rules.get(&tk_type).expect("<TokenType, ParseRule> pair not found.");
    }

    fn expression(&mut self) {

        self.parse_precedence(Precedence::Assignment);
    }

    fn error_at(&mut self, token: Token, message: &str) {

        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        if token.token_type == TokenType::Eof {
            eprint!(" at end");
        } else if token.token_type == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at {}'", token.lexeme);
        }

        eprint!(": {}\n", message);
        self.had_error = true;
    }

    fn error(&mut self, message: &str) {

        self.error_at(self.previous, message);
    }

    fn error_at_current(&mut self, message: &str) {

        self.error_at(self.current, message);
    }
}