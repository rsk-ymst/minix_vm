use super::constant;
use std::fmt::Debug;

#[repr(C)]
#[derive(PartialEq, Clone)]
pub enum Register {
    Reg16(Reg16),
    Reg8(Reg8),
    None,
}

impl Register {
    pub fn gen(num: u8, w: u8) -> Self {
        if w == 1 {
            return Self::Reg16(Reg16::from(num));
        } else {
            return Self::Reg8(Reg8::from(num));
        }
    }
}

impl Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::Reg16(reg) => write!(f, "{:?}", reg),
            Register::Reg8(reg) => write!(f, "{:?}", reg),
            Register::None => write!(f, "None"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum Reg16 {
    AX = constant::reg::AX,
    CX = constant::reg::CX,
    DX = constant::reg::DX,
    BX = constant::reg::BX,
    SP = constant::reg::SP,
    BP = constant::reg::BP,
    SI = constant::reg::SI,
    DI = constant::reg::DI,
    None,
}

impl Debug for Reg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg16::AX => write!(f, "ax"),
            Reg16::CX => write!(f, "cx"),
            Reg16::DX => write!(f, "dx"),
            Reg16::BX => write!(f, "bx"),
            Reg16::SP => write!(f, "sp"),
            Reg16::BP => write!(f, "bp"),
            Reg16::SI => write!(f, "si"),
            Reg16::DI => write!(f, "di"),
            Reg16::None => write!(f, "none"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum Reg8 {
    AL = constant::reg::AL,
    CL = constant::reg::CL,
    DL = constant::reg::DL,
    BL = constant::reg::BL,
    AH = constant::reg::AH,
    CH = constant::reg::CH,
    DH = constant::reg::DH,
    BH = constant::reg::BH,
    None,
}

impl Debug for Reg8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg8::AL => write!(f, "al"),
            Reg8::CL => write!(f, "cl"),
            Reg8::DL => write!(f, "dl"),
            Reg8::BL => write!(f, "bl"),
            Reg8::AH => write!(f, "ah"),
            Reg8::CH => write!(f, "ch"),
            Reg8::DH => write!(f, "dh"),
            Reg8::BH => write!(f, "bh"),
            Reg8::None => write!(f, "hoge"),
        }
    }
}

impl From<u8> for Reg16 {
    fn from(num: u8) -> Self {
        let num = num as isize;
        match num {
            constant::reg::AX => Reg16::AX,
            constant::reg::CX => Reg16::CX,
            constant::reg::DX => Reg16::DX,
            constant::reg::BX => Reg16::BX,
            constant::reg::SP => Reg16::SP,
            constant::reg::BP => Reg16::BP,
            constant::reg::SI => Reg16::SI,
            constant::reg::DI => Reg16::DI,
            _ => Reg16::None,
        }
    }
}

impl From<u8> for Reg8 {
    fn from(num: u8) -> Self {
        let num = num as isize;
        match num {
            constant::reg::AL => Reg8::AL,
            constant::reg::CL => Reg8::CL,
            constant::reg::DL => Reg8::DL,
            constant::reg::BL => Reg8::BL,
            constant::reg::AH => Reg8::AH,
            constant::reg::CH => Reg8::CH,
            constant::reg::DH => Reg8::DH,
            constant::reg::BH => Reg8::BH,
            _ => Reg8::None,
        }
    }
}
