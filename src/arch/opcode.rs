use super::constant;
use std::fmt::Debug;

#[derive(PartialEq, Clone, Copy)]
pub enum Opcode {
    // Mov
    MovImmediateRegisterMemory,
    MovImmediateRegisterMemoryWord,
    MovImmediateRegisterMemoryByte,
    MovImmediate,
    MovMemoryToAccumulator,

    // Push
    PushRegMem,
    PushReg,
    PushSegReg,

    // Pop
    PopRegMem,
    PopReg,
    PopSegReg,

    // Xchg
    XchgRegisterMemoryWithRegister,
    XchgRegisterWithAccumulator,

    // Int
    IntTypeSpecified,

    // Add
    AddRegEither,
    AddImmediateRegisterMemory,
    AddImmediateFromAccumulator,
    AddImmediateToAccumulator,

    OrRegEither,
    OrImmediateRegisterMemory,
    OrImmediateFromAccumulator,

    SubRegEither,
    SubImmediateRegisterMemory,
    SubImmediateFromAccumulator,

    AdcRegEither,
    AdcImmediateRegisterMemory,
    AdcImmediateFromAccumulator,

    SsbRegEither,
    SsbImmediateRegisterMemory,
    SsbImmediateFromAccumulator,

    AndRegEither,
    AndImmediateRegisterMemory,
    AndImmediateFromAccumulator,

    MovRmToFromReg,
    XorRegEither,

    TestRegisterMemoryAndRegister,
    TestImmediate,
    TestImmediateByte,
    TestImmediateDataAndAccumulator,

    CallWithinDirect,

    Rep,
    CompsByte,
    CompsWord,

    // cmp
    CmpImmediateWord,
    CmpImmediateByte,
    CmpRegEither,
    CmpImmediateFromAccumulator,

    Lea,
    Lds,
    Les,

    JmpDirectWithinSegment,
    JmpDirectWithinSegmentShort,
    JmpIndirectWithinSegment,

    Shl,
    Shr,
    Sar,
    Rol,
    Ror,
    Rcl,
    Rcr,

    Neg,
    Not,

    RetWithinSegment,
    RetWithinSegAddingImmedToSp,
    RetIntersegment,
    RetIntersegmentAddingImmediateToSp,

    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jne,
    Jnl,
    Jnle,
    Jnb,
    Jnbe,
    Jnp,
    Jno,
    Jns,

    Loop,
    Loopz,
    Loopnz,
    Jcxz,

    Clc,
    Cmc,
    Stc,
    Cld,
    Std,
    Cli,
    Sti,
    Hlt,
    Wait,
    Esc,
    Lock,

    InFixedPort,
    InVariablePort,

    OutFixedPort,
    OutVariablePort,

    Cbw,
    Cwd,

    IncRegisterMemory,
    IncRegister,

    DecRegisterMemory,
    DecRegister,

    Mul,
    Imul,
    Div,
    Idiv,

    Undefined,
    NOP,

    RepMovsw,
    RepMovsb,
    RepStosb,
    RepScasb,
}

impl Opcode {
    pub fn is_calculated(&self) -> bool {
        match self {
            Opcode::OrRegEither
            | Opcode::OrImmediateRegisterMemory
            | Opcode::OrImmediateFromAccumulator
            | Opcode::AdcRegEither
            | Opcode::AdcImmediateRegisterMemory
            | Opcode::AdcImmediateFromAccumulator
            | Opcode::SsbRegEither
            | Opcode::SsbImmediateRegisterMemory
            | Opcode::SsbImmediateFromAccumulator
            | Opcode::AndRegEither
            | Opcode::AndImmediateRegisterMemory
            | Opcode::AndImmediateFromAccumulator
            | Opcode::XorRegEither
            | Opcode::TestImmediate
            | Opcode::TestImmediateByte
            | Opcode::Shl
            | Opcode::Shr
            | Opcode::Sar
            | Opcode::Rol
            | Opcode::Ror
            | Opcode::Rcl
            | Opcode::Rcr
            | Opcode::Neg
            | Opcode::Not
            | Opcode::Jnp
            | Opcode::Jno
            | Opcode::Jns
            | Opcode::Loop
            | Opcode::Loopz
            | Opcode::Loopnz
            | Opcode::Jcxz
            | Opcode::Clc
            | Opcode::Cmc
            | Opcode::Stc
            | Opcode::Std
            | Opcode::Cli
            | Opcode::Sti
            | Opcode::Hlt
            | Opcode::Wait
            | Opcode::Esc
            | Opcode::Lock
            | Opcode::InFixedPort
            | Opcode::InVariablePort
            | Opcode::OutFixedPort
            | Opcode::OutVariablePort
            | Opcode::IncRegisterMemory
            | Opcode::IncRegister
            | Opcode::DecRegisterMemory
            | Opcode::DecRegister
            | Opcode::Imul
            | Opcode::Undefined
            | Opcode::NOP => true,
            _ => false,
        }
    }

