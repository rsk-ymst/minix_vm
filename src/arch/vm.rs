use libc::O_RDWR;

use crate::VM_MODE;

use super::asm::Assembly;
use super::bin::{bytes_to_16bit_little_endian, BinaryManager};
use super::constant::syscall::{BRK, CLOSE, EXIT, IOCTL, LSEEK, OPEN, READ, WRITE};
use super::{
    opcode::Opcode,
    operand::{Operand, EA},
    reg::{Reg16, Reg8, Register},
};
use std::ffi::{c_int, CString};
use std::fmt::Debug;
use std::os::raw::c_void;
use std::path::Path;
use std::process::exit;

const DATA2_SIZE: usize = 20;

#[derive(Debug)]
pub struct VM {
    pub(crate) reg: [u16; 8], // レジスタの値．添字でアクセスする
    flags: u16,
    pub(crate) ip: u16,
    pub(crate) ram: Ram,
    pub text_size: usize,
}

#[derive(Debug)]
pub struct Ram {
    cells: [u8; 0xfffff],
    pub text_base: u16,
    pub data_base: u16,
}

impl Ram {
    pub fn new(cells: [u8; 0xfffff], text_base: u16, data_base: u16) -> Self {
        Ram {
            cells,
            text_base,
            data_base,
        }
    }

    pub unsafe fn get_real_address(&self, addr: u16) -> *mut c_void {
        self.cells.as_ptr().offset((self.data_base + addr) as isize) as *mut c_void
    }

    pub fn read_i8(&self, addr: u16) -> i8 {
        let base = (self.data_base as usize + addr as usize) as usize;
        self.cells[base] as i8
    }

    pub fn read_i16(&self, addr: u16) -> i16 {
        let base = (self.data_base as usize + addr as usize) as usize;
        bytes_to_16bit_little_endian(&self.cells[base..=(base + 1)]) as i16
    }

    pub fn read_text(&self, offset: u16) -> u8 {
        self.cells[self.text_base as usize + offset as usize]
    }

    pub fn read_data_size(&self, offset: usize) -> &[u8] {
        let base = self.data_base as usize + offset;
        &self.cells[base..base + DATA2_SIZE]
    }

    pub fn read_data_slice(&self, base: u16, offset: u16) -> &[u8] {
        let data_base = self.data_base as usize + base as usize;

        &self.cells[data_base..data_base + offset as usize]
    }

    pub fn write_i8(&mut self, addr: u16, value: i8) {
        let base = (self.data_base as usize + addr as usize) as usize;
        self.cells[base as usize] = value as u8;
    }

    pub fn direct_write_i8(&mut self, addr: u16, value: i8) {
        self.cells[addr as usize] = value as u8;
    }

    pub fn write_i16(&mut self, addr: u16, value: i16) {
        let base = (self.data_base as usize + addr as usize) as usize;

        /* エンディアンの相違を考慮しながら配置 */
        self.cells[base] = (value & 0xff) as u8;
        self.cells[base + 1] = (value >> 8 & 0xff) as u8;
    }

    pub fn direct_write_i16(&mut self, addr: u16, value: i16) {
        /* エンディアンの相違を考慮しながら配置 */
        self.cells[addr as usize] = (value & 0xff) as u8;
        self.cells[addr as usize + 1] = (value >> 8 & 0xff) as u8;
    }

    fn get_string(&self, addr: u16) -> String {
        let mut s = String::new();
        let mut i = addr;

        loop {
            let c = self.read_i8(i);
            if c == 0 {
                break;
            }

            s.push(c as u8 as char);
            i += 1;
        }

        s
    }
}

trait ManageFlags {
    fn evaluate_overflow(&mut self, dst: u16, result: u16);
    fn evaluate_cmp(&mut self, val: u16, dst: u16, src: u16);
    fn get_flag(&self, flag: Flag) -> bool;
    fn set_flag(&mut self, flag: Flag, val: bool);
    fn evaluate_calculation_result(&mut self, result: u16);
    fn print_flags(&self);
}

enum Flag {
    CF = 0, // Carry Flag
    PF = 2,
    AF = 4,
    ZF = 6, // Zero Flag
    SF = 7,
    TF = 8,
    IF = 9,
    DF = 10,
    OF = 11,
}

impl ManageFlags for VM {
    fn get_flag(&self, flag: Flag) -> bool {
        self.flags >> (flag as usize) & 1 == 1
    }

    fn set_flag(&mut self, flag: Flag, val: bool) {
        let pos = flag as usize;
        if val {
            self.flags |= 1 << pos;
        } else {
            self.flags &= !(1 << pos);
        }
    }

    fn evaluate_calculation_result(&mut self, val: u16) {
        self.set_flag(Flag::ZF, val == 0);
        self.set_flag(Flag::SF, val >> 15 & 1 == 1);
    }

    fn evaluate_overflow(&mut self, dst: u16, result: u16) {
        self.set_flag(Flag::OF, (result >> 15 & 1) != (dst >> 15 & 1));
    }

    fn evaluate_cmp(&mut self, result: u16, dst: u16, src: u16) {
        self.set_flag(Flag::CF, (result >> 15 & 1) != (dst >> 15 & 1));
    }

    fn print_flags(&self) {
        let of_char = if self.get_flag(Flag::OF) { 'O' } else { '-' }; // zero flag
        let sf_char = if self.get_flag(Flag::SF) { 'S' } else { '-' }; // sign flag
        let zf_char = if self.get_flag(Flag::ZF) { 'Z' } else { '-' }; // zero flag
        let cf_char = if self.get_flag(Flag::CF) { 'C' } else { '-' }; // cmp flag

        if unsafe { VM_MODE } {
            print!("{}{}{}{}", of_char, sf_char, zf_char, cf_char);
        }
    }
}

impl VM {
    pub fn new(mut bytes: Vec<u8>, args: &[String]) -> Self {
        let mut bm = BinaryManager::new(bytes.clone());

        let header_size = bm.get_header_size();
        let text_size = bm.get_text_size();
        let data_size = bm.get_data_size();
        let bss_size = bm.get_bss_size();

        let data_base_ptr = header_size + text_size;
        let bss_base_ptr = data_base_ptr + data_size;

        let text = &bytes[header_size..header_size + text_size];
        let data = &bytes[data_base_ptr..data_base_ptr + data_size];

        let mut cells: [u8; 0xfffff] = [0u8; 0xfffff];

        cells[..text_size].copy_from_slice(text);
        cells[text_size..text_size + data_size].copy_from_slice(data);

        let ram = Ram::new(cells, 0, text_size as u16);

        let mut vm = Self {
            reg: [0; 8],
            ip: 0,
            flags: 0,
            ram,
            text_size,
        };

        vm.init(args);

        vm
    }

    pub fn init(&mut self, args: &[String]) {
        self.make_stack_frame(args);
    }

    pub fn make_stack_frame(&mut self, args: &[String]) {
        let cstr_byte = "PATH=/usr:/usr/bin".as_bytes();
        let mut args = args.to_owned();

        if args.len() != 0 {
            args[0] = Path::new(args.get(0).unwrap())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
        }

        self.reg[Reg16::SP as usize] = 0xfffe;

        /* env content in */
        for i in (0..cstr_byte.len()).rev() {
            self.ram
                .write_i8(self.get_reg16(Reg16::SP), cstr_byte[i] as i8);
            self.reg[Reg16::SP as usize] -= 1;
        }
        let env_ptr = self.get_reg16(Reg16::SP) + 1;

        let mut argsv = Vec::new();
        for i in (0..args.len()).rev() {
            self.ram.write_i8(self.get_reg16(Reg16::SP), '\0' as i8);
            self.reg[Reg16::SP as usize] -= 1;

            for j in (0..args[i].len()).rev() {
                self.ram
                    .write_i8(self.get_reg16(Reg16::SP), args[i].as_bytes()[j] as i8);
                self.reg[Reg16::SP as usize] -= 1;
            }

            argsv.push(self.get_reg16(Reg16::SP) + 1);
        }

        self.ram.write_i8(self.reg[Reg16::SP as usize], '\0' as i8);
        self.reg[Reg16::SP as usize] -= 1;
        self.ram.write_i8(self.reg[Reg16::SP as usize], '\0' as i8);

        self.reg[Reg16::SP as usize] -= 2;

        self.ram
            .write_i16(self.get_reg16(Reg16::SP), env_ptr as i16);
        self.reg[Reg16::SP as usize] -= 2;

        self.ram.write_i8(self.reg[Reg16::SP as usize], '\0' as i8);
        self.reg[Reg16::SP as usize] -= 1;
        self.ram.write_i8(self.reg[Reg16::SP as usize], '\0' as i8);
        self.reg[Reg16::SP as usize] -= 1;

        for i in 0..argsv.len() {
            self.ram
                .write_i16(self.get_reg16(Reg16::SP), argsv[i] as i16);
            self.reg[Reg16::SP as usize] -= 2;
        }

        self.ram
            .write_i16(self.get_reg16(Reg16::SP), args.len() as u16 as i16);
    }

    // 一連の処理の実行
    pub fn run(&mut self) {
        self.print_status_header();

        loop {
            /* fetch & decode */
            let inst = self.decode().unwrap();
            self.print_current_status(&inst);

            /* decode */
            let (opcode, dst_val, src_val) = self.fetch_operand_value(&inst);

            /* execute */
            let result = self.execute(opcode, dst_val, src_val);

            /* evaluate */
            self.evaluate(&opcode, result, dst_val, src_val);

            /* store */
            if opcode.is_assign_effect() {
                self.store(&inst, result);
            }

            if unsafe { VM_MODE } {
                /* 改行 */
                println!();
            }
        }
    }

    fn evaluate(&mut self, opcode: &Opcode, result: u16, dst_val: u16, src_val: u16) {
        /* evaluate */
        if opcode.is_calculated() {
            self.evaluate_calculation_result(result);
        }

        if opcode.could_be_carried() {
            self.evaluate_cmp(result, dst_val, src_val);
        }

        if opcode.could_be_over_flow() {
            self.set_flag(Flag::OF, result >> 15 & 1 != dst_val >> 15 & 1);
        }
    }

    pub fn disassemble(&mut self) -> Vec<Assembly> {
        let mut v = Vec::new();
        loop {
            if let Some(asm) = self.decode() {
                v.push(asm.clone());

                if asm.instruction.opcode == Opcode::Undefined {
                    break;
                }
            } else {
                break;
            }
        }
        v
    }

    fn store(&mut self, asm: &Assembly, value: u16) {
        match asm.instruction.opcode {
            Opcode::XchgRegisterMemoryWithRegister | Opcode::XchgRegisterWithAccumulator => {
                let dst_val = self.get_val_from_operand(
                    asm.instruction.operand1.as_ref(),
                    asm.instruction.operand2.as_ref(),
                    &asm.instruction.opcode,
                );

                let src_val = self.get_val_from_operand(
                    asm.instruction.operand2.as_ref(),
                    asm.instruction.operand1.as_ref(),
                    &asm.instruction.opcode,
                );

                self.set_val_from_operand(
                    asm.instruction.operand1.as_ref(),
                    asm.instruction.operand2.as_ref(),
                    &asm.instruction.opcode,
                    src_val,
                );

                self.set_val_from_operand(
                    asm.instruction.operand2.as_ref(),
                    asm.instruction.operand1.as_ref(),
                    &asm.instruction.opcode,
                    dst_val,
                );

                return;
            }
            Opcode::MovImmediateRegisterMemory
            | Opcode::MovImmediateRegisterMemoryWord
            | Opcode::MovImmediateRegisterMemoryByte
            | Opcode::MovMemoryToAccumulator
            | Opcode::MovImmediate => {
                self.set_val_from_operand(
                    asm.instruction.operand1.as_ref(),
                    asm.instruction.operand2.as_ref(),
                    &asm.instruction.opcode,
                    value,
                );
            }
            _ => (),
        }

        self.set_val_from_operand(
            asm.instruction.operand1.as_ref(),
            asm.instruction.operand2.as_ref(),
            &asm.instruction.opcode,
            value,
        );
    }

    fn jmp(&mut self, addr: u16) {
        self.ip = addr;
    }

    fn fetch_operand_value(&self, asm: &Assembly) -> (Opcode, u16, u16) {
        let dst_val = self.get_val_from_operand(
            asm.instruction.operand1.as_ref(),
            asm.instruction.operand2.as_ref(),
            &asm.instruction.opcode,
        );
        let src_val = self.get_val_from_operand(
            asm.instruction.operand2.as_ref(),
            asm.instruction.operand1.as_ref(),
            &asm.instruction.opcode,
        );

        (asm.instruction.opcode.clone(), dst_val, src_val)
    }

    fn push(&mut self, value: u16) {
        let new_sp = self.get_reg16(Reg16::SP) - 2;
        self.ram.write_i16(new_sp, value as i16);
        self.set_reg16(Reg16::SP, new_sp);
    }

    fn pop(&mut self) -> u16 {
        let sp = self.get_reg16(Reg16::SP);
        let ret = self.ram.read_i16(sp);

        self.set_reg16(Reg16::SP, sp + 2);

        ret as u16
    }

    fn get_val_from_operand(
        &self,
        target_operand: Option<&Operand>,
        pair_operand: Option<&Operand>,
        opcode: &Opcode,
    ) -> u16 {
        match target_operand {
            Some(Operand::Register(reg)) => {
                /* exception case */
                match opcode {
                    Opcode::PopReg | Opcode::PopRegMem => {
                        match reg {
                            Register::Reg16(r16) => return *r16 as u16,
                            Register::Reg8(_) => todo!(),
                            Register::None => todo!(),
                            // Reg16
                        }
                    }
                    _ => (),
                }

                self.get_val_from_reg(reg)
            }
            Some(Operand::Immediate(val)) => Into::<u16>::into(*val),
            Some(Operand::EffectiveAddress(ea)) => {
                let addr = self.get_val_from_ea(ea);

                let val = self.ram.read_i16(addr) as u16;

                if unsafe { VM_MODE } {
                    print!(" ;[{addr:04x}]");
                }

                if display_byte_flag(pair_operand, &opcode) {
                    if unsafe { VM_MODE } {
                        print!("{:02x}", val & 0xff);
                    }
                } else {
                    if unsafe { VM_MODE } {
                        print!("{val:04x}");
                    }
                }

                /* exception case */
                match opcode {
                    Opcode::Lea => {
                        return addr;
                    }
                    _ => (),
                }

                val
            }
            _ => 0,
        }
    }

