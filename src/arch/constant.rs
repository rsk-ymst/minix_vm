pub mod reg {
    // 16bit(w = 1)
    pub const AX: isize = 0b000;
    pub const CX: isize = 0b001;
    pub const DX: isize = 0b010;
    pub const BX: isize = 0b011;
    pub const SP: isize = 0b100;
    pub const BP: isize = 0b101;
    pub const SI: isize = 0b110;
    pub const DI: isize = 0b111;

    // 8bit(w = 0)
    pub const AL: isize = 0b000;
    pub const CL: isize = 0b001;
    pub const DL: isize = 0b010;
    pub const BL: isize = 0b011;
    pub const AH: isize = 0b100;
    pub const CH: isize = 0b101;
    pub const DH: isize = 0b110;
    pub const BH: isize = 0b111;

    // Segment
    pub const ES: isize = 0b00;
    pub const CS: isize = 0b01;
    pub const SS: isize = 0b10;
    pub const DS: isize = 0b11;
}

pub mod opcode {
    // 16bit(w = 1)
    pub const MOV_IMMEDIATE_REGISTER_MEMORY_7BIT: isize = 0b_1100_011;
    pub const MOV_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b_000;
    pub const MOV_IMMEDIATE: isize = 0b_1011;
    pub const MOV_RM_TO_FROM_REG: isize = 0b_1000_10;
    pub const MOV_MEMORY_TO_REGISTER: isize = 0b1010_000;

    pub const XCHG_REGISTER_MEMORY_WITH_REGISTER: isize = 0b_1000_011;
    pub const XCHG_REGISTER_WITH_ACCUMULATOR: isize = 0b_1001_0;

    // Interruption
    pub const INT_TYPE_SPECIFIED: isize = 0b11001101;

    // 汎用系
    pub const IMMEDIATE_WITH_REGISTER_MEMORY_6BIT: isize = 0b100000;

    // Add
    pub const ADD_REG_EITHER: isize = 0b000000;
    pub const ADD_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b000;
    pub const ADD_IMMEDIATE_WITH_ACCUMULATOR: isize = 0b100000;
    pub const ADD_IMMEDIATE_TO_ACCUMULATOR: isize = 0b0000_010;

    // Adc
    pub const ADC_REG_EITHER: isize = 0b000100;
    pub const ADC_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b010;
    pub const ADC_IMMEDIATE_WITH_ACCUMULATOR: isize = 0b0001010;

    // Sub
    pub const SUB_REG_EITHER: isize = 0b001010;
    pub const SUB_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b101;
    pub const SUB_IMMEDIATE_WITH_ACCUMULATOR: isize = 0b0010110;

    pub const SHR_3BIT: isize = 0b_101;
    pub const SAR_3BIT: isize = 0b_111;
    pub const ROL_3BIT: isize = 0b_000;
    pub const ROR_3BIT: isize = 0b_001;
    pub const RCL_3BIT: isize = 0b_010;
    pub const RCR_3BIT: isize = 0b_011;

    // And
    pub const AND_REG_EITHER: isize = 0b001000;
    pub const AND_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b100;
    pub const AND_IMMEDIATE_WITH_ACCUMULATOR: isize = 0b0010010;

    // Test
    pub const TEST_REGISTER_MEMORY_AND_REGISTER: isize = 0b1000010;
    pub const TEST_IMMEDIATE_RM: isize = 0b000;
    pub const TEST_IMMEDIATE_DATA_AND_ACCUMULATOR: isize = 0b1010100;

    // SSB
    pub const SSB_REG_EITHER: isize = 0b000110;
    pub const SSB_IMMEDIATE_REGISTER_MEMORY_8BIT: isize = 0b100000;
    pub const SSB_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b011;
    pub const SSB_IMMEDIATE_WITH_ACCUMULATOR: isize = 0b000111;

    // MUL, DIV, NEG, NOT, TEST....
    pub const MUL_DIV_7BIT: isize = 0b1111011;
    pub const MUL_3BIT: isize = 0b100;
    pub const IMUL_3BIT: isize = 0b101;

    pub const DIV_3BIT: isize = 0b110;
    pub const IDIV_3BIT: isize = 0b111;
    pub const NEG_3BIT: isize = 0b011;
    pub const NOT_3BIT: isize = 0b010;

    // OR
    pub const OR_REG_EITHER: isize = 0b000010;
    pub const OR_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b001;
    pub const OR_IMMEDIATE_WITH_ACCUMULATOR: isize = 0b0000110;

    // Xor
    pub const XOR_REG_EITHER: isize = 0b001100;

    // CMP
    pub const CMP_REG_EITHER: isize = 0b001110;
    pub const CMP_IMMEDIATE_WITH_REGISTER_MEMORY_3BIT: isize = 0b111;
    pub const CMP_IMMEDIATE_WITH_ACCUMULATOR: isize = 0b0011110;