    pub fn could_be_over_flow(&self) -> bool {
        match self {
            Opcode::Rol
            | Opcode::Ror
            | Opcode::Rcl
            | Opcode::Rcr
            | Opcode::Not
            | Opcode::IncRegisterMemory
            | Opcode::IncRegister
            | Opcode::Imul
            | Opcode::Div
            | Opcode::Idiv
            | Opcode::Undefined
            | Opcode::NOP => true,
            _ => false,
        }
    }

    pub fn could_be_carried(&self) -> bool {
        match self {
            Opcode::OrRegEither
            | Opcode::OrImmediateRegisterMemory
            | Opcode::OrImmediateFromAccumulator
            | Opcode::TestImmediate
            | Opcode::Shr
            | Opcode::Rol
            | Opcode::Ror
            | Opcode::Rcl
            | Opcode::Rcr
            | Opcode::Neg => true,
            _ => false,
        }
    }

    pub fn is_assign_effect(&self) -> bool {
        match self {
            Opcode::MovImmediateRegisterMemory
            | Opcode::MovImmediateRegisterMemoryWord
            | Opcode::MovImmediateRegisterMemoryByte
            | Opcode::MovImmediate
            | Opcode::MovRmToFromReg
            | Opcode::MovMemoryToAccumulator
            | Opcode::XchgRegisterMemoryWithRegister
            | Opcode::XchgRegisterWithAccumulator
            | Opcode::AddRegEither
            | Opcode::AddImmediateRegisterMemory
            | Opcode::AddImmediateFromAccumulator
            | Opcode::AddImmediateToAccumulator
            | Opcode::OrRegEither
            | Opcode::OrImmediateRegisterMemory
            | Opcode::OrImmediateFromAccumulator
            | Opcode::SubRegEither
            | Opcode::SubImmediateRegisterMemory
            | Opcode::SubImmediateFromAccumulator
            | Opcode::AdcRegEither
            | Opcode::AdcImmediateRegisterMemory
            | Opcode::AdcImmediateFromAccumulator
            | Opcode::SsbRegEither
            | Opcode::SsbImmediateRegisterMemory
            | Opcode::SsbImmediateFromAccumulator
            | Opcode::AndRegEither
            | Opcode::AndImmediateRegisterMemory
            | Opcode::AndImmediateFromAccumulator
            | Opcode::XorRegEither
            | Opcode::Lea
            | Opcode::Lds
            | Opcode::Les
            | Opcode::Shl
            | Opcode::Shr
            | Opcode::Sar
            | Opcode::Rol
            | Opcode::Ror
            | Opcode::Rcl
            | Opcode::Rcr
            | Opcode::Neg
            | Opcode::IncRegisterMemory
            | Opcode::IncRegister
            | Opcode::DecRegisterMemory
            | Opcode::DecRegister
            | Opcode::Imul => true,
            _ => false,
        }
    }
}

