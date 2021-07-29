use std::collections::HashMap;
use crate::log::Log;
use crate::lexer::{Lexer, Lexeme, Token};

#[derive(Debug)]
pub enum CodegenData {
    Byte(u8),
    Register(usize),
}

#[derive(Debug)]
pub enum Codegen {
    // Writes the data directly to the output buffer
    Data(CodegenData),
    
    // Writes the data to the upper and lower half bytes of the output buffer
    UpperLower(CodegenData, CodegenData),
}
impl Codegen {
    fn byte(b: u8) -> Self { Codegen::Data(CodegenData::Byte(b)) }
    fn register(r: usize) -> Self { Codegen::Data(CodegenData::Register(r)) }
}

#[derive(Debug)]
enum Transition {
    Reject,
    NextState(usize),
}
impl Default for Transition {
    fn default() -> Self { Self::Reject }
}

#[derive(Debug, Default)]
pub struct TransitionTable {
    register: Transition,
    comma: Transition,
    // TODO: more options for syntax
    
    // If some, the state can accept the input and proceed to codegen
    pub accept_codegen: Option<Vec<Codegen>>,
}

#[derive(Debug)]
pub struct Instruction {
    pub states: Vec<TransitionTable>,
}

#[derive(Debug)]
pub struct Assembler {
    pub instructions: HashMap<String, Instruction>
}

pub fn parse(source: &str) -> (Option<Assembler>, Vec<Log>) {
    let origin = "[unknown]";
    let mut map = HashMap::new();
    let mut logs = Vec::new();
    
    for (line, source) in source.lines().enumerate() {
        macro_rules! log {
            ($kind:ident, $msg:expr) => {{
                logs.push(Log::$kind {
                    origin: origin.to_owned(),
                    line,
                    message: format!($msg),
                });
            }};
            ($kind:ident, $msg:expr, $($params:expr),+) => {{
                logs.push(Log::$kind {
                    origin: origin.to_owned(),
                    line,
                    message: format!($msg, $($params),+)
                });
            }};
        }
        
        let mut lexer = Lexer::new(source);
        
        // Only supports instructions right now
        let name = match lexer.next() {
            Some(Lexeme { token: Token::Ident(name), .. }) => name.to_lowercase(),
            None => continue,
            _ => {
                log!(Error, "only instruction patterns are supported in the assembler config at the moment");
                continue;
            }
        };
        
        let states = &mut map.entry(name.clone()).or_insert(Instruction { states: vec![TransitionTable::default()] }).states;
        let mut current_state = 0;
        let mut registers = 0;
        let mut accept_state = false;
        
        // Generate DFA
        while let Some(token) = lexer.next() {
            match token.token {
                Token::Register(r) => {
                    if r != registers {
                        log!(Warning, "registers are parsed in the order they appear regardless of number; {} will correspond to r{} in codegen", token.slice, registers);
                    }
                    if let Transition::NextState(next) = states[current_state].register {
                        current_state = next;
                    } else {
                        states[current_state].register = Transition::NextState(states.len());
                        current_state = states.len();
                        states.push(TransitionTable::default());
                    }
                    registers += 1;
                }

                Token::Comma => {
                    if let Transition::NextState(next) = states[current_state].comma {
                        current_state = next;
                    } else {
                        states[current_state].comma = Transition::NextState(states.len());
                        current_state = states.len();
                        states.push(TransitionTable::default());
                    }
                },
                
                Token::Arrow => {
                    if states[current_state].accept_codegen.is_some() {
                        log!(Error, "conflicting patterns for instruction '{}'", name);
                    } else {
                        let mut codegen = Vec::new();
                        while let Some(token) = lexer.next() {
                            match token.token {
                                Token::Integer(int) => {
                                    if int > 255 {
                                        log!(Warning, "{} is larger than a byte and will be truncated", token.slice);
                                    }
                                    codegen.push(Codegen::byte(int as u8));
                                },
                                
                                Token::Register(r) => {
                                    if r >= registers {
                                        log!(Error, "'{}' uses register {} which is not given in the instruction pattern", name, r);
                                    }
                                    codegen.push(Codegen::register(r));
                                }
                                
                                Token::OpenBracket => {
                                    macro_rules! match_codegen_data_after {
                                        ($after:expr) => {
                                            match lexer.next() {
                                                Some(Lexeme { token: Token::Integer(int), slice }) => {
                                                    if int > 0xF {
                                                        log!(Warning, "{} is larger than 4 bits and will be truncated", slice);
                                                    }
                                                    CodegenData::Byte((int & 0xF) as u8)
                                                },
                                                Some(Lexeme { token: Token::Register(r), .. }) => {
                                                    if r >= registers {
                                                        log!(Error, "'{}' uses register {} which is not given in the instruction pattern", name, r);
                                                    }
                                                    CodegenData::Register(r)
                                                },
                                                Some(Lexeme { slice, .. }) => {
                                                    log!(Error, "expected a literal or register after '{}', but got '{}'", $after, slice);
                                                    break;
                                                }
                                                None => {
                                                    log!(Error, "expected a literal or register after '{}'", $after);
                                                    break;
                                                }
                                            };
                                        }
                                    }
                                    macro_rules! match_symbol {
                                        ($token:pat, $symbol:expr) => {
                                            match lexer.next() {
                                                Some(Lexeme { token: $token, .. }) => {},
                                                Some(Lexeme { slice, .. }) => {
                                                    log!(Error, "expected '{}' in bracket group, but got '{}'", $symbol, slice);
                                                    break;
                                                },
                                                None => {
                                                    log!(Error, "expected '{}' in bracket group", $symbol);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    
                                    let upper = match_codegen_data_after!('[');
                                    match_symbol!(Token::Or, '|');
                                    let lower = match_codegen_data_after!('|');
                                    match_symbol!(Token::CloseBracket, ']');
                                    codegen.push(Codegen::UpperLower(upper, lower));
                                },
                                
                                _ => {
                                    log!(Error, "codegen only supports literal values, registers, and bracket groups, but got '{}'", token.slice);
                                    break;
                                },
                            }
                        }
                        states[current_state].accept_codegen = Some(codegen);
                    }
                    accept_state = true;
                    break;
                },
                
                _ => log!(Error, "unexpected token in instrution pattern: '{}'", token.slice)
            }
        }
        if !accept_state {
            log!(Error, "expected '->' following an instruction pattern");
        }
    }
    
    // If an error was reported
    if logs.iter().any(Log::is_error) {
        (None, logs)
    } else {
        (Some(Assembler { instructions: map }), logs)
    }
}
