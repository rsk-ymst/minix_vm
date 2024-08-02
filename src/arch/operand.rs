use super::reg::Register;
use std::fmt::Debug;

#[derive(PartialEq, Clone)]
pub enum Operand {
    Register(Register),
    Immediate(ImmediateValue),
    EffectiveAddress(EA),
}

impl Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register(reg) => write!(f, "{:?}", reg),
            Self::Immediate(imm) => write!(f, "{:?}", imm),
            Self::EffectiveAddress(ea) => write!(f, "{:?}", ea),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Memory {
    base: Register,
    index: Register,
    scale: i32,
    displacement: i32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ImmediateValue {
    I8(i8, usize),
    I16(i16, usize),
    I32(i32, usize),
}

impl Debug for ImmediateValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I8(i, display_size) => match (*i, *display_size) {
                (0..=10, _) => write!(f, "{i:01x}"),
                (_, 2) => write!(f, "{i:02x}"),
                (-127..=127, 3) => write!(f, "{i:03x}"),
                (_, 4) => write!(f, "{i}"),
                _ => write!(f, "{i:02x}"),
            },
            Self::I16(i, display_size) => match (*i, *display_size) {
                (-9..=-1, 1) => write!(f, "{i}"), // cmp immediate
                (_, 2) => write!(f, "{i:02x}"),
                (_, 4) => write!(f, "{i:04x}"),
                (_, _) => write!(f, "{i:01x}"),
            },
            Self::I32(i, _) => write!(f, "{i:04x}"),
        }
    }
}

impl Into<u16> for ImmediateValue {
    fn into(self) -> u16 {
        match self {
            Self::I8(val, _) => val as u16,
            Self::I16(val, _) => val as u16,
            Self::I32(val, _) => val as u16,
        }
    }
}

impl Into<i16> for ImmediateValue {
    fn into(self) -> i16 {
        match self {
            Self::I8(val, _) => val as i16,
            Self::I16(val, _) => val as i16,
            Self::I32(val, _) => val as i16,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Disp(pub isize);

impl Debug for Disp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Disp(value) => {
                if *value > 0 {
                    return write!(f, "+{:0x}", value);
                }

                if *value < 0 {
                    let value = !*value + 1;
                    return write!(f, "-{:0x}", value);
                }

                return write!(f, "");
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum EA {
    BxSi(Disp),
    BxDi(Disp),
    BpSi(Disp),
    BpDi(Disp),
    Si(Disp),
    Di(Disp),
    Bp(Disp),
    Bx(Disp),
    DispOnly(Disp),
}

impl Debug for EA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EA::BxSi(disp) => write!(f, "[bx+si{:?}]", disp),
            EA::BxDi(disp) => write!(f, "[bx+di{:?}]", disp),
            EA::BpSi(disp) => write!(f, "[bp+si{:?}]", disp),
            EA::BpDi(disp) => write!(f, "[bp+di{:?}]", disp),
            EA::Si(disp) => write!(f, "[si{:?}]", disp),
            EA::Di(disp) => write!(f, "[di{:?}]", disp),
            EA::Bp(disp) => write!(f, "[bp{:?}]", disp),
            EA::Bx(disp) => write!(f, "[bx{:?}]", disp),
            EA::DispOnly(Disp(val)) => write!(f, "[{:04x}]", val),
        }
    }
}

impl EA {
    pub fn new(rm: u8, value: isize) -> Self {
        let value = Disp(value);

        match rm {
            0b000 => EA::BxSi(value),
            0b001 => EA::BxDi(value),
            0b010 => EA::BpSi(value),
            0b011 => EA::BpDi(value),
            0b100 => EA::Si(value),
            0b101 => EA::Di(value),
            0b110 => EA::Bp(value),
            0b111 => EA::Bx(value),
            _ => panic!("Invalid EA"),
        }
    }
}