    fn set_val_from_operand(
        &mut self,
        dst_operand: Option<&Operand>,
        src_operand: Option<&Operand>,
        opcode: &Opcode,
        mut value: u16,
    ) {
        match dst_operand {
            Some(Operand::Register(reg)) => match reg {
                Register::Reg16(r16) => {
                    self.set_reg16(*r16, value);
                }
                Register::Reg8(r8) => {
                    self.set_reg8(*r8, value as i8);
                }
                Register::None => todo!(),
            },
            Some(Operand::Immediate(val)) => {}
            Some(Operand::EffectiveAddress(ea)) => {
                let addr = self.get_val_from_ea(ea);

                if let Some(Operand::Register(Register::Reg8(r8))) = src_operand {
                    self.ram.write_i8(addr, value as i8);
                    return;
                }

                if *opcode == Opcode::MovImmediateRegisterMemoryByte {
                    value = value & 0xff;
                    self.ram.write_i8(addr, value as i8);
                    return;
                }

                self.ram.write_i16(addr, value as i16);
            }
            _ => (),
        }
    }

    fn get_val_from_reg(&self, reg: &Register) -> u16 {
        match reg {
            Register::Reg16(r16) => self.get_reg16(*r16),
            Register::Reg8(r8) => self.get_reg8(*r8) as u16,
            Register::None => 8,
        }
    }

    fn execute(&mut self, opcode: Opcode, dst: u16, src: u16) -> u16 {
        match opcode {
            Opcode::MovImmediateRegisterMemory
            | Opcode::MovImmediateRegisterMemoryWord
            | Opcode::MovImmediateRegisterMemoryByte
            | Opcode::MovMemoryToAccumulator
            | Opcode::MovImmediate => src,
            Opcode::PushRegMem => {
                self.push(dst);
                0
            }
            Opcode::PushReg => {
                self.push(dst);
                0
            }
            Opcode::PushSegReg => todo!(),
            Opcode::PopRegMem => todo!(),
            Opcode::PopReg => {
                let ret = self.pop();
                self.set_reg16(Reg16::from(dst as u8), ret);
                ret
            }
            Opcode::PopSegReg => todo!(),
            Opcode::XchgRegisterMemoryWithRegister | Opcode::XchgRegisterWithAccumulator => {
                // if src < 0 {

                // }
                self.set_flag(Flag::SF, dst >> 15 & 1 == 1);
                src
            }
            Opcode::IntTypeSpecified => {
                if dst == 0x20 {
                    // println!("OK!");
                }

                let bx = self.get_reg16(Reg16::BX);
                self.interrupt_syscall(bx as usize);

                bx
            }
            Opcode::AddRegEither
            | Opcode::AddImmediateRegisterMemory
            | Opcode::AddImmediateToAccumulator
            | Opcode::AddImmediateFromAccumulator => {
                self.set_flag(Flag::CF, (dst as usize + src as usize) > u16::MAX as usize);

                // オペランドが負の数だった場合
                if dst >> 15 & 1 == 1 || src >> 15 & 1 == 1 {
                    let result = dst as i16 + src as i16;
                    self.set_flag(Flag::ZF, result == 0);
                    self.set_flag(Flag::SF, result >> 15 & 1 == 1);
                    return result as u16;
                }

                let result = if dst as usize + src as usize > u16::MAX as usize {
                    let result = dst as usize + src as usize - u16::MAX as usize - 1;
                    result as u16
                } else {
                    dst + src
                };

                self.set_flag(Flag::ZF, result == 0);
                self.set_flag(Flag::SF, result >> 15 & 1 == 1);

                result
            }
            Opcode::OrRegEither
            | Opcode::OrImmediateRegisterMemory
            | Opcode::OrImmediateFromAccumulator => dst | src,
            Opcode::SubRegEither
            | Opcode::SubImmediateRegisterMemory
            | Opcode::SubImmediateFromAccumulator => {
                let result = (dst as i16 - src as i16) as i16;

                self.set_flag(Flag::CF, dst < src);
                self.set_flag(Flag::ZF, result == 0);
                self.set_flag(Flag::SF, result >> 15 & 1 == 1);

                result as u16
            }
            Opcode::AdcRegEither
            | Opcode::AdcImmediateRegisterMemory
            | Opcode::AdcImmediateFromAccumulator => {
                let result = (dst as i16 + src as i16 + self.get_flag(Flag::CF) as i16) as i16;

                self.set_flag(Flag::CF, dst < src && (result as u16 >> 15) != (dst >> 15));
                self.set_flag(Flag::ZF, result == 0);
                self.set_flag(Flag::SF, result >> 15 & 1 == 1);

                result as u16
            }
            Opcode::SsbRegEither => todo!(),
            Opcode::SsbImmediateRegisterMemory => todo!(),
            Opcode::SsbImmediateFromAccumulator => todo!(),
            Opcode::AndRegEither
            | Opcode::AndImmediateRegisterMemory
            | Opcode::AndImmediateFromAccumulator => {
                self.set_flag(Flag::CF, false);
                dst & src
            }
            Opcode::MovRmToFromReg => src,
            Opcode::XorRegEither => {
                self.set_flag(Flag::CF, false);
                dst ^ src
            }

            Opcode::TestRegisterMemoryAndRegister
            | Opcode::TestImmediateDataAndAccumulator
            | Opcode::TestImmediate => {
                let result = dst & src;
                self.set_flag(Flag::SF, result >> 15 == 1);
                self.set_flag(Flag::ZF, result == 0);
                result
            }
            Opcode::TestImmediateByte => (dst & 0xff) & (src & 0xff),

            Opcode::CallWithinDirect => {
                self.push(self.ip as u16);
                self.jmp(dst);
                0
            }
            Opcode::RepScasb => {
                let al_val = self.get_reg8(Reg8::AL);

                loop {
                    let di_val = self.ram.read_i8(self.get_reg16(Reg16::DI));

                    if self.get_flag(Flag::DF) {
                        self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) - 1);
                    } else {
                        self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) + 1);
                    }