impl Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::PushRegMem | Opcode::PushReg | Opcode::PushSegReg => write!(f, "push"),
            Opcode::PopReg | Opcode::PopRegMem => write!(f, "pop"),
            Opcode::MovImmediateRegisterMemory
            | Opcode::MovImmediate
            | Opcode::MovRmToFromReg
            | Opcode::MovMemoryToAccumulator => {
                write!(f, "mov")
            }
            Opcode::MovImmediateRegisterMemoryByte => write!(f, "mov byte"),
            Opcode::MovImmediateRegisterMemoryWord => write!(f, "mov word"),
            Opcode::XchgRegisterMemoryWithRegister | Opcode::XchgRegisterWithAccumulator => {
                write!(f, "xchg")
            }
            Opcode::IntTypeSpecified => write!(f, "int"),
            Opcode::AddRegEither
            | Opcode::AddImmediateRegisterMemory
            | Opcode::AddImmediateToAccumulator
            | Opcode::AddImmediateFromAccumulator => write!(f, "add"),
            Opcode::AdcRegEither
            | Opcode::AdcImmediateRegisterMemory
            | Opcode::AdcImmediateFromAccumulator => write!(f, "adc"),
            Opcode::Neg => write!(f, "neg"),
            Opcode::SsbRegEither
            | Opcode::SsbImmediateRegisterMemory
            | Opcode::SsbImmediateFromAccumulator => write!(f, "sbb"),
            Opcode::SubRegEither
            | Opcode::SubImmediateRegisterMemory
            | Opcode::SubImmediateFromAccumulator => write!(f, "sub"),
            Opcode::AndRegEither
            | Opcode::AndImmediateRegisterMemory
            | Opcode::AndImmediateFromAccumulator => write!(f, "and"),
            Opcode::OrRegEither
            | Opcode::OrImmediateRegisterMemory
            | Opcode::OrImmediateFromAccumulator => write!(f, "or"),
            Opcode::Mul => write!(f, "mul"),
            Opcode::Imul => write!(f, "imul"),
            Opcode::Div => write!(f, "div"),
            Opcode::Idiv => write!(f, "idiv"),
            Opcode::Not => write!(f, "not"),
            Opcode::XorRegEither => write!(f, "xor"),
            Opcode::CmpImmediateWord
            | Opcode::CmpRegEither
            | Opcode::CmpImmediateFromAccumulator => write!(f, "cmp"),
            Opcode::CmpImmediateByte => write!(f, "cmp byte"),
            Opcode::CallWithinDirect => write!(f, "call"),
            Opcode::JmpDirectWithinSegment | Opcode::JmpIndirectWithinSegment => write!(f, "jmp"),
            Opcode::JmpDirectWithinSegmentShort => write!(f, "jmp short"),
            Opcode::IncRegisterMemory | Opcode::IncRegister => write!(f, "inc"),
            Opcode::DecRegisterMemory | Opcode::DecRegister => write!(f, "dec"),

            Opcode::RetWithinSegment
            | Opcode::RetIntersegmentAddingImmediateToSp
            | Opcode::RetWithinSegAddingImmedToSp => write!(f, "ret"),

            Opcode::Je => write!(f, "je"),
            Opcode::Jl => write!(f, "jl"),
            Opcode::Jle => write!(f, "jle"),
            Opcode::Jb => write!(f, "jb"),
            Opcode::Jbe => write!(f, "jbe"),
            Opcode::Jp => write!(f, "jp"),
            Opcode::Jo => write!(f, "jo"),
            Opcode::Js => write!(f, "js"),
            Opcode::Jne => write!(f, "jne"),
            Opcode::Jnl => write!(f, "jnl"),
            Opcode::Jnle => write!(f, "jnle"),
            Opcode::Jnb => write!(f, "jnb"),
            Opcode::Jnbe => write!(f, "jnbe"),
            Opcode::Jnp => write!(f, "jnp"),
            Opcode::Jno => write!(f, "jno"),
            Opcode::Jns => write!(f, "jns"),

            Opcode::Loop => write!(f, "loop"),
            Opcode::Loopz => write!(f, "loopz"),
            Opcode::Loopnz => write!(f, "loopnz"),
            Opcode::Jcxz => write!(f, "jcxz"),

            Opcode::RepMovsb => write!(f, "rep movsb"),
            Opcode::RepMovsw => write!(f, "rep movsw"),
            Opcode::RepScasb => write!(f, "rep scasb"),
            Opcode::RepStosb => write!(f, "rep stosb"),

            Opcode::CompsByte => write!(f, "cmpsb"),

            Opcode::Lea => write!(f, "lea"),
            Opcode::Lds => write!(f, "lds"),
            Opcode::Les => write!(f, "les"),
            Opcode::Hlt => write!(f, "hlt"),
            Opcode::Clc => write!(f, "clc"),
            Opcode::Cmc => write!(f, "cmc"),
            Opcode::Cld => write!(f, "cld"),
            Opcode::Std => write!(f, "std"),
            Opcode::Cli => write!(f, "cli"),
            Opcode::Sti => write!(f, "sti"),
            Opcode::Cbw => write!(f, "cbw"),
            Opcode::Cwd => write!(f, "cwd"),

            Opcode::Shl => write!(f, "shl"),
            Opcode::Shr => write!(f, "shr"),
            Opcode::Sar => write!(f, "sar"),
            Opcode::Rol => write!(f, "rol"),
            Opcode::Ror => write!(f, "ror"),
            Opcode::Rcl => write!(f, "rcl"),
            Opcode::Rcr => write!(f, "rcr"),

            Opcode::TestRegisterMemoryAndRegister
            | Opcode::TestImmediate
            | Opcode::TestImmediateDataAndAccumulator => write!(f, "test"),

            Opcode::TestImmediateByte => write!(f, "test byte"),

            Opcode::InFixedPort | Opcode::InVariablePort => write!(f, "in"),
            Opcode::OutFixedPort | Opcode::OutVariablePort => write!(f, "out"),
            Opcode::Undefined => write!(f, "(undefined)"),
            _ => write!(f, "hoge"),
        }
    }
}
