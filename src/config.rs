use crate::lexer::{Lexer, Lexeme, Token};
use crate::log::{Logger, LoggedResult, Origin};
use crate::parser::*;
use std::collections::HashMap;

fn codegen_brackets<'a>(lexer: &mut Lexer<'a, Token<'a>>, name: &str, registers: usize, immediates: &[(usize, usize)]) -> LoggedResult<Codegen> {
    let mut logger = Logger::new(None);
    
    macro_rules! match_codegen_data_after {
        ($after:expr) => {
            match lexer.next() {
                Some(Lexeme { token: Token::Integer(int), slice }) => {
                    if int > 0xF {
                        logger.log_warning(format!("{} is larger than 4 bits and will be truncated", slice));
                    }
                    CodegenData::Byte((int & 0xF) as u8)
                },
                Some(Lexeme { token: Token::Immediate(im), .. }) => {
                    if im >= immediates.len() {
                        logger.log_error(format!("'{}' uses immediate {} which is not given in the instruction pattern", name, im));
                        return logger.into_none();
                    }
                    let immediate = immediates[im];
                    if immediate.1 != 4 {
                        logger.log_error("width of immediate in bracket group must be 4 (for now)".to_owned());
                        return logger.into_none();
                    }
                    CodegenData::Immediate(immediate.0, immediate.1)
                },
                Some(Lexeme { token: Token::Register(r), .. }) => {
                    if r >= registers {
                        logger.log_error(format!("'{}' uses register {} which is not given in the instruction pattern", name, r));
                    }
                    CodegenData::Register(r)
                },
                Some(Lexeme { slice, .. }) => {
                    logger.log_error(format!("expected a literal or register after '{}', but got '{}'", $after, slice));
                    return logger.into_none();
                }
                None => {
                    logger.log_error(format!("expected a literal or register after '{}'", $after));
                    return logger.into_none();
                }
            };
        }
    }
    macro_rules! match_symbol {
        ($token:pat, $symbol:expr) => {
            match lexer.next() {
                Some(Lexeme { token: $token, .. }) => {},
                Some(Lexeme { slice, .. }) => {
                    logger.log_error(format!("expected '{}' in bracket group, but got '{}'", $symbol, slice));
                    return logger.into_none();
                },
                None => {
                    logger.log_error(format!("expected '{}' in bracket group", $symbol));
                    return logger.into_none();
                }
            }
        }
    }
    
    let upper = match_codegen_data_after!('[');
    match_symbol!(Token::Or, '|');
    let lower = match_codegen_data_after!('|');
    match_symbol!(Token::CloseBracket, ']');
    
    logger.into_result(|| Codegen::UpperLower(upper, lower))
}

pub fn create_assembler_from_config(config: &str) -> LoggedResult<Assembler> {
    let origin = "[unknown]";
    let mut map = HashMap::new();
    let mut logger = Logger::new(None);
    
    for (line, source) in config.lines().enumerate() {
        logger.origin = Some(Origin { file: origin.to_owned(), line });
        let mut lexer = Lexer::new(source);
        
        // Only supports instructions right now
        let name = match lexer.next() {
            Some(Lexeme { token: Token::Ident(name), .. }) => name.to_lowercase(),
            None => continue,
            _ => {
                logger.log_error("only instruction patterns are supported in the assembler config at the moment".to_owned());
                continue;
            }
        };
        
        let instruction = &mut map.entry(name.clone()).or_insert(Instruction { syntaxes: Vec::new(), states: vec![TransitionTable::default()] });
        
        let states = &mut instruction.states;
        let mut current_state = 0;
        let mut registers = 0;
        let mut immediates = Vec::new();
        let mut accept_state = false;
        
        // Generate DFA
        while let Some(token) = lexer.next() {
            match token.token {
                Token::Immediate(im) => {
                    if im > immediates.len() {
                        logger.log_warning(format!("immediates are parsed in the order they appear regardless of number; {} will correspond to i{} in codegen", token.slice, immediates.len()));
                    }
                    match lexer.next() {
                        Some(Lexeme { token: Token::Colon, .. }) => {
                            let width = match lexer.next() {
                                Some(Lexeme { token: Token::Integer(width), .. }) => width,
                                Some(Lexeme { slice, .. }) => {
                                    logger.log_error(format!("expected width of immediate, but got: '{}'", slice));
                                    break;
                                },
                                None => {
                                    logger.log_error("expected width of immediate".to_owned());
                                    break;
                                }
                            };
                            immediates.push((im, width));
                            if let Transition::NextState(next) = states[current_state].immediate {
                                current_state = next;
                            } else {
                                states[current_state].immediate = Transition::NextState(states.len());
                                current_state = states.len();
                                states.push(TransitionTable::default());
                            }
                        },
                        Some(Lexeme { slice, .. }) => logger.log_error(format!("expected width of immediate, but got '{}'", slice)),
                        None => logger.log_error("expected width of immediate".to_owned()),
                    }
                }
                
                Token::Register(r) => {
                    if r != registers {
                        logger.log_warning(format!("registers are parsed in the order they appear regardless of number; {} will correspond to r{} in codegen", token.slice, registers));
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
                        logger.log_error(format!("conflicting patterns for instruction '{}'", name));
                    } else {
                        let mut codegen = Vec::new();
                        while let Some(token) = lexer.next() {
                            match token.token {
                                Token::Integer(int) => {
                                    if int > 255 {
                                        logger.log_warning(format!("{} is larger than 8 bits and will be truncated", token.slice));
                                    }
                                    codegen.push(Codegen::byte(int as u8));
                                },
                                
                                Token::Immediate(im) => {
                                    if im >= immediates.len() {
                                        logger.log_error(format!("'{}' uses immediate {} which is not given in the instruction pattern", name, im));
                                        break;
                                    }
                                    let immediate = immediates[im];
                                    if immediate.1 % 8 != 0 {
                                        logger.log_error("immediate width must be byte aligned (for now)".to_owned());
                                    } else {
                                        codegen.push(Codegen::immediate(immediate.0, immediate.1));
                                    }
                                },
                                
                                Token::Register(r) => {
                                    if r >= registers {
                                        logger.log_error(format!("'{}' uses register {} which is not given in the instruction pattern", name, r));
                                    }
                                    codegen.push(Codegen::register(r));
                                }
                                
                                Token::OpenBracket => {
                                    codegen_brackets(&mut lexer, &name, registers, &immediates).if_ok(&mut logger, |bracket| codegen.push(bracket));
                                },
                                
                                _ => {
                                    logger.log_error(format!("codegen only supports literal values, registers, and bracket groups, but got '{}'", token.slice));
                                    break;
                                },
                            }
                        }
                        states[current_state].accept_codegen = Some(codegen);
                    }
                    accept_state = true;
                    break;
                },
                
                _ => logger.log_error(format!("unexpected token in instrution pattern: '{}'", token.slice))
            }
        }
        if !accept_state {
            logger.log_error("expected '->' following an instruction pattern".to_owned());
        } else {
            let syntax = source.split_once("->").unwrap().0;
            let lex_fold = Lexer::new(syntax).fold(String::with_capacity(16), |a, Lexeme{slice,..}| {
                if a.is_empty() || a.ends_with(':') || slice == "," || slice == ":"{
                    a + slice
                } else {
                    a + " " + slice
                }
            });
            instruction.syntaxes.push(lex_fold.to_lowercase());
        }
    }
    
    // If an error was reported
    logger.into_result(|| Assembler { instructions: map })
}
