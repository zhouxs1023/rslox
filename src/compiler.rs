use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

use std::collections::HashMap;
use crate::chunk::OpCode::OpPop;

static USIZE_COUNT: usize = u8::MAX as usize + 1;

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

pub struct Local<'src> {
    name: Token<'src>,
    depth: i32,
}

impl<'src> Local<'src> {
    pub fn new(name: Token<'src>, depth: i32 ) -> Local<'src> {
        Local {name, depth}
    }
}

pub struct Compiler<'src> {
    locals: Vec<Local<'src>>,
    scope_depth: i32,
}

impl<'src> Compiler<'src> {
    pub fn new() -> Compiler<'src> {
        Compiler {
            locals: Vec::with_capacity(USIZE_COUNT),
            scope_depth: 0,
        }
    }
}

pub struct Parser<'src> {
    scanner: Scanner<'src>,
    compiler: Compiler<'src>,
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
        rule_map.insert(TokenType::And, ParseRule::new(None, Some(Parser::and), Precedence::And));
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
        rule_map.insert(TokenType::Or, ParseRule::new(None, Some(Parser::or), Precedence::Or));
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
            compiler: Compiler::new(),
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

    fn emit_u8(&mut self, byte: u8) {
        self.chunk.write_u8(byte, self.previous.line);
    }

    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2.into());
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::OpLoop);

        let offset = self.chunk.code.len() - loop_start + 2;
        if offset > u16::MAX.into() { self.error("Loop body too large.")}

        self.emit_u8(((offset >> 8) & 0xff) as u8);
        self.emit_u8((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_byte(instruction);
        self.emit_u8(0xff);
        self.emit_u8(0xff);
        return self.chunk.code.len() - 2;
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

    fn patch_jump(&mut self, offset: usize) {
        // -2 to adjust for the bytecode for the jump offset itself.
        let jump = self.chunk.code.len() - offset - 2;

        if jump > USIZE_COUNT {
            self.error("Too much code to jump over.");
        }

        self.chunk.code[offset] = (((jump >> 8) & 0xff) as u8).into();
        self.chunk.code[offset + 1] = ((jump & 0xff) as u8).into();
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if cfg!(debug_assetions) && !self.had_error {
            (&self.chunk).disassemble_chunk("code");
        }
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        while self.compiler.locals.len() > 0 &&
            self.compiler.locals[self.compiler.locals.len() - 1].depth > self.compiler.scope_depth
        {
            self.emit_byte(OpCode::OpPop);
            self.compiler.locals.pop();
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

    fn or(&mut self, can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        let end_jump = self.emit_jump(OpCode::OpJump);

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::OpPop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn string(&mut self, can_assign: bool) {
        let len = self.previous.lexeme.len() - 1;
        let st = String::from(&self.previous.lexeme[1..len]);
        self.emit_constant(Value::ObjString(st));
    }

    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let (arg, get_op, set_op) = if let Some(local_arg) = self.resolve_local(*name) {
            (local_arg, OpCode::OpGetLocal, OpCode::OpSetLocal)
        } else {
            (
                self.identifier_constant(*name),
                OpCode::OpGetGlobal,
                OpCode::OpSetGlobal,
            )
        };

        if can_assign && self.match_type(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op, arg as u8);
        } else {
            self.emit_bytes(get_op,arg as u8);
        }
    }

    fn variable(&mut self, can_assign: bool ) {
        self.named_variable(&self.previous.clone(), can_assign);
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

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        a.lexeme == b.lexeme
    }

    fn resolve_local(&mut self, name: Token) -> Option<u8> {
        for (i, local) in self.compiler.locals.iter().rev().enumerate() {
            if self.identifiers_equal(&name, &local.name) {
                if local.depth == -1 {
                    self.error("Cannot read local variable in its own initializer.");
                }
                return Some(i as u8);
            }
        }
        return None;
    }

    fn  add_local(&mut self, name: Token<'src>) {
        if self.compiler.locals.len() == USIZE_COUNT {
            self.error("Too many local variables in function.");
            return;
        }

        let local = Local::new(name, -1);
        self.compiler.locals.push(local);
    }

    fn parse_variable(&mut self, err_msg: &str) -> u8 {
        self.consume(TokenType::Identifier, err_msg);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(self.previous)
    }

    fn mark_initialized(&mut self) {
        let last = self.compiler.locals.last_mut().unwrap();
        last.depth = self.compiler.scope_depth
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(OpCode::OpDefineGlobal, global);
    }

    fn and(&mut self, can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::OpJumpIfFalse);

        self.emit_byte(OpCode::OpPop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        let name = self.previous;

        for local in self.compiler.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.compiler.scope_depth {
                break;
            }

            if self.identifiers_equal(&name, &local.name) {
                self.error("Already a variable with this name in this scope.");
                break;
            }
        }
        self.add_local(name);
    }

    fn get_rule(&self, tk_type: TokenType) -> &ParseRule<'src> {
        return self.rules.get(&tk_type).expect("<TokenType, ParseRule> pair not found.");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
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

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.emit_byte(OpCode::OpPop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::OpJump);

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::OpPop);
        if self.match_type(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon,"Expect ';' after value.");
        self.emit_byte(OpCode::OpPrint);
    }

    fn while_statement(&mut self) {
        let loop_start = self.chunk.code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::OpJumpIfFalse);
        self.emit_byte(OpCode::OpPop);
        self.statement();

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::OpPop);
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
        } else if self.match_type(TokenType::If) {
            self.if_statement();
        } else if self.match_type(TokenType::While) {
            self.while_statement();
        } else if self.match_type(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
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