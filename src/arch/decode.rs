use super::asm::Assembly;
use super::bin::BinaryConsume;
use super::bin::BinaryPeek;
use super::constant::opcode::ADD_IMMEDIATE_TO_ACCUMULATOR;
use super::constant::opcode::CALL_DIRECT_WITHIN_SEGMENT;
use super::constant::opcode::CALL_INDIRECT_INTERSEGMENT_3BIT;
use super::constant::opcode::CALL_INDIRECT_INTERSEGMENT_8BIT;
use super::constant::opcode::CALL_INDIRECT_WITHIN_SEGMENT_3BIT;
use super::constant::opcode::CALL_INDIRECT_WITHIN_SEGMENT_8BIT;
use super::constant::opcode::COMPS;
use super::constant::opcode::DEC_REGISTER;
use super::constant::opcode::HLT;
use super::constant::opcode::INT_TYPE_SPECIFIED;
use super::constant::opcode::JMP_DIRECT_WITHIN_SEGMENT;
use super::constant::opcode::JMP_DIRECT_WITHIN_SEGMENT_SHORT;
use super::constant::opcode::JMP_INDIRECT_WITHIN_SEGMENT_3BIT;
use super::constant::opcode::MOV_MEMORY_TO_REGISTER;
use super::constant::opcode::POP_REG;
use super::constant::opcode::POP_RM;
use super::constant::opcode::PUSH_REG;
use super::constant::opcode::PUSH_RM;
use super::constant::opcode::SHL_3BIT;
use super::operand::ImmediateValue;
use super::vm::VM;
use super::{
    asm::Instruction,
    bin::{self, reverse_order_u16},
    opcode::Opcode,
    operand::{Disp, Operand, EA},
    reg::{Reg16, Reg8, Register},
};
use crate::arch::decode::ImmediateValue::I16;
use crate::arch::decode::ImmediateValue::I8;

use super::constant::opcode::{
    ADC_IMMEDIATE_WITH_ACCUMULATOR, ADC_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT, ADC_REG_EITHER,
    ADD_IMMEDIATE_WITH_ACCUMULATOR, ADD_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT, ADD_REG_EITHER,
    AND_IMMEDIATE_WITH_ACCUMULATOR, AND_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT, AND_REG_EITHER, CLC,
    CLD, CLI, CMC, CMP_IMMEDIATE_WITH_ACCUMULATOR, CMP_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT,
    CMP_REG_EITHER, CONVERT_BYTE_TO_BYTE, CONVERT_WORD_TO_DOUBLE_WORD, DEC_REGISTER_MEMORY_3BIT,
    DIV_3BIT, IDIV_3BIT, IMMEDIATE_DATA, IMMEDIATE_WITH_REGISTER_MEMORY_6BIT, IMUL_3BIT,
    INC_DEC_7BIT, INC_REGISTER, INC_REGISTER_MEMORY_3BIT, IN_FIXED_PORT, IN_VARIABLE_PORT, JB, JBE,
    JCXZ, JE, JL, JLE, JNB, JNBE, JNE, JNL, JNLE, JNO, JNP, JNS, JO, JP, JS, LEA, LOGIC_6BIT, LOOP,
    LOOPNZ, LOOPZ, MOV_IMMEDIATE, MOV_IMMEDIATE_REGISTER_MEMORY_7BIT,
    MOV_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT, MOV_RM_TO_FROM_REG, MUL_3BIT, MUL_DIV_7BIT, NEG_3BIT,
    NOT_3BIT, OR_IMMEDIATE_WITH_ACCUMULATOR, OR_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT, OR_REG_EITHER,
    OUT_FIXED_PORT, OUT_VARIABLE_PORT, RCL_3BIT, RCR_3BIT, REP, RET_WITHIN_SEGMENT,
    RET_WITHIN_SEG_ADDING_IMMED_TO_SP, ROL_3BIT, ROR_3BIT, SAR_3BIT, SHR_3BIT,
    SSB_IMMEDIATE_WITH_ACCUMULATOR, SSB_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT, SSB_REG_EITHER, STD,
    STI, SUB_IMMEDIATE_WITH_ACCUMULATOR, SUB_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT, SUB_REG_EITHER,
    TEST_IMMEDIATE_DATA_AND_ACCUMULATOR, TEST_IMMEDIATE_RM, TEST_REGISTER_MEMORY_AND_REGISTER,
    XCHG_REGISTER_MEMORY_WITH_REGISTER, XCHG_REGISTER_WITH_ACCUMULATOR, XOR_REG_EITHER,
};

impl VM {
    pub fn decode(&mut self) -> Option<Assembly> {
        if self.ip as usize >= self.text_size {
            return None;
        }

        if !cfg!(feature = "vm") && self.ip as usize == self.text_size - 1 {
            return self.undefined();
        }

        let cur_8bits = self.peek_u8()? as isize;
        let next_8bits = self.peek_offset(1);

        let cur_upper_4bits = (cur_8bits >> 4 & 0xff) as isize;
        let cur_upper_5bits = (cur_8bits >> 3 & 0xff) as isize;
        let cur_upper_6bits = (cur_8bits >> 2 & 0xff) as isize;
        let cur_upper_7bits = (cur_8bits >> 1 & 0xff) as isize;

        let next_3bits = next_8bits? as isize >> 3 & 0b111;

        match cur_upper_4bits {
            MOV_IMMEDIATE => {
                return self.immediate_register();
            }
            _ => (),
        }

        if let Some(opcode) = match cur_upper_5bits {
            PUSH_REG => Some(Opcode::PushReg),
            POP_REG => Some(Opcode::PopReg),
            XCHG_REGISTER_WITH_ACCUMULATOR => Some(Opcode::XchgRegisterWithAccumulator),
            DEC_REGISTER => Some(Opcode::DecRegister),
            INC_REGISTER => Some(Opcode::IncRegister),
            _ => None,
        } {
            return self.reg_series(opcode);
        }

        match cur_upper_6bits {
            ADD_REG_EITHER => {
                return self.reg_either(Opcode::AddRegEither);
            }
            SUB_REG_EITHER => {
                return self.reg_either(Opcode::SubRegEither);
            }
            AND_REG_EITHER => {
                return self.reg_either(Opcode::AndRegEither);
            }
            CMP_REG_EITHER => {
                return self.reg_either(Opcode::CmpRegEither);
            }
            OR_REG_EITHER => {
                return self.reg_either(Opcode::OrRegEither);
            }
            ADC_REG_EITHER => {
                return self.reg_either(Opcode::AdcRegEither);
            }
            IMMEDIATE_WITH_REGISTER_MEMORY_6BIT => match next_3bits {
                ADD_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT => {
                    return self.immediate_register_memory(Opcode::AddImmediateRegisterMemory);
                }
                SUB_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT => {
                    return self.immediate_register_memory(Opcode::SubImmediateRegisterMemory);
                }
                AND_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT => {
                    return self.immediate_register_memory(Opcode::AndImmediateRegisterMemory);
                }
                ADC_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT => {
                    return self.immediate_register_memory(Opcode::AdcImmediateRegisterMemory);
                }
                OR_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT => {
                    return self.immediate_register_memory(Opcode::OrImmediateRegisterMemory);
                }
                CMP_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT => {
                    let w = cur_8bits & 0b1;
                    let opcode = if w == 1 {
                        Opcode::CmpImmediateWord
                    } else {
                        Opcode::CmpImmediateByte
                    };
                    return self.immediate_register_memory(opcode);
                }
                SSB_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT => {
                    return self.immediate_register_memory(Opcode::SsbImmediateRegisterMemory);
                }
                _ => (),
            },
            LOGIC_6BIT => match next_3bits {
                SHL_3BIT => {
                    return self.register_memory(Opcode::Shl);
                }
                SHR_3BIT => {
                    return self.register_memory(Opcode::Shr);
                }
                SAR_3BIT => {
                    return self.register_memory(Opcode::Sar);
                }
                ROL_3BIT => {
                    return self.register_memory(Opcode::Rol);
                }
                ROR_3BIT => {
                    return self.register_memory(Opcode::Ror);
                }
                RCL_3BIT => {
                    return self.register_memory(Opcode::Rcl);
                }
                RCR_3BIT => {
                    return self.register_memory(Opcode::Rcr);
                }
                _ => (),
            },

            SSB_REG_EITHER => {
                return self.reg_either(Opcode::SsbImmediateRegisterMemory);
            }
            XOR_REG_EITHER => {
                return self.reg_either(Opcode::XorRegEither);
            }
            MOV_RM_TO_FROM_REG => {
                return self.reg_either(Opcode::MovRmToFromReg);
            }
            _ => (),
        }

        match (cur_upper_7bits, next_3bits) {
            (MOV_IMMEDIATE_REGISTER_MEMORY_7BIT, MOV_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT) => {
                let w = cur_8bits & 0b1;
                let opcode = if w == 0 {
                    Opcode::MovImmediateRegisterMemoryByte
                } else {
                    Opcode::MovImmediateRegisterMemory
                };
                return self.immediate_register_memory(opcode);
            }
            (MOV_MEMORY_TO_REGISTER, _) => {
                return self.memory_to_register(Opcode::MovMemoryToAccumulator);
            }
            (XCHG_REGISTER_MEMORY_WITH_REGISTER, _) => {
                return self.register_memory(Opcode::XchgRegisterMemoryWithRegister);
            }
            (ADD_IMMEDIATE_WITH_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::AddImmediateFromAccumulator);
            }
            (ADD_IMMEDIATE_TO_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::AddImmediateToAccumulator);
            }
            (SUB_IMMEDIATE_WITH_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::SubImmediateFromAccumulator);
            }
            (ADC_IMMEDIATE_WITH_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::SubImmediateFromAccumulator);
            }
            (AND_IMMEDIATE_WITH_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::AndImmediateFromAccumulator);
            }
            (TEST_IMMEDIATE_DATA_AND_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::TestImmediateDataAndAccumulator);
            }
            (TEST_REGISTER_MEMORY_AND_REGISTER, _) => {
                return self.register_memory(Opcode::TestRegisterMemoryAndRegister);
            }
            (SSB_IMMEDIATE_WITH_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::AndImmediateFromAccumulator);
            }
            (CMP_IMMEDIATE_WITH_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::CmpImmediateFromAccumulator);
            }
            (OR_IMMEDIATE_WITH_ACCUMULATOR, _) => {
                return self.immediate_accumulator(Opcode::OrImmediateFromAccumulator);
            }
            (IMMEDIATE_DATA, TEST_IMMEDIATE_RM) => {
                return self.immediate_register_memory(Opcode::TestImmediate);
            }
            (MUL_DIV_7BIT, NEG_3BIT) => {
                return self.register_memory(Opcode::Neg);
            }
            (MUL_DIV_7BIT, NOT_3BIT) => {
                return self.register_memory(Opcode::Not);
            }
            (MUL_DIV_7BIT, MUL_3BIT) => {
                return self.register_memory(Opcode::Mul);
            }
            (MUL_DIV_7BIT, IMUL_3BIT) => {
                return self.register_memory(Opcode::Imul);
            }
            (MUL_DIV_7BIT, DIV_3BIT) => {
                return self.register_memory(Opcode::Div);
            }
            (MUL_DIV_7BIT, IDIV_3BIT) => {
                return self.register_memory(Opcode::Idiv);
            }
            (INC_DEC_7BIT, INC_REGISTER_MEMORY_3BIT) => {
                return self.register_memory(Opcode::IncRegisterMemory);
            }
            (INC_DEC_7BIT, DEC_REGISTER_MEMORY_3BIT) => {
                return self.register_memory(Opcode::DecRegisterMemory);
            }
            (IN_FIXED_PORT, _) => {
                return self.fixed_port(Opcode::InFixedPort);
            }
            (IN_VARIABLE_PORT, _) => {
                return self.fixed_port(Opcode::InVariablePort);
            }
            (OUT_FIXED_PORT, _) => {
                return self.fixed_port(Opcode::OutFixedPort);
            }
            (OUT_VARIABLE_PORT, _) => {
                return self.fixed_port(Opcode::OutVariablePort);
            }
            (REP, _) => {
                return self.string_manipulation(Opcode::Rep);
            }
            (COMPS, _) => {
                return self.string_manipulation(Opcode::CompsByte);
            }

            _ => (),
        }

        match (cur_8bits, next_3bits) {
            (INT_TYPE_SPECIFIED, _) => {
                return self.int_specified();
            }
            (LEA, _) => {
                return self.load(Opcode::Lea);
            }
            (JMP_DIRECT_WITHIN_SEGMENT, _) => {
                return self.direct_seg(Opcode::JmpDirectWithinSegment);
            }
            (JMP_DIRECT_WITHIN_SEGMENT_SHORT, _) => {
                return self.direct_seg_short(Opcode::JmpDirectWithinSegmentShort);
            }
            (CALL_INDIRECT_INTERSEGMENT_8BIT, CALL_INDIRECT_INTERSEGMENT_3BIT) => {
                return self.register_memory(Opcode::CallWithinDirect);
            }

            (CALL_INDIRECT_WITHIN_SEGMENT_8BIT, CALL_INDIRECT_WITHIN_SEGMENT_3BIT) => {
                return self.register_memory(Opcode::CallWithinDirect);
            }
            (CALL_INDIRECT_WITHIN_SEGMENT_8BIT, JMP_INDIRECT_WITHIN_SEGMENT_3BIT) => {
                return self.register_memory(Opcode::JmpIndirectWithinSegment);
            }

            (RET_WITHIN_SEGMENT, _) => {
                return self.proc_control(Opcode::RetWithinSegment);
            }
            (JE, _) => {
                return self.direct_seg_short(Opcode::Je);
            }
            (JL, _) => {
                return self.direct_seg_short(Opcode::Jl);
            }
            (JLE, _) => {
                return self.direct_seg_short(Opcode::Jle);
            }
            (JB, _) => {
                return self.direct_seg_short(Opcode::Jb);
            }
            (JBE, _) => {
                return self.direct_seg_short(Opcode::Jbe);
            }
            (JP, _) => {
                return self.direct_seg_short(Opcode::Jp);
            }
            (JO, _) => {
                return self.direct_seg_short(Opcode::Jo);
            }
            (JS, _) => {
                return self.direct_seg_short(Opcode::Js);
            }
            (JNE, _) => {
                return self.jmp_disp(Opcode::Jne);
            }
            (JNL, _) => {
                return self.jmp_disp(Opcode::Jnl);
            }
            (JNLE, _) => {
                return self.jmp_disp(Opcode::Jnle);
            }
            (JNB, _) => {
                return self.jmp_disp(Opcode::Jnb);
            }
            (JNBE, _) => {
                return self.jmp_disp(Opcode::Jnbe);
            }
            (JNP, _) => {
                return self.jmp_disp(Opcode::Jnp);
            }
            (JNO, _) => {
                return self.jmp_disp(Opcode::Jno);
            }
            (JNS, _) => {
                return self.jmp_disp(Opcode::Jns);
            }
            (LOOP, _) => {
                return self.jmp_disp(Opcode::Loop);
            }
            (LOOPZ, _) => {
                return self.jmp_disp(Opcode::Loopz);
            }
            (LOOPNZ, _) => {
                return self.jmp_disp(Opcode::Loopnz);
            }
            (JCXZ, _) => {
                return self.jmp_disp(Opcode::Jcxz);
            }
            (PUSH_RM, _) => {
                return self.register_memory(Opcode::PushRegMem);
            }
            (POP_RM, _) => {
                return self.register_memory(Opcode::PopRegMem);
            }
            (CALL_DIRECT_WITHIN_SEGMENT, _) => {
                return self.direct_seg(Opcode::CallWithinDirect);
            }
            // PROC CONTROL
            (CLC, _) => {
                return self.proc_control(Opcode::Clc);
            }
            (CMC, _) => {
                return self.proc_control(Opcode::Cmc);
            }
            (CLD, _) => {
                return self.proc_control(Opcode::Cld);
            }
            (STD, _) => {
                return self.proc_control(Opcode::Std);
            }
            (CLI, _) => {
                return self.proc_control(Opcode::Cli);
            }
            (STI, _) => {
                return self.proc_control(Opcode::Sti);
            }
            (HLT, _) => {
                return self.proc_control(Opcode::Hlt);
            }
            (CONVERT_BYTE_TO_BYTE, _) => {
                return self.proc_control(Opcode::Cbw);
            }
            (CONVERT_WORD_TO_DOUBLE_WORD, _) => {
                return self.proc_control(Opcode::Cwd);
            }
            (RET_WITHIN_SEG_ADDING_IMMED_TO_SP, _) => {
                return self.immed_to_sp(Opcode::RetWithinSegAddingImmedToSp);
            }
            _ => (),
        }

        return self.undefined();
    }

    // pub fn get_asm(&self) -> Vec<Assembly> {
    //     self.asm.clone()
    // }
}

pub trait TextParser {
    fn memory_to_register(&mut self, opcode: Opcode) -> Option<Assembly>;
    // immediate register
    fn immediate_register(&mut self) -> Option<Assembly>;

    fn int_specified(&mut self) -> Option<Assembly>;

    // Register/Memory to/from Register
    // Reg./Memory with Register to Either
    // mov(8b), ssb
    // ------sw, mod---r/m
    fn reg_either(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Load {EA to Register | Pointer to DS | Pointer to ES}
    fn load(&mut self, opcode: Opcode) -> Option<Assembly>;

    fn sub_immediate(&mut self) -> Option<Assembly>;

    // fn shit_control(&mut self) -> Option<Assembly>;

    // {jmp, dl, call} disp
    // on Equal/Zero
    fn jmp_disp(&mut self, opcode: Opcode) -> Option<Assembly>;

    // ------sw, mod---r/m, data, data if sw = 01
    fn immediate_register_memory(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Add, Sub, Ssb,
    // ------sw, data, data if sw = 01
    fn immediate_accumulator(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Register/Memory
    // XXXXXXXX mod_110_r/m, ff

    // shl/sal | shr | sar | rol | ror | rol | rcl | rcr
    // ------vw mod---r/m

    // Indirect Intersegment
    // Indirect within Segment
    // Indirect Intersegment
    // -------- mod---r/m
    fn register_memory(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Register
    // XXXXX_reg
    fn reg_series(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Direct within Segment
    // {call | jmp} disp-low, disp-high
    fn direct_seg(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Direct within Segment-Short
    // {jmp | je | jl ... etc} disp
    fn direct_seg_short(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Direct within Segment
    // cbw, cwd
    // --------
    fn proc_control(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Fixed port
    // In, Out
    fn fixed_port(&mut self, opcode: Opcode) -> Option<Assembly>;

    // Variable port
    // In, Out
    fn variable_port(&mut self, opcode: Opcode) -> Option<Assembly>;

    fn string_manipulation(&mut self, opcode: Opcode) -> Option<Assembly>;

    // data-low, data-high
    // Within Seg Adding Immed to SP
    // Intersegment Adding Immediate to SP
    fn immed_to_sp(&mut self, opcode: Opcode) -> Option<Assembly>;

    fn undefined(&mut self) -> Option<Assembly>;
}

impl TextParser for VM {
    fn immediate_register(&mut self) -> Option<Assembly> {
        let address = self.ip;
        let target = self.consume_u8()?;

        let lower_4bits = target & 0xf;
        let w = lower_4bits >> 3 & 0b1;
        let reg = lower_4bits & 0b111;

        let upper_data_byte = self.consume_u8()? as u16;
        let lower_data_byte = self.consume_u8()? as u16;

        let code =
            (target as usize) << 16 | (upper_data_byte as usize) << 8 | (lower_data_byte as usize);

        let mut register: Register = Register::None;
        let mut value: u16 = 0;

        if w == 1 {
            register = Register::Reg16(Reg16::from(reg));
            value = (lower_data_byte << 8) | upper_data_byte as u16; // 即値はリトルエンディアンなので入れ替える
        } else {
            register = Register::Reg8(Reg8::from(reg));
            value = lower_data_byte as u16;
        }

        let instruction = Instruction {
            opcode: Opcode::MovImmediate,
            operand1: Some(Operand::Register(register)),
            operand2: Some(Operand::Immediate(ImmediateValue::I32(value as i32, 4))),
        };

        let asm = Assembly {
            address,
            size: 3,
            code,
            instruction,
        };

        Some(asm)
    }

    fn int_specified(&mut self) -> Option<Assembly> {
        let address = self.ip;

        let opcode = self.consume_u8()?;
        let value = self.consume_u8()?;

        let code = ((opcode as u16) << 8 | value as u16) as usize;
        let immediate = Some(Operand::Immediate(I8(value as i8, 2)));

        let instruction = Instruction {
            opcode: Opcode::IntTypeSpecified,
            operand1: immediate,
            operand2: None,
        };

        let asm = Assembly {
            address,
            size: 2,
            code,
            instruction,
        };

        Some(asm)
    }

    fn sub_immediate(&mut self) -> Option<Assembly> {
        let address = self.ip;

        let opcode = self.consume_u8()?;
        let value = self.consume_u8()?;

        let (mod_, _, rm) = bin::get_reg_mem_element(value);

        let sw = opcode & 0b11;
        if sw == 0b11 {
            if mod_ == 0b00 && rm == 0b110 {
                let mut disp = self.consume_u16()?;

                disp = reverse_order_u16(disp)?;

                let ea = EA::DispOnly(Disp(disp as isize));
                let immediate_value = self.consume_u8()? as i8;

                let instruction = Instruction {
                    opcode: Opcode::SubImmediateRegisterMemory,
                    operand1: Some(Operand::EffectiveAddress(ea)),
                    operand2: Some(Operand::Immediate(I8(immediate_value as i8, 2))),
                };

                let code: usize = (opcode as usize) << 32
                    | (value as usize) << 24
                    | (disp as usize) << 16
                    | immediate_value as usize;

                let asm = Assembly {
                    address,
                    size: 5,
                    code,
                    instruction,
                };

                return Some(asm);
            }
        }

        None
    }

    fn immediate_register_memory(&mut self, mut opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        let mut size = 0;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;
        size += 2;

        let mut code = (cur_byte as usize) << 8 | (next_byte as usize);

        let (mod_, _, rm) = bin::get_reg_mem_element(next_byte);

        let sw = cur_byte & 0b11;
        let s = cur_byte >> 1 & 0b1;
        let w = cur_byte & 0b1;

        let mut register: Register = Register::None;
        let mut value: u16 = 0;

        // w == 1 then 16bit length
        // if sw == 0b01 {
        if mod_ == 0b00 && rm == 0b110 {
            let third_byte = self.consume_u8()?;
            size += 1;
            let mut disp = self.consume_u8()?;
            size += 1;

            code = (code as usize) << 8 | third_byte as usize;
            code = (code as usize) << 8 | disp as usize;
            // println!("**** {:04x} {:0x}", address, code);
            // println!("bin: {:04x} {:064b}", address, code);

            let data = (third_byte as u16) << 8 | (disp as u16);

            let reversed_disp = reverse_order_u16(data)?;

            let ea = EA::DispOnly(Disp(reversed_disp as isize));

            let immediate_value_first = self.consume_u8()?;
            size += 1;
            // println!("bin: {:04x} {:064b}", address, code<<8);
            // println!("bin: {:04x} {:064b}", address, immediate_value_first as usize);
            code = code << 8 | immediate_value_first as usize;
            // println!("**** {:04x} {:0x}", address, code);
            // println!("bin: {:04x} {:064b}", address, code);

            let operand2 = if w == 1 && opcode == Opcode::MovImmediateRegisterMemory {
                let immediate_value_second = self.consume_u8()? as i8;
                let immediate_value: i16 =
                    (immediate_value_second as i16) << 8 | immediate_value_first as i16;

                // println!("**** {:04x} {:0x}", address, code);
                // println!("bin: {:04x} {:064b}", address, code);
                size += 1;
                code = (code as usize) << 8 | immediate_value_second as usize;

                Some(Operand::Immediate(I16(immediate_value, 4)))
            } else {
                Some(Operand::Immediate(I8(immediate_value_first as i8, 2)))
            };

            let instruction = Instruction {
                opcode,
                operand1: Some(Operand::EffectiveAddress(ea)),
                operand2,
            };
            // println!("**** {:04x} {:0x}", address, code);

            let asm = Assembly {
                address,
                size,
                code,
                instruction,
            };

            return Some(asm);
        }

        // 0b10追加する。
        let main_operand = match mod_ {
            0b00 => {
                let ea = EA::new(rm, 0);
                Some(Operand::EffectiveAddress(ea))
            }
            0b01 => {
                if opcode == Opcode::TestImmediate {
                    opcode = Opcode::TestImmediateByte;
                }
                let third_byte = self.consume_u8()?;
                code = code << 8 | third_byte as usize;
                size += 1;

                let value = third_byte as i8;
                let value: i16 = -(!(value as i16) + 1);

                let ea = EA::new(rm, value as isize);
                Some(Operand::EffectiveAddress(ea))
            }
            // このとき，r/mはREG fieldとして扱われる
            0b10 => {
                let mut disp = self.consume_u16()?;

                code = code << 16 | disp as usize;
                size += 2;
                disp = reverse_order_u16(disp)?;
                let reg = Register::gen(rm, w);
                let ea = EA::new(rm, (disp as i16) as isize);

                Some(Operand::EffectiveAddress(ea))
            }
            // このとき，r/mはREG fieldとして扱われる
            0b11 => {
                let reg = Register::gen(rm, w);
                if opcode == Opcode::CmpImmediateByte {
                    opcode = Opcode::CmpImmediateWord;
                }
                Some(Operand::Register(reg))
            }
            _ => None,
        };

        // 基本的にはsw値を持つが，mov のときは wだけである
        let immediate_value = match (&opcode, sw) {
            (Opcode::MovImmediateRegisterMemory, 0b11)
            | (Opcode::TestImmediate, 0b11)
            | (Opcode::TestImmediateByte, 0b11)
            | (_, 0b01) => {
                if opcode == Opcode::TestImmediateByte {
                    opcode = Opcode::TestImmediate;
                }

                let third_byte = self.consume_u8()?;
                let forth_byte = self.consume_u8()?;
                size += 2;

                let value = ((forth_byte as i16) << 8) | third_byte as i16; // 即値はリトルエンディアンなので入れ替える
                                                                            // println!("{:4x} {:0x}", address, code);

                code = (code << 8) | third_byte as usize;
                code = (code << 8) | forth_byte as usize;

                ImmediateValue::I16(value, 4)
            }
            (_, 0b11) => {
                let new_byte = self.consume_u8()?;
                size += 1;

                let disp_low: i8 = new_byte as i8;
                let signed_disp_low: i16 = disp_low as i16;
                // let disp_low: i16 = -(!(disp_low as i16) + 1);

                // println!("--- {:04x} {:0x}", address, code);

                code = (code << 8) | new_byte as usize;

                // add immediate register memoryとか
                ImmediateValue::I16(signed_disp_low, 1)
            }
            _ => {
                let third_byte = self.consume_u8()?;
                size += 1;
                code = (code << 8) | third_byte as usize;
                // println!("==== {:04x} {:0x}", address, code);

                ImmediateValue::I8(third_byte as i8, 2)
            }
        };

        // "0b07: ffffffffffff9015  mov [0140], ff90"
        let sub_operand = Some(Operand::Immediate(immediate_value));

        let instruction = Instruction {
            opcode,
            operand1: main_operand,
            operand2: sub_operand,
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }

    fn undefined(&mut self) -> Option<Assembly> {
        let address = self.ip;

        let code = self.consume_u8()? as usize;

        let instruction = Instruction {
            opcode: Opcode::Undefined,
            operand1: None,
            operand2: None,
        };

        let asm = Assembly {
            address,
            size: 1,
            code,
            instruction,
        };

        Some(asm)
    }

    fn reg_either(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        // println!("{:0x}", address);

        let mut size = 0;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;
        size += 2;

        let mut code = ((cur_byte as u16) << 8 | next_byte as u16) as usize;

        let (d, w) = bin::get_dw(cur_byte);
        let (mod_, reg, rm) = bin::get_reg_mem_element(next_byte);

        let register = Register::gen(reg, w);
        let main_operand = Some(Operand::Register(register));

        let sub_operand = match mod_ {
            0b00 => {
                let ea = if rm == 0b110 {
                    let mut disp = self.consume_u16()?;
                    code = code << 16 | disp as usize;

                    size += 2;
                    disp = reverse_order_u16(disp)?;

                    EA::DispOnly(Disp(disp as isize))
                } else {
                    EA::new(rm, 0)
                };

                Some(Operand::EffectiveAddress(ea))
            }
            0b01 => {
                // DISP = disp-low sign-extended to 16 bits,
                // disp-high is absent
                let third_byte = self.consume_u8()?;
                let disp_low = third_byte as i8;
                let disp_low_extend = disp_low as i16; // 符号付き16bit長にする
                let disp_low_comp = -(!disp_low_extend + 1); // 補数を取得

                size += 1;

                let ea = match rm {
                    0b110 | 0b101 | 0b011 | 0b100 | 0b111 => {
                        code = code << 8 | third_byte as usize;
                        EA::new(rm, disp_low_comp as isize)
                    }
                    _ => EA::new(rm, 0),
                };

                Some(Operand::EffectiveAddress(ea))
            }
            0b10 => {
                let mut disp = self.consume_u16()?;

                code = code << 16 | disp as usize;
                size += 2;
                disp = reverse_order_u16(disp)?;

                let reg = Register::gen(rm, w);
                let ea = EA::new(rm, (disp as i16) as isize);

                Some(Operand::EffectiveAddress(ea))
            }
            0b11 => {
                let reg = Register::gen(rm, w);
                Some(Operand::Register(reg))
            }
            _ => None,
        };

        let instruction = if d == 1 {
            Instruction {
                opcode,
                operand1: main_operand,
                operand2: sub_operand,
            }
        } else {
            Instruction {
                opcode,
                operand1: sub_operand,
                operand2: main_operand,
            }
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }

    fn load(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        let mut size = 0;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;
        let mut code = ((cur_byte as usize) << 8 | next_byte as usize) as usize;
        size += 2;

        let (_, w) = bin::get_dw(cur_byte);
        let (mod_, reg, rm) = bin::get_reg_mem_element(next_byte);

        let register = Register::gen(reg, w);
        let operand1 = Some(Operand::Register(register));

        let operand2 = match mod_ {
            0b00 => {
                let ea = EA::DispOnly(Disp(0));
                Some(Operand::EffectiveAddress(ea))
            }
            0b01 => {
                let byte = self.consume_u8()?;
                code = code << 8 | byte as usize;
                size += 1;

                let disp = byte as i8;
                let ea = EA::new(rm, disp as isize);

                Some(Operand::EffectiveAddress(ea))
            }
            0b10 => {
                let disp_low = self.consume_u8()?;
                let disp_high = self.consume_u8()?;
                size += 2;

                code = code << 8 | disp_low as usize;
                code = code << 8 | disp_high as usize;

                let disp = (disp_high as i16) << 8 | disp_low as i16;

                let ea = EA::new(rm, disp as isize);
                Some(Operand::EffectiveAddress(ea))
            }
            0b11 => {
                let reg = Register::gen(rm, w);
                Some(Operand::Register(reg))
            }
            _ => None,
        };

        let instruction = Instruction {
            opcode,
            operand1,
            operand2,
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }

    fn jmp_disp(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;
        let code = ((cur_byte as usize) << 8 | next_byte as usize) as usize;

        // 現在のアドレス + disp
        let disp = self.ip as i16 + (next_byte as i8) as i16;

        let instruction = Instruction {
            opcode,
            operand1: Some(Operand::Immediate(I16(disp, 4))),
            operand2: None,
        };

        let asm = Assembly {
            address,
            size: 2,
            code,
            instruction,
        };

        Some(asm)
    }

    fn register_memory(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;
        let mut size = 2;
        let mut code = ((cur_byte as usize) << 8 | next_byte as usize) as usize;

        let (v, w) = bin::get_dw(cur_byte);
        let (mod_, reg_, rm) = bin::get_reg_mem_element(next_byte);

        let operand1 = match mod_ {
            0b00 => {
                let ea = if rm == 0b110 {
                    let mut disp = self.consume_u16()?;
                    code = code << 16 | disp as usize;

                    size += 2;
                    disp = reverse_order_u16(disp)?;

                    EA::DispOnly(Disp(disp as isize))
                } else {
                    EA::new(rm, 0)
                };

                Some(Operand::EffectiveAddress(ea))
            }
            0b01 => {
                let third_byte = self.consume_u8()?;
                let val = third_byte as i8;

                size += 1;
                code = (code << 8) | third_byte as usize;

                let ea = EA::new(rm, val as isize);
                Some(Operand::EffectiveAddress(ea))
            }
            0b10 => {
                let mut disp = self.consume_u16()?;

                code = code << 16 | disp as usize;
                size += 2;
                disp = reverse_order_u16(disp)?;

                let reg = Register::gen(rm, w);
                let ea = EA::new(rm, (disp as i16) as isize);

                Some(Operand::EffectiveAddress(ea))
            }
            0b11 => {
                let reg = Register::gen(rm, w);
                Some(Operand::Register(reg))
            }
            _ => None,
        };

        let operand2: Option<Operand> = match opcode {
            Opcode::Shl
            | Opcode::Shr
            | Opcode::Sar
            | Opcode::Rol
            | Opcode::Ror
            | Opcode::Rcl
            | Opcode::Rcr => {
                if v == 1 {
                    Some(Operand::Register(Register::Reg8(Reg8::CL)))
                } else {
                    Some(Operand::Immediate(ImmediateValue::I8(1, 2)))
                }
            }
            Opcode::TestRegisterMemoryAndRegister | Opcode::XchgRegisterMemoryWithRegister => {
                let reg = Register::gen(reg_, w);
                Some(Operand::Register(reg))
            }
            _ => None,
        };

        let instruction = Instruction {
            opcode,
            operand1,
            operand2,
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }

    fn reg_series(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;

        let cur_byte = self.consume_u8()?;
        let code = cur_byte as usize;
        let reg = cur_byte & 0b111;

        let operand1 = Some(Operand::Register(Register::gen(reg, 1)));
        let operand2: Option<Operand> = match opcode {
            Opcode::PushRegMem => {
                let third_byte = self.consume_u8()?;
                let ea = EA::new(reg, third_byte as isize);

                Some(Operand::EffectiveAddress(ea))
            }
            Opcode::XchgRegisterWithAccumulator => {
                Some(Operand::Register(Register::Reg16(Reg16::AX)))
            }
            _ => None,
        };

        let instruction = Instruction {
            opcode,
            operand1,
            operand2,
        };

        let asm = Assembly {
            address,
            size: 1,
            code,
            instruction,
        };

        Some(asm)
    }

    fn direct_seg(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        let cur_byte = self.consume_u8()?;

        let disp = self.consume_u16()?;
        let rev_disp = reverse_order_u16(disp)?;

        let code = ((cur_byte as usize) << 16 | disp as usize) as usize;
        let target_addr = self.ip as isize + rev_disp as isize;

        let instruction = Instruction {
            opcode,
            operand1: Some(Operand::Immediate(ImmediateValue::I16(
                target_addr as i16,
                4,
            ))),
            operand2: None,
        };

        let asm = Assembly {
            address,
            size: 3,
            code,
            instruction,
        };

        Some(asm)
    }

    fn memory_to_register(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        let cur_byte = self.consume_u8()?;

        let disp = self.consume_u16()?;
        let rev_disp = reverse_order_u16(disp)?;

        let code = ((cur_byte as usize) << 16 | disp as usize) as usize;
        let target_addr = rev_disp as isize;

        let instruction = Instruction {
            opcode,
            operand1: Some(Operand::Register(Register::Reg16(Reg16::AX))),
            operand2: Some(Operand::EffectiveAddress(EA::DispOnly(Disp(
                target_addr as isize,
            )))),
        };

        let asm = Assembly {
            address,
            size: 3,
            code,
            instruction,
        };

        Some(asm)
    }

    fn immed_to_sp(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        let cur_byte = self.consume_u8()?;

        let data = self.consume_u16()?;
        let rev_data = reverse_order_u16(data)?;

        let code = ((cur_byte as usize) << 16 | data as usize) as usize;

        let instruction = Instruction {
            opcode,
            operand1: Some(Operand::Immediate(ImmediateValue::I16(rev_data as i16, 4))),
            operand2: None,
        };

        let asm = Assembly {
            address,
            size: 3,
            code,
            instruction,
        };

        Some(asm)
    }

    fn proc_control(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;

        let cur_byte = self.consume_u8()?;
        let code = cur_byte as usize;

        let instruction = Instruction {
            opcode,
            operand1: None,
            operand2: None,
        };

        let asm = Assembly {
            address,
            size: 1,
            code,
            instruction,
        };

        Some(asm)
    }

    fn direct_seg_short(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;
        let code = ((cur_byte as usize) << 8 | next_byte as usize) as usize;

        let disp_low: i8 = next_byte as i8;
        let disp_low: i16 = -(!(disp_low as i16) + 1);
        let jmp_target = self.ip as i16 + disp_low;
        let disp = Some(Operand::Immediate(ImmediateValue::I16(
            jmp_target as i16,
            4,
        )));

        let instruction = Instruction {
            opcode,
            operand1: disp,
            operand2: None,
        };

        let asm = Assembly {
            address,
            size: 2,
            code,
            instruction,
        };

        Some(asm)
    }

    fn fixed_port(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;

        let cur_byte = self.consume_u8()?;

        let mut code = cur_byte as usize;
        let mut size = 1;

        let (_v, w) = bin::get_dw(cur_byte);

        let operand1 = Some(if w == 1 {
            Operand::Register(Register::Reg16(Reg16::AX))
        } else {
            Operand::Register(Register::Reg8(Reg8::AL))
        });

        let operand2 = match opcode {
            Opcode::InFixedPort | Opcode::OutFixedPort => {
                let next_byte = self.consume_u8()?;
                code = ((code as usize) << 8 | next_byte as usize) as usize;
                size += 1;
                Some(Operand::Immediate(ImmediateValue::I8(next_byte as i8, 2)))
            }
            Opcode::InVariablePort | Opcode::OutVariablePort => {
                Some(Operand::Register(Register::Reg16(Reg16::DX)))
            }
            _ => None,
        };

        let instruction = Instruction {
            opcode,
            operand1,
            operand2,
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }

    fn variable_port(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;

        let size = 2;
        let code = ((cur_byte as usize) << 8 | next_byte as usize) as usize;

        let (_v, w) = bin::get_dw(cur_byte);

        let operand1 = Some(if w == 1 {
            Operand::Register(Register::Reg16(Reg16::AX))
        } else {
            Operand::Register(Register::Reg8(Reg8::AL))
        });

        let operand2 = Some(Operand::Immediate(ImmediateValue::I8(next_byte as i8, 2)));

        let instruction = Instruction {
            opcode,
            operand1,
            operand2,
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }

    fn string_manipulation(&mut self, mut opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.peek_u8()?;

        let mut size = 1;
        let mut code = cur_byte as usize;

        let (_v, _) = bin::get_dw(cur_byte);

        opcode = match next_byte {
            0xa5 => Opcode::RepMovsw,
            0xa4 => Opcode::RepMovsb,
            0xaa => Opcode::RepStosb,
            0xae => Opcode::RepScasb,
            _ => Opcode::CompsByte,
        };

        if Opcode::CompsByte != opcode {
            self.consume_u8()?;
            size += 1;
            code = ((cur_byte as usize) << 8 | next_byte as usize) as usize;
        }

        let instruction = Instruction {
            opcode,
            operand1: None,
            operand2: None,
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }

    fn immediate_accumulator(&mut self, opcode: Opcode) -> Option<Assembly> {
        let address = self.ip;
        let mut size = 0;

        let cur_byte = self.consume_u8()?;
        let next_byte = self.consume_u8()?;
        size += 2;

        let mut code = (cur_byte as usize) << 8 | (next_byte as usize);

        let w = cur_byte & 0b1;

        let immediate_value = if w == 0b1 {
            let third_byte = self.consume_u8()?;
            size += 1;

            let value = ((third_byte as i16) << 8) | next_byte as i16; // 即値はリトルエンディアンなので入れ替える
            code = (code << 8) | third_byte as usize;

            ImmediateValue::I16(value, 4)
        } else {
            ImmediateValue::I8(next_byte as i8, 2)
        };

        let main_operand = Some(Operand::Register(if w == 1 {
            Register::Reg16(Reg16::AX)
        } else {
            Register::Reg8(Reg8::AL)
        }));

        let sub_operand = Some(Operand::Immediate(immediate_value));

        let instruction = Instruction {
            opcode,
            operand1: main_operand,
            operand2: sub_operand,
        };

        let asm = Assembly {
            address,
            size,
            code,
            instruction,
        };

        Some(asm)
    }
}

impl BinaryConsume for VM {
    fn consume_u8(&mut self) -> Option<u8> {
        // if self.ip >= self.text_size as u16 {
        //     return None
        // }

        let out = self.ram.read_text(self.ip);
        self.ip += 1;

        Some(out)
    }

    fn consume_u16(&mut self) -> Option<u16> {
        Some((self.consume_u8()? as u16) << 8 | self.consume_u8()? as u16)
    }

    fn consume_u32(&mut self) -> Option<u32> {
        let target = (self.consume_u16()? as u32) << 16 | self.consume_u16()? as u32;

        #[cfg(feature = "big_endian")]
        let target = bin::reverse_order_u32(target)?;

        Some(target)
    }
}

impl BinaryPeek for VM {
    fn peek_u8(&mut self) -> Option<u8> {
        let out = self.ram.read_text(self.ip);
        Some(out)
    }

    fn peek_u16(&mut self) -> Option<u16> {
        Some((self.peek_u8()? as u16) << 8 | self.peek_u8()? as u16)
    }

    fn peek_u32(&mut self) -> Option<u32> {
        let target = (self.peek_u16()? as u32) << 16 | self.peek_u16()? as u32;

        #[cfg(feature = "big_endian")]
        let target = bin::reverse_order_u32(target)?;

        Some(target)
    }

    fn peek_offset(&mut self, offset: usize) -> Option<u8> {
        let out = self.ram.read_text(self.ip + offset as u16);

        Some(out)
    }
}
