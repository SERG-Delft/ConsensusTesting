use std::convert::TryFrom;

use crate::deserialization::types::{AccountID, Amount, Blob, Hash256, UInt16, UInt32, UInt64, UInt8, Vector256};

pub struct BlobParser<'a> {
    blob: &'a mut [u8],
    start: usize,
    end: usize,
}

impl BlobParser<'_> {
    pub fn new(blob: &mut [u8]) -> BlobParser {
        let length = blob.len();
        BlobParser { blob, start: 0, end: length }
    }

    fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn has_next(&self) -> bool {
        0 < self.len()
    }

    fn peek(&self) -> u8 {
        self.blob[self.start]
    }

    fn next_n_bytes(&mut self, n: usize) -> &mut [u8] {
        self.start += n;
        &mut self.blob[self.start - n..self.start]
    }

    pub fn last_n_bytes(&mut self, n: usize) -> &mut [u8] {
        self.end -= n;
        &mut self.blob[self.end..self.end + n]
    }

    pub fn next_byte(&mut self) -> u8 {
        self.next_n_bytes(1)[0]
    }

    pub fn last_byte(&mut self) -> u8 {
        self.last_n_bytes(1)[0]
    }

    fn read_variable_length(&mut self) -> usize {
        let byte_1 = self.next_byte();
        match byte_1 {
            0..=192 => {
                byte_1 as usize
            }
            193..=240 => {
                let byte_2 = self.next_byte();
                193 + (((byte_1 as usize) - 193) * 256) + (byte_2 as usize)
            }
            241..=254 => {
                let byte_2 = self.next_byte();
                let byte_3 = self.next_byte();
                12481 + (((byte_1 as usize) - 241) * 65536) + ((byte_2 as usize) * 256) + (byte_3 as usize)
            }
            _ => { panic!("size out of range!") }
        }
    }

    pub fn read_account_id(&mut self) -> AccountID {
        self.next_byte();
        AccountID { id: <&mut [u8; 20]>::try_from(self.next_n_bytes(20)).unwrap() }
    }

    pub fn read_amount(&mut self) -> Amount {
        let is_xrp: bool = (self.peek() & 0b1000_0000) == 0;
        if !is_xrp { panic!("cannot parse issued currency") }
        Amount { amount: self.next_n_bytes(8) }
    }

    pub fn read_blob(&mut self) -> Blob {
        let length = self.read_variable_length();
        Blob { blob: self.next_n_bytes(length) }
    }

    pub fn read_hash256(&mut self) -> Hash256 {
        Hash256 { hash: <&mut [u8; 32]>::try_from(self.next_n_bytes(32)).unwrap() }
    }

    pub fn read_uint16(&mut self) -> UInt16 {
        UInt16 { value: self.next_n_bytes(2) }
    }

    pub fn read_uint32(&mut self) -> UInt32 {
        UInt32 { value: self.next_n_bytes(4) }
    }

    pub fn read_uint64(&mut self) -> UInt64 {
        UInt64 { value: self.next_n_bytes(8) }
    }

    pub fn read_uint8(&mut self) -> UInt8 {
        UInt8 { value: self.next_n_bytes(1) }
    }

    pub fn read_vector256(&mut self) -> Vector256 {
        let length = self.read_variable_length();
        Vector256 { blob: self.next_n_bytes(length) }
    }
}