                    self.set_reg16(Reg16::CX, self.get_reg16(Reg16::CX) - 1);

                    let cmp_result = (al_val - di_val) as i16;
                    if cmp_result == 0 || self.get_reg16(Reg16::CX) == 0 {
                        self.set_flag(Flag::CF, cmp_result < 0);
                        self.set_flag(Flag::ZF, cmp_result == 0);
                        self.set_flag(Flag::SF, cmp_result >> 15 & 1 == 1);

                        break;
                    }
                }
                return 0;
            }

            Opcode::RepMovsb => loop {
                if self.get_reg16(Reg16::CX) == 0 {
                    break 0;
                }

                let si_val = self.ram.read_i8(self.get_reg16(Reg16::SI));
                self.ram.write_i16(self.get_reg16(Reg16::DI), si_val as i16);

                if self.get_flag(Flag::DF) {
                    self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) - 1);
                    self.set_reg16(Reg16::SI, self.get_reg16(Reg16::SI) - 1);
                } else {
                    self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) + 1);
                    self.set_reg16(Reg16::SI, self.get_reg16(Reg16::SI) + 1);
                }

                self.set_reg16(Reg16::CX, self.get_reg16(Reg16::CX) - 1);
            },

            Opcode::RepStosb => loop {
                if self.get_reg16(Reg16::CX) == 0 {
                    break 0;
                }
                let al_val = self.get_reg8(Reg8::AL);
                self.ram.write_i16(self.get_reg16(Reg16::DI), al_val as i16);

                if self.get_flag(Flag::DF) {
                    self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) - 1);
                } else {
                    self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) + 1);
                }

                self.set_reg16(Reg16::CX, self.get_reg16(Reg16::CX) - 1);

                if self.get_reg16(Reg16::CX) == 0 {
                    break 0;
                }
            },
            // }
            Opcode::CompsByte => {
                let si_val = self.ram.read_i8(self.get_reg16(Reg16::SI));
                let di_val = self.ram.read_i8(self.get_reg16(Reg16::DI));

                let result = (si_val as i16 - di_val as i16) as u16;

                self.set_flag(Flag::CF, si_val < di_val);
                self.set_flag(Flag::SF, result >> 15 & 1 == 1);
                self.set_flag(Flag::ZF, result == 0);

                if self.get_flag(Flag::DF) {
                    self.set_reg16(Reg16::SI, self.get_reg16(Reg16::SI) - 1);
                    self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) - 1);
                } else {
                    self.set_reg16(Reg16::SI, self.get_reg16(Reg16::SI) + 1);
                    self.set_reg16(Reg16::DI, self.get_reg16(Reg16::DI) + 1);
                }

                result
            }
            Opcode::CmpImmediateWord
            | Opcode::CmpImmediateByte
            | Opcode::CmpImmediateFromAccumulator
            | Opcode::CmpRegEither => {
                let mut dst = dst;
                let mut src = src;

                if opcode == Opcode::CmpImmediateByte {
                    dst = dst & 0xff;
                    src = src & 0xff;
                }
                let result = (dst as i16 - src as i16) as u16;

                self.set_flag(Flag::CF, dst < src);
                self.set_flag(Flag::SF, result >> 15 & 1 == 1);
                self.set_flag(Flag::ZF, result == 0);

                result
            }
            Opcode::Lea => src,
            Opcode::Lds => todo!(),
            Opcode::Les => todo!(),
            Opcode::JmpDirectWithinSegment
            | Opcode::JmpDirectWithinSegmentShort
            | Opcode::JmpIndirectWithinSegment => {
                self.jmp(dst);
                0
            }
            Opcode::Shl => {
                // 最上位bitは残ったまま
                // オーバーフローフラグが立つのは，あくまで最上位ビットが0 -> 1になったとき1に変化したとき

                // 最初にシフトされた値がCFに入る ⇒ つまり最下位ビットがCFに入る
                let result = (dst << src) as i16;
                let dst = dst as i16;

                // For each shift count, the most significant bit of the destination operand is shifted into the CF flag
                // shiftして境界を超えるたびに，こえたbitがCFに入る
                let last_into_cf = (dst >> (16 - src)) & 1;
                self.set_flag(Flag::CF, last_into_cf == 1);

                self.set_flag(Flag::OF, (result >> 15) & 1 != last_into_cf);

                result as u16
            }
            Opcode::Shr => dst >> src,
            Opcode::Sar => {
                let is_sign = dst >> 15 == 1;
                let mut result = dst >> src;

                if is_sign {
                    result |= 0xffff << (16 - src - 1);
                }

                let last_into_cf = (dst >> src - 1) & 1;
                self.set_flag(Flag::CF, last_into_cf == 1);

                result
            }
            Opcode::Neg => !dst + 1,
            Opcode::RetWithinSegAddingImmedToSp
            | Opcode::RetIntersegment
            | Opcode::RetIntersegmentAddingImmediateToSp
            | Opcode::RetWithinSegment => {
                let dst_addr = self.pop();
                if dst > 0 {
                    let additional_sp = self.get_reg16(Reg16::SP) + dst;
                    self.set_reg16(Reg16::SP, additional_sp)
                }
                self.jmp(dst_addr);
                0
            }
            Opcode::Je => {
                if self.get_flag(Flag::ZF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jl => {
                if self.get_flag(Flag::SF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jle => {
                if self.get_flag(Flag::SF) || self.get_flag(Flag::ZF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jb => {
                if self.get_flag(Flag::CF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jbe => {
                if self.get_flag(Flag::CF) || self.get_flag(Flag::ZF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jp => {
                if self.get_flag(Flag::PF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jo => {
                if self.get_flag(Flag::OF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Js => {
                if self.get_flag(Flag::SF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jne => {
                if !self.get_flag(Flag::ZF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jnl => {
                // Jump on Not Less/Greater or Equal
                if self.get_flag(Flag::SF) == self.get_flag(Flag::OF) {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jnle => {
                if !self.get_flag(Flag::ZF) && (self.get_flag(Flag::SF) == self.get_flag(Flag::OF))
                {
                    self.jmp(dst);
                }
                0
            }
            Opcode::Jnb => {
                if !self.get_flag(Flag::CF) {
                    self.jmp(dst);
                }
                dst
            }
            Opcode::Jnbe => {
                if !self.get_flag(Flag::CF) && !self.get_flag(Flag::ZF) {
                    self.jmp(dst);
                }
                dst
            }
            Opcode::Cld => {
                self.set_flag(Flag::DF, false);
                0
            }
            Opcode::Cbw => {
                let al = self.get_reg8(Reg8::AL);
                let sign = al >> 7 & 1;
                let ah = if sign == 1 { 0xffu8 as i8 } else { 0 };

                self.set_reg8(Reg8::AH, ah);
                0
            }
            Opcode::Cwd => {
                let ax = self.get_reg16(Reg16::AX);
                let sign = ax >> 15 & 1;
                let dx = if sign == 1 { 0xffffu16 as i16 } else { 0 };

                self.set_reg16(Reg16::DX, dx as u16);
                0
            }
            Opcode::IncRegisterMemory | Opcode::IncRegister => dst + 1,
            Opcode::DecRegisterMemory | Opcode::DecRegister => (dst as i16 - 1) as u16,
            Opcode::Mul => {
                let result = if src == 0 {
                    let ax = self.get_reg16(Reg16::AX);
                    let res = ax * dst;

                    self.set_reg16(Reg16::DX, 0);
                    self.set_reg16(Reg16::AX, res);

                    self.set_flag(Flag::CF, (res >> 15 & 1) != (dst >> 15 & 1));

                    res
                } else {
                    let result = dst * src;

                    self.set_flag(Flag::ZF, result == 0);
                    self.set_flag(Flag::SF, result >> 15 & 1 == 1);

                    result
                };

                result
            }
            Opcode::Imul => todo!(),

            Opcode::Div | Opcode::Idiv => {
                let src = self.get_reg16(Reg16::AX);

                if src == 0 || dst == 0 {
                    self.set_flag(Flag::ZF, true);
                    return dst;
                }

                let result = src / dst;

                self.set_reg16(Reg16::AX, result);
                self.set_reg16(Reg16::DX, src % dst);

                result
            }
            Opcode::Undefined => 0,
            _ => 0,
        }
    }

    /* BXの値からセグメントを取得 */
    fn interrupt_syscall(&mut self, addr: usize) {
        let data_segment = self.ram.read_data_size(addr);

        let _initial_bytes = &data_segment[0..=1];
        let syscall_id = bytes_to_16bit_little_endian(&data_segment[2..=3]);
        let fd = bytes_to_16bit_little_endian(&data_segment[4..=5]);
        let size = bytes_to_16bit_little_endian(&data_segment[6..=7]);
        let anonymus = bytes_to_16bit_little_endian(&data_segment[8..=9]);
        let buff = bytes_to_16bit_little_endian(&data_segment[10..=11]);

        let data = bytes_to_16bit_little_endian(&data_segment[18..=19]);

        match syscall_id {
            EXIT => {
                if unsafe { VM_MODE } {
                    println!("\n<exit({fd})>");
                }

                exit(1);
            }
            READ => {
                let real_addr = unsafe { self.ram.get_real_address(buff) };
                let x = unsafe { libc::read(fd as i32, real_addr as *mut c_void, size as usize) };

                if unsafe { VM_MODE } {
                    print!("\n<read({fd}, 0x{buff:04x}, {size}) => {x}>");
                }

                // SIが指すアドレスに対して，data2をコピー
                let si_val = self.get_reg16(Reg16::SI);
                self.ram.write_i16(si_val + 2, size as i16);

                self.set_reg16(Reg16::AX, 0);
            }
            WRITE => {
                // ソースインデックスから取得
                let bytes = self.ram.read_data_slice(buff, size);

                let target = match std::str::from_utf8(bytes) {
                    Ok(s) => s,
                    Err(_) => "",
                };

                if unsafe { VM_MODE } {
                    print!("\n<write({fd}, 0x{buff:04x}, {size})");
                }

                print!("{target}");

                if unsafe { VM_MODE } {
                    print!(" => {size}>");
                }

                // SIが指すアドレスに対して，data2をコピー
                let si_val = self.get_reg16(Reg16::SI);
                self.ram.write_i16(si_val + 2, size as i16);

                self.set_reg16(Reg16::AX, 0);
            }
            OPEN => {
                // ソースインデックスから取得
                let bytes = self.ram.read_data_slice(buff, size);
                let filename = self.ram.get_string(anonymus);

                let target = match std::str::from_utf8(bytes) {
                    Ok(s) => s,
                    Err(_) => "",
                };

                let x = CString::new(filename.clone()).unwrap();
                let res = unsafe { libc::open(x.as_ptr(), O_RDWR) };

                if unsafe { VM_MODE } {
                    print!("\n<open(\"{filename}\", {size}){target} => {res}>");
                }

                // SIが指すアドレスに対して，data2をコピー
                let si_val = self.get_reg16(Reg16::SI);
                self.ram.write_i16(si_val + 2, res as i16);

                self.set_reg16(Reg16::AX, 0);
            }
            CLOSE => {
                let res = unsafe { libc::close(fd as c_int) };

                if unsafe { VM_MODE } {
                    print!("\n<close({fd}) => {res}>");
                }

                // SIが指すアドレスに対して，data2をコピー
                let si_val = self.get_reg16(Reg16::SI);
                self.ram.write_i16(si_val + 2, res as i16);

                self.set_reg16(Reg16::AX, 0);
            }
            BRK => {
                if unsafe { VM_MODE } {
                    print!("\n<brk(0x{buff:04x}) => 0>");
                }

                let si_val = self.get_reg16(Reg16::SI);

                self.ram.write_i16(si_val + 2, 0);
                self.ram.write_i16(addr as u16 + 18, buff as i16);

                self.set_reg16(Reg16::AX, 0);
            }
            LSEEK => {
                let res = unsafe { libc::lseek(fd as i32, buff as i64, size as c_int) };

                if unsafe { VM_MODE } {
                    print!("\n<lseek({fd}, {buff}, {size}) => {res}>");
                }

                let si_val = self.get_reg16(Reg16::SI);

                if res != -1 {
                    self.ram.write_i16(si_val + 2, 0);
                } else {
                    self.ram.write_i16(si_val + 2, 0xffeau16 as i16);
                }

                self.set_reg16(Reg16::AX, 0);
            }
            IOCTL => {
                if unsafe { VM_MODE } {
                    print!("\n<ioctl({fd}, 0x{anonymus:04x}, 0x{data:04x})>");
                }

                let si_val = self.get_reg16(Reg16::SI);

                self.ram.write_i16(si_val + 2, 0xffeau16 as i16);
                self.set_reg16(Reg16::AX, 0);
            }

            _ => {}
        }
    }

    fn print_status_header(&self) {
        if unsafe { VM_MODE } {
            println!(" AX   BX   CX   DX   SP   BP   SI   DI  FLAGS IP");
        }
    }

    fn print_current_status(&self, asm: &Assembly) {
        self.print_reg_status(Reg16::AX);
        self.print_reg_status(Reg16::BX);
        self.print_reg_status(Reg16::CX);
        self.print_reg_status(Reg16::DX);
        self.print_reg_status(Reg16::SP);
        self.print_reg_status(Reg16::BP);
        self.print_reg_status(Reg16::SI);
        self.print_reg_status(Reg16::DI);
        self.print_flags();

        if unsafe { VM_MODE } {
            print!(" {:?}", asm);
        }
    }

    fn print_reg_status(&self, reg: Reg16) {
        if unsafe { VM_MODE } {
            print!("{:04x} ", self.reg[reg as usize]);
        }
    }
}

trait Reg8Trait {
    fn get_reg8(&self, reg: Reg8) -> i8;
    fn set_reg8(&mut self, reg: Reg8, value: i8);
}

impl Reg8Trait for VM {
    fn get_reg8(&self, reg: Reg8) -> i8 {
        match reg {
            Reg8::AL | Reg8::CL | Reg8::DL | Reg8::BL => {
                let idx = reg as usize;
                // println!("\nreg8: {idx} {:02x}", (self.reg[idx] & 0xff) as i8);
                (self.reg[idx] & 0xff) as i8
            }
            Reg8::AH | Reg8::CH | Reg8::DH | Reg8::BH => {
                let idx = reg as usize - 4;
                ((self.reg[idx] << 8) & 0xff) as i8
            }
            _ => {
                eprintln!("Invalid register: {:?}", reg);
                exit(1)
            }
        }
    }

    fn set_reg8(&mut self, reg: Reg8, value: i8) {
        match reg {
            Reg8::AL | Reg8::CL | Reg8::DL | Reg8::BL => {
                let idx = reg as usize;
                self.reg[idx] = (self.reg[idx] & 0xff00) | value as u8 as u16;
            }
            Reg8::AH | Reg8::CH | Reg8::DH | Reg8::BH => {
                let idx = reg as usize - 4;
                self.reg[idx] = (self.reg[idx] & 0x00ff) | ((value as u16) << 8);
            }
            _ => {
                eprintln!("Invalid register: {:?}", reg);
                exit(1)
            }
        }
    }
}

trait Reg16Trait {
    fn get_val_from_ea(&self, ea: &EA) -> u16;
    fn get_reg16(&self, reg: Reg16) -> u16;
    fn set_reg16(&mut self, reg: Reg16, value: u16);
}

impl Reg16Trait for VM {
    fn get_reg16(&self, reg: Reg16) -> u16 {
        self.reg[reg as usize]
    }

    fn set_reg16(&mut self, reg: Reg16, value: u16) {
        self.reg[reg as usize] = value;
    }

    fn get_val_from_ea(&self, ea: &EA) -> u16 {
        match ea {
            EA::BxSi(d) => {
                let bx = self.get_reg16(Reg16::BX);
                let si = self.get_reg16(Reg16::SI);
                ((bx + si) as i16 + d.0 as i16) as u16
            }
            EA::BxDi(d) => {
                let bx = self.get_reg16(Reg16::BX);
                let si = self.get_reg16(Reg16::DI);
                ((bx + si) as i16 + d.0 as i16) as u16
            }
            EA::BpSi(d) => {
                let bp = self.get_reg16(Reg16::BP);
                let si = self.get_reg16(Reg16::SI);
                ((bp + si) as i16 + d.0 as i16) as u16
            }
            EA::BpDi(d) => {
                let bp = self.get_reg16(Reg16::BP);
                let di = self.get_reg16(Reg16::DI);
                ((bp + di) as i16 + d.0 as i16) as u16
            }
            EA::Bx(d) => {
                let bx = self.get_reg16(Reg16::BX);
                (bx as i16 + d.0 as i16) as u16
            }
            EA::Si(d) => {
                let si = self.get_reg16(Reg16::SI);
                (si as i16 + d.0 as i16) as u16
            }
            EA::Di(d) => {
                let di = self.get_reg16(Reg16::DI);
                (di as i16 + d.0 as i16) as u16
            }
            EA::Bp(d) => {
                let bp = self.get_reg16(Reg16::BP);
                (bp as i16 + d.0 as i16) as u16
            }
            EA::DispOnly(d) => d.0 as u16,
        }
    }
}

fn display_byte_flag(target_operand: Option<&Operand>, opcode: &Opcode) -> bool {
    if let Some(Operand::Register(Register::Reg8(_))) = target_operand {
        return true;
    }

    match opcode {
        Opcode::TestImmediateByte
        | Opcode::CmpImmediateByte
        | Opcode::MovImmediateRegisterMemoryByte => return true,
        _ => (),
    };

    false
}
