use std::fmt;

use crate::arch::asm::Assembly;

#[repr(C)]
#[derive(Clone)]
pub struct AOutHeader {
    // 8bytes
    pub magic_number: [u8; 2], /* magic number */
    pub flags: u8,             /* flags */
    pub cpu_id: u8,            /* size of symbol table */
    pub length: u8,            /* size of symbol table */
    pub unused: u8,            /* size of symbol table */
    pub version: u16,          /* size of symbol table */

    // 12bytes
    pub text_size: u32, /* size of symbol table */
    pub data_size: u32, /* size of symbol table */
    pub bss_size: u32,  /* size of symbol table */

    // 4bytes
    pub entry_point: u32, /* entry point */
    pub total: u32,       /* total memory allocated */
    pub syms: u32,        /* size of symbol table */
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: Vec<u8>,
    pub offset: usize,
    pub size: usize,
    pub asm: Vec<Assembly>,
    pointer: usize,
}

impl Text {
    pub fn new(text: Vec<u8>, offset: usize) -> Self {
        let size = text.len();
        Self {
            text,
            offset,
            size,
            pointer: 0,
            asm: Vec::new(),
        }
    }
}

pub struct Data {
    pub data: Vec<u8>,
    pub offset: u32,
    pub user_size: u32,
    pub all_size: u32,
}

pub struct Bss {
    pub data: Vec<u8>,
    pub offset: u32,
    pub user_size: u32,
    pub all_size: u32,
}

// 32ビット整数を16進数の文字列に変換する関数
fn stringfy_u32(n: u32) -> String {
    format!(
        "{:02x} {:02x} {:02x} {:02x}",
        (n >> 24) as u8,
        (n >> 16) as u8 & 0xFF,
        (n >> 8) as u8 & 0xFF,
        n as u8 & 0xFF
    )
}

impl fmt::Debug for AOutHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"
Header {{
    magic:     {:02x} {:02x}
    flags:     {:02x}
    cpu_id:    {:02x}
    length:    {:02x}
    unused:    {:02x}
    version:   {:02x} {:02x}

    text_size: {} -> {}
    data_size: {} -> {}
    bss_size:  {} -> {}

    entry:     {} -> {}
    total:     {} -> {}
    syms:      {} -> {}
}}
"#,
            self.magic_number[0],
            self.magic_number[1],
            self.flags,
            self.cpu_id,
            self.length,
            self.unused,
            (self.version >> 8) as u8,
            (self.version & 0xFF) as u8,
            stringfy_u32(self.text_size),
            self.text_size,
            stringfy_u32(self.data_size),
            self.data_size,
            stringfy_u32(self.bss_size),
            self.bss_size,
            stringfy_u32(self.entry_point),
            self.entry_point,
            stringfy_u32(self.total),
            self.total,
            stringfy_u32(self.syms),
            self.syms,
        )
    }
}
