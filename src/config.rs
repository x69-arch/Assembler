use crate::lexer::{Lexer, Lexeme, Token};
use crate::{log, log::{LoggedResult, Origin}};
use crate::parser::*;
use std::collections::HashMap;

fn codegen_brackets<'a>(lexer: &mut Lexer<'a, Token<'a>>, name: &str, registers: usize) -> LoggedResult<Codegen> {
    let mut result = LoggedResult::new();
    
    macro_rules! match_codegen_data_after {
        ($after:expr) => {
            match lexer.next() {
                Some(Lexeme { token: Token::Integer(int), slice }) => {
                    if int > 0xF {
                        result.log_warning(format!("{} is larger than 4 bits and will be truncated", slice));
                    }
                    CodegenData::Byte((int & 0xF) as u8)
                },
                Some(Lexeme { token: Token::Register(r), .. }) => {
                    if r >= registers {
                        result.log_error(format!("'{}' uses register {} which is not given in the instruction pattern", name, r));
                    }
                    CodegenData::Register(r)
                },
                Some(Lexeme { slice, .. }) => {
                    result.log_error(format!("expected a literal or register after '{}', but got '{}'", $after, slice));
                    return result;
                }
                None => {
                    result.log_error(format!("expected a literal or register after '{}'", $after));
                    return result;
                }
            };
        }
    }
    macro_rules! match_symbol {
        ($token:pat, $symbol:expr) => {
            match lexer.next() {
                Some(Lexeme { token: $token, .. }) => {},
                Some(Lexeme { slice, .. }) => {
                    result.log_error(format!("expected '{}' in bracket group, but got '{}'", $symbol, slice));
                    return result;
                },
                None => {
                    result.log_error(format!("expected '{}' in bracket group", $symbol));
                    return result;
                }
            }
        }
    }
    
    let upper = match_codegen_data_after!('[');
    match_symbol!(Token::Or, '|');
    let lower = match_codegen_data_after!('|');
    match_symbol!(Token::CloseBracket, ']');
    
    result.return_value(|| Codegen::UpperLower(upper, lower))
}

pub fn create_assembler_from_config(config: &str) -> LoggedResult<Assembler> {
    let origin = "[unknown]";
    let mut map = HashMap::new();
    let mut result = LoggedResult::new();
    
    for (line, source) in config.lines().enumerate() {
        let mut lexer = Lexer::new(source);
        
        // Only supports instructions right now
        let name = match lexer.next() {
            Some(Lexeme { token: Token::Ident(name), .. }) => name.to_lowercase(),
            None => continue,
            _ => {
                result.push_log(log!(Error, origin, line, "only instruction patterns are supported in the assembler config at the moment"));
                continue;
            }
        };
        
        let instruction = &mut map.entry(name.clone()).or_insert(Instruction { syntaxes: Vec::new(), states: vec![TransitionTable::default()] });
        
        let states = &mut instruction.states;
        let mut current_state = 0;
        let mut registers = 0;
        let mut accept_state = false;
        
        // Generate DFA
        while let Some(token) = lexer.next() {
            match token.token {
                Token::Register(r) => {
                    if r != registers {
                        result.push_log(log!(Warning, origin, line, "registers are parsed in the order they appear regardless of number; {} will correspond to r{} in codegen", token.slice, registers));
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
                        result.push_log(log!(Error, origin, line, "conflicting patterns for instruction '{}'", name));
                    } else {
                        let mut codegen = Vec::new();
                        while let Some(token) = lexer.next() {
                            match token.token {
                                Token::Integer(int) => {
                                    if int > 255 {
                                        result.push_log(log!(Warning, origin, line, "{} is larger than a byte and will be truncated", token.slice));
                                    }
                                    codegen.push(Codegen::byte(int as u8));
                                },
                                
                                Token::Register(r) => {
                                    if r >= registers {
                                        result.push_log(log!(Error, origin, line, "'{}' uses register {} which is not given in the instruction pattern", name, r));
                                    }
                                    codegen.push(Codegen::register(r));
                                }
                                
                                Token::OpenBracket => {
                                    codegen_brackets(&mut lexer, &name, registers).map_origin(Origin {
                                        file: origin.to_owned(),
                                        line,
                                    }).take_log_and(&mut result, |bracket| codegen.push(bracket));
                                },
                                
                                _ => {
                                    result.push_log(log!(Error, origin, line, "codegen only supports literal values, registers, and bracket groups, but got '{}'", token.slice));
                                    break;
                                },
                            }
                        }
                        states[current_state].accept_codegen = Some(codegen);
                    }
                    accept_state = true;
                    break;
                },
                
                _ => result.push_log(log!(Error, origin, line, "unexpected token in instrution pattern: '{}'", token.slice))
            }
        }
        if !accept_state {
            result.push_log(log!(Error, origin, line, "expected '->' following an instruction pattern"));
        } else {
            let syntax = source.split_once("->").unwrap().0;
            let lex_fold = Lexer::new(syntax).fold(String::with_capacity(16), |a, Lexeme{slice,..}| {
                if slice == "," || a.is_empty() {
                    a + slice
                } else {
                    a + " " + slice
                }
            });
            instruction.syntaxes.push(lex_fold.to_lowercase());
        }
    }
    
    // If an error was reported
    result.return_value(|| Assembler { instructions: map })
}
