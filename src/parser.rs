use crate::log::{Logger, LoggedResult, Origin};
use crate::lexer::{Lexer, Lexeme, Token};
use std::collections::HashMap;

#[derive(Debug)]
pub enum CodegenData {
    Byte(u8),
    Immediate(usize, usize),
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
    pub fn byte(b: u8) -> Self { Codegen::Data(CodegenData::Byte(b)) }
    pub fn immediate(imm: usize, b: usize) -> Self { Codegen::Data(CodegenData::Immediate(imm, b)) }
    pub fn register(r: usize) -> Self { Codegen::Data(CodegenData::Register(r)) }
}

#[derive(Debug)]
pub enum Transition {
    Reject,
    NextState(usize),
}
impl Default for Transition {
    fn default() -> Self { Self::Reject }
}

#[derive(Debug, Default)]
pub struct TransitionTable {
    pub register: Transition,
    pub immediate: Transition,
    pub comma: Transition,
    // TODO: more options for syntax
    
    // If some, the state can accept the input and proceed to codegen
    pub accept_codegen: Option<Vec<Codegen>>,
}

#[derive(Debug)]
pub struct Instruction {
    pub syntaxes: Vec<String>,
    pub states: Vec<TransitionTable>,
}

#[derive(Debug)]
pub struct Assembler {
    pub instructions: HashMap<String, Instruction>
}

impl Assembler {
    pub fn assemble(&self, source: &str) -> LoggedResult<Vec<u8>> {
        let origin = "[unknown]";
        let mut captured_registers = Vec::new();
        let mut captured_immediates = Vec::new();
        let mut output = Vec::new();
        let mut logger = Logger::new(None);
        
        'outer: for (line, source) in source.lines().enumerate() {
            let mut lexer = Lexer::new(source);
            logger.origin = Some(Origin { file: origin.to_owned(), line });
            captured_registers.clear();
            
            if let Some(lexeme) = lexer.next() {
                match lexeme.token {
                    // Instruction
                    Token::Ident(ident) => {
                        let name = ident.to_lowercase();
                        let instruction = if let Some(ins) = self.instructions.get(&name) {
                            ins
                        } else {
                            logger.log_error(format!("unknown instruction: '{}'", lexeme.slice));
                            continue;
                        };
                        
                        let mut current_state = 0;
                        
                        let codegen = loop {
                            match lexer.next() {
                                Some(Lexeme{ token: Token::Integer(int), slice }) => {
                                    if let Transition::NextState(next) = instruction.states[current_state].immediate {
                                        captured_immediates.push(int);
                                        current_state = next;
                                    } else {
                                        logger.log_error(format!("unexpected immediate: '{}'", slice));
                                        logger.log_error(format!("syntaxes available for {}: {:?}", name, instruction.syntaxes));
                                        continue 'outer;
                                    }
                                },
                                
                                Some(Lexeme{ token: Token::Register(r), slice }) => {
                                    if let Transition::NextState(next) = instruction.states[current_state].register {
                                        if r > 15 {
                                            logger.log_error(format!("register out of bounds: '{}'", slice));
                                            continue 'outer;
                                        }
                                        captured_registers.push(r as u8);
                                        current_state = next;
                                    } else {
                                        logger.log_error(format!("unexpected register: '{}'", slice));
                                        logger.log_error(format!("syntaxes available for {}: {:?}", name, instruction.syntaxes));
                                        continue 'outer;
                                    }
                                },
                                
                                Some(Lexeme{ token: Token::Comma, .. }) => {
                                    if let Transition::NextState(next) = instruction.states[current_state].comma {
                                        current_state = next;
                                    } else {
                                        logger.log_error("unexpected comma".to_owned());
                                        logger.log_error(format!("syntaxes available for {}: {:?}", name, instruction.syntaxes));
                                        continue 'outer;
                                    }
                                },
                                
                                None => {
                                    if let Some(ref codegen) = instruction.states[current_state].accept_codegen {
                                        break codegen;
                                    } else {
                                        logger.log_error("syntax error".to_owned());
                                        logger.log_error(format!("syntaxes available for {}: {:?}", name, instruction.syntaxes));
                                        continue 'outer;
                                    }
                                },
                                
                                Some(Lexeme{ slice, .. }) => {
                                    logger.log_error(format!("unexpected token: '{}'", slice));
                                    logger.log_error(format!("syntaxes available for {}: {:?}", name, instruction.syntaxes));
                                    continue 'outer;
                                },
                            }
                        };
                        
                        let decode = |codegen: &CodegenData| match *codegen {
                            CodegenData::Byte(b) => b,
                            CodegenData::Register(r) => captured_registers[r],
                            CodegenData::Immediate(imm, _) => captured_immediates[imm] as u8,
                        };
                        
                        for data in codegen {
                            match data {
                                Codegen::Data(data) => {
                                    match *data {
                                        CodegenData::Immediate(imm, b) => {
                                            let imm = captured_immediates[imm];
                                            if imm.leading_zeros() < (64-b+1) as u32 {
                                                logger.log_warning(format!("'{}' will be truncated to {} bits", imm, b));
                                            }
                                            let bytes = b / 8;
                                            output.extend(&imm.to_le_bytes()[..bytes]);
                                        },
                                        _ => output.push(decode(data)),
                                    }
                                },
                                Codegen::UpperLower(upper, lower) => {
                                    let upper = decode(upper);
                                    let lower = decode(lower);
                                    output.push((upper & 0xF) << 4 | (lower & 0xF));
                                }
                            }
                        }
                    },
                    
                    _ => logger.log_error(format!("unexpected token: '{}'", lexeme.slice))
                }
            }
        }
        
        logger.into_result(||output)
    }
}
