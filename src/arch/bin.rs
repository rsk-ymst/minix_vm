use crate::arch::bin;
use std::iter::Peekable;
use std::mem::size_of_val;
use std::u8;
use std::vec::IntoIter;

pub struct BinaryManager<I: Iterator> {
    stream: Peekable<I>,
    header: Option<AOutHeader>,
    text: Option<Text>,
    data: Option<Data>,
    bss: Option<Bss>,
    pointer: usize,
    text_pointer: usize,
}

impl BinaryManager<IntoIter<u8>> {
    pub fn new(stream: Vec<u8>) -> Self {
        let mut instance = Self {
            stream: stream.into_iter().peekable(),
            header: None,
            text: None,
            data: None,
            bss: None,
            pointer: 0,
            text_pointer: 0,
        };

        instance.parse();

        instance
    }

    pub fn parse(&mut self) -> Option<()> {
        self.header = self.make_header();
        self.text = self.make_text();

        Some(())
    }

    pub fn make_header(&mut self) -> Option<AOutHeader> {
        Some(AOutHeader {
            magic_number: [self.consume_u8()?, self.consume_u8()?],
            flags: self.consume_u8()?,
            cpu_id: self.consume_u8()?,
            length: self.consume_u8()?,
            unused: self.consume_u8()?,
            version: self.consume_u16()?,
            text_size: self.consume_u32()?,
            data_size: self.consume_u32()?,
            bss_size: self.consume_u32()?,
            entry_point: self.consume_u32()?,
            total: self.consume_u32()?,
            syms: self.consume_u32()?,
        })
    }

    pub fn make_text(&mut self) -> Option<Text> {
        let size = self.header.as_ref()?.text_size as u32;

        let mut text = Vec::new();

        for _ in 0..size {
            text.push(self.consume_u8()?);
        }

        let x = Text::new(text, 0);

        Some(x)
    }

    pub fn get_header_size(&self) -> usize {
        if let Some(val) = self.header.as_ref() {
            size_of_val(val)
        } else {
            0
        }
    }

    pub fn get_text_size(&self) -> usize {
        if let Some(val) = self.header.as_ref() {
            val.text_size as usize
        } else {
            0
        }
    }

    pub fn get_data_size(&self) -> usize {
        if let Some(val) = self.header.as_ref() {
            val.data_size as usize
        } else {
            0
        }
    }

    pub fn get_bss_size(&self) -> usize {
        if let Some(val) = self.header.as_ref() {
            val.bss_size as usize
        } else {
            0
        }
    }
}

impl BinaryConsume for BinaryManager<IntoIter<u8>> {
    fn consume_u8(&mut self) -> Option<u8> {
        self.pointer += 1;
        self.stream.next()
    }

    fn consume_u16(&mut self) -> Option<u16> {
        Some((self.consume_u8()? as u16) << 8 | self.consume_u8()? as u16)
    }

    fn consume_u32(&mut self) -> Option<u32> {
        let target = (self.consume_u16()? as u32) << 16 | self.consume_u16()? as u32;

        let target = bin::reverse_order_u32(target)?;

        Some(target)
    }
}
use super::header::{AOutHeader, Bss, Data, Text};

pub trait BinaryConsume {
    fn consume_u8(&mut self) -> Option<u8>;
    fn consume_u16(&mut self) -> Option<u16>;
    fn consume_u32(&mut self) -> Option<u32>;
}

pub trait BinaryPeek {
    fn peek_u8(&mut self) -> Option<u8>;
    fn peek_u16(&mut self) -> Option<u16>;
    fn peek_u32(&mut self) -> Option<u32>;
    fn peek_offset(&mut self, offset: usize) -> Option<u8>;
}

pub fn reverse_order_u16(target: u16) -> Option<u16> {
    Some((target << 8 & 0xff00) | (target >> 8 & 0x00ff))
}

pub fn reverse_order_u32(target: u32) -> Option<u32> {
    Some((target >> 24) | (target >> 8 & 0xff00) | (target << 8 & 0xff0000) | (target << 24))
}

pub fn bytes_to_16bit_little_endian(bytes: &[u8]) -> u16 {
    (bytes[1] as u16) << 8 | bytes[0] as u16
}

pub fn bytes_to_16bit_big_endian(bytes: &[u8]) -> u16 {
    (bytes[0] as u16) << 8 | bytes[1] as u16
}

pub fn get_reg_mem_element(value: u8) -> (u8, u8, u8) {
    let mod_ = value >> 6 & 0b11;
    let reg = value >> 3 & 0b111;
    let rm = value & 0b111;

    (mod_, reg, rm)
}

// 対象の8bitの下位2bit dwを取得するｊｊ
pub fn get_dw(value: u8) -> (u8, u8) {
    let d = value >> 1 & 0b1;
    let w = value & 0b1;

    (d, w)
}
