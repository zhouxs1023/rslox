use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

use std::collections::HashMap;

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

pub type ParseFn<'src> = fn(&mut Parser<'src>, bool) -> ();

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
            ParseRule::new(None, Some(Parser::binary), Precedence::Equality),
        );
        rule_map.insert(
            TokenType::Equal,
            ParseRule::new(None, None, Precedence::None),
        );
        rule_map.insert(
            TokenType::EqualEqual,
            ParseRule::new(None, Some(Parser::binary), Precedence::Equality),
        );
        rule_map.insert(
            TokenType::Greater,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::GreaterEqual,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::Less,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::LessEqual,
            ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
        );
        rule_map.insert(
            TokenType::Identifier,
            ParseRule::new(Some(Parser::variable), None, Precedence::None),
        );
        rule_map.insert(
            TokenType::String,
            ParseRule::new(Some(Parser::string), None, Precedence::None),
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
        while !self.match_type(TokenType::Eof) {
            self.declaration();
        }
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

    fn check(&self, tk_type: TokenType) -> bool {
        self.current.token_type == tk_type
    }

    fn match_type(&mut self, tk_type: TokenType) -> bool {
        if self.check(tk_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn emit_byte(&mut self, byte: OpCode) {
        self.chunk.write_byte(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2.into());
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::OpReturn);
    }

    fn make_constant(&mut self, val: Value) -> u8 {
        if let Some(constant) = self.chunk.add_constant(val) {
            constant
        } else {
            self.error("Too many constants in one chunk.");
            0
        }
    }

    fn emit_constant(&mut self, val: Value) {
        let con_idx = self.make_constant(val);
        self.emit_bytes(OpCode::OpConstant, con_idx);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if cfg!(debug_assetions) && !self.had_error {
            (&self.chunk).disassemble_chunk("code");
        }
    }

    fn binary(&mut self, can_assign: bool) {
        let op_type = self.previous.token_type;
        let rule = self.get_rule(op_type);
        self.parse_precedence(rule.precedence.next());

        match op_type {
            TokenType::Plus  => self.emit_byte(OpCode::OpAdd.into()),
            TokenType::Minus => self.emit_byte(OpCode::OpSubtract.into()),
            TokenType::Star  => self.emit_byte(OpCode::OpMultiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::OpDivide.into()),
            TokenType::BangEqual => self.emit_bytes(OpCode::OpEqual, OpCode::OpNot.into()),
            TokenType::EqualEqual => self.emit_byte(OpCode::OpEqual.into()),
            TokenType::Greater => self.emit_byte(OpCode::OpGreater.into()),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::OpLess, OpCode::OpNot.into()),
            TokenType::Less => self.emit_byte(OpCode::OpLess.into()),
            TokenType::LessEqual => self.emit_bytes(OpCode::OpGreater, OpCode::OpNot.into()),
            _ => ()   // Unreachable.
        }
    }

    fn literal(&mut self, can_assign: bool) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::OpFalse),
            TokenType::Nil   => self.emit_byte(OpCode::OpNil),
            TokenType::True  => self.emit_byte(OpCode::OpTrue),
            _ => () // Unreachable.
        }
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self, can_assign: bool) {
        let val = self.previous.lexeme.parse().expect("Cannot convert str to f64");
        self.emit_constant(Value::Number(val));
    }

    fn string(&mut self, can_assign: bool) {
        let len = self.previous.lexeme.len() - 1;
        let st = String::from(&self.previous.lexeme[1..len]);
        self.emit_constant(Value::ObjString(st));
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.identifier_constant(name);

        if can_assign && self.match_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::OpSetGlobal, arg);
        } else {
            self.emit_bytes(OpCode::OpGetGlobal,arg);
        }
    }

    fn variable(&mut self, can_assign: bool ) {
        self.named_variable(self.previous, can_assign);
    }

    fn unary(&mut self, can_assign: bool) {
        let op_type = self.previous.token_type;

        // Compile the operand.
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match op_type {
            TokenType::Minus => self.emit_byte(OpCode::OpNegate.into()),
            TokenType::Bang  => self.emit_byte(OpCode::OpNot.into()),
            _ => ()
        }
    }

    fn parse_precedence(&mut self, prec: Precedence ) {

        self.advance();
        let prefix_rule = self.get_rule(self.previous.token_type).prefix;
        let can_assign = prec <= Precedence::Assignment;
        match prefix_rule {
            Some(r) => r(self, can_assign),
            None => {
                self.error("Expect expression.");
                return;
            }
        }

        while prec <= self.get_rule(self.current.token_type).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.previous.token_type).infix;
            match infix_rule {
                Some(r) => r(self, can_assign),
                None => {
                    self.error("Infix rule not found.");
                    return;
                }
            }
        }

        if can_assign && self.match_type(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn identifier_constant(&mut self, name: Token) -> u8 {
        self.make_constant(Value::ObjString(name.lexeme.to_string().clone()))
    }

    fn parse_variable(&mut self, err_msg: &str) -> u8 {
        self.consume(TokenType::Identifier, err_msg);
        self.identifier_constant(self.previous)
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::OpDefineGlobal, global);
    }

    fn get_rule(&self, tk_type: TokenType) -> &ParseRule<'src> {
        return self.rules.get(&tk_type).expect("<TokenType, ParseRule> pair not found.");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_type(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::OpNil);
        }
        self.consume(TokenType::Semicolon,
        "Expect ';' after variable declaration.");

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::OpPop);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon,"Expect ';' after value.");
        self.emit_byte(OpCode::OpPrint);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.token_type != TokenType::Eof {
            if self.previous.token_type == TokenType::Semicolon {
                return;
            }
            match self.current.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => { return; }

                _ => (),
            }
            self.advance();
        }
    }

    fn declaration(&mut self) {
        if self.match_type(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_type(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
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