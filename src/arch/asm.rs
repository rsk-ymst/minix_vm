use super::{opcode::Opcode, operand::Operand};
use crate::VM_MODE;
use std::fmt::Debug;

#[derive(Clone)]
pub struct Assembly {
    pub address: u16,
    pub size: usize,
    pub code: usize,
    pub instruction: Instruction,
}

impl Debug for Assembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (base_space_size, addr_space_size) = if unsafe { VM_MODE } { (13, 0) } else { (14, 1) };

        let base_space = " ".repeat(base_space_size - self.size * 2);
        let addr_space = " ".repeat(addr_space_size);

        if self.code == 0 {
            if self.size == 1 {
                return write!(
                    f,
                    "{:04x}:{addr_space}{:02x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                );
            }

            return write!(
                f,
                "{:04x}:{addr_space}{:04x}{base_space}{:?}",
                self.address, self.code, self.instruction
            );
        }

        // 本当はmatch文を使いたくなかったが、0000等の表示が{:0x}だと0になってしまうので、match文を使う
        match self.size {
            1 => {
                write!(
                    f,
                    "{:04x}:{addr_space}{:02x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                )
            }
            2 => {
                write!(
                    f,
                    "{:04x}:{addr_space}{:04x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                )
            }
            3 => {
                write!(
                    f,
                    "{:04x}:{addr_space}{:06x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                )
            }
            4 => {
                write!(
                    f,
                    "{:04x}:{addr_space}{:08x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                )
            }
            5 => {
                write!(
                    f,
                    "{:04x}:{addr_space}{:10x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                )
            }
            6 => {
                write!(
                    f,
                    "{:04x}:{addr_space}{:12x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                )
            }
            _ => {
                write!(
                    f,
                    "{:04x}:{addr_space}{:0x}{base_space}{:?}",
                    self.address, self.code, self.instruction
                )
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operand1: Option<Operand>,
    pub operand2: Option<Operand>,
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.operand1 {
            Some(operand1) => match &self.operand2 {
                Some(operand2) => write!(f, "{:?} {:?}, {:?}", self.opcode, operand1, operand2),
                None => write!(f, "{:?} {:?}", self.opcode, operand1),
            },
            None => write!(f, "{:?}", self.opcode),
        }
    }
}

// MyStruct に対して Default トレイトを実装
impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            opcode: Opcode::NOP,
            operand1: None,
            operand2: None,
        }
    }
}