    // Load
    pub const LEA: isize = 0b_1000_1101;
    pub const LDS: isize = 0b_1100_0101;
    pub const LES: isize = 0b_1100_0100;

    pub const RET_WITHIN_SEGMENT: isize = 0b_1100_0011;
    pub const RET_WITHIN_SEG_ADDING_IMMED_TO_SP: isize = 0b_1100_0010;
    pub const RET_INTERSEGMENT: isize = 0b_1100_1011;
    pub const RET_INTERSEGMENT_ADDING_IMMEDIATE_TO_SP: isize = 0b_1100_1010;

    pub const JE: isize = 0b_0111_0100;
    pub const JL: isize = 0b_0111_1100;
    pub const JLE: isize = 0b_0111_1110;
    pub const JB: isize = 0b_0111_0010;
    pub const JBE: isize = 0b_0111_0110;
    pub const JP: isize = 0b_0111_1010;
    pub const JO: isize = 0b_0111_0000;
    pub const JS: isize = 0b_0111_1000;

    // JMP
    pub const JNE: isize = 0b_0111_0101;
    pub const JNL: isize = 0b_0111_1101;
    pub const JNLE: isize = 0b_0111_1111;
    pub const JNB: isize = 0b_0111_0011;
    pub const JNBE: isize = 0b_0111_0111;
    pub const JNP: isize = 0b_0111_1011;
    pub const JNO: isize = 0b_0111_0001;
    pub const JNS: isize = 0b_0111_1001;

    // LOOP
    pub const LOOP: isize = 0b_1110_0010;
    pub const LOOPZ: isize = 0b_1110_0001;
    pub const LOOPNZ: isize = 0b_1110_0000;
    pub const JCXZ: isize = 0b_1110_0011;

    pub const IMMEDIATE_DATA: isize = 0b_1111011;

    // In
    pub const IN_FIXED_PORT: isize = 0b_1110_010;
    pub const IN_VARIABLE_PORT: isize = 0b_1110_110;

    // Out
    pub const OUT_FIXED_PORT: isize = 0b_1110_011;
    pub const OUT_VARIABLE_PORT: isize = 0b_1110_111;
    pub const CONVERT_BYTE_TO_BYTE: isize = 0b_1001_1000;
    pub const CONVERT_WORD_TO_DOUBLE_WORD: isize = 0b_1001_1001;

    pub const INC_DEC_7BIT: isize = 0b_1111_111;

    pub const INC_REGISTER: isize = 0b_01000;
    pub const INC_REGISTER_MEMORY_3BIT: isize = 0b_000;

    pub const DEC_REGISTER_MEMORY_3BIT: isize = 0b_001;

    pub const CLC: isize = 0b_1111_1000;
    pub const CMC: isize = 0b_1111_0101;
    pub const CLD: isize = 0b_1111_1100;
    pub const STD: isize = 0b_1111_1101;
    pub const CLI: isize = 0b_1111_1010;
    pub const STI: isize = 0b_1111_1011;

    pub const LOGIC_6BIT: isize = 0b110100;

    pub const REP: isize = 0b_1111001;
    pub const COMPS: isize = 0b_1010011;

    pub const PUSH_RM: isize = 0b_1111_1111;
    pub const POP_RM: isize = 0b_1000_1111;
    pub const HLT: isize = 0b_1111_0100;

    pub const CALL_DIRECT_WITHIN_SEGMENT: isize = 0b_1110_1000;

    pub const CALL_INDIRECT_WITHIN_SEGMENT_8BIT: isize = 0b_1111_1111;
    pub const CALL_INDIRECT_WITHIN_SEGMENT_3BIT: isize = 0b_010;

    pub const CALL_INDIRECT_INTERSEGMENT_8BIT: isize = 0b_1111_1111;
    pub const CALL_INDIRECT_INTERSEGMENT_3BIT: isize = 0b_011;

    pub const JMP_DIRECT_WITHIN_SEGMENT: isize = 0b_1110_1001;
    pub const JMP_DIRECT_WITHIN_SEGMENT_SHORT: isize = 0b_1110_1011;
    pub const JMP_INDIRECT_WITHIN_SEGMENT_3BIT: isize = 0b_100;

    pub const PUSH_REG: isize = 0b_01010;
    pub const POP_REG: isize = 0b_01011;
    pub const DEC_REGISTER: isize = 0b_01001;

    pub const SHL_6BIT: isize = 0b_110100;
    pub const SHL_3BIT: isize = 0b_100;

    pub const SHL: isize = 0b_110100;
}

pub mod size {
    pub const REG_EITHER_SIZE: usize = 2;
}

pub mod syscall {
    pub const EXIT: u16 = 1;
    pub const READ: u16 = 3;
    pub const WRITE: u16 = 4;
    pub const OPEN: u16 = 5;
    pub const CLOSE: u16 = 6;
    pub const BRK: u16 = 17;
    pub const LSEEK: u16 = 19;
    pub const IOCTL: u16 = 54;
}
