

use bitvec::{field::BitField, order::Msb0, vec::BitVec};

#[derive(Debug)]
pub struct Port {
    buffer: BitVec
}

impl Port {
    pub fn new() -> Self {
        Self {
            buffer: BitVec::default()
        }
    }
    pub fn from_byte(value: u8) -> Self {
        let mut obj = Self::new();
        obj.write_byte(value);
        obj
    }
    
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.buffer.is_empty() {
            return None;
        }
        let (first, second) = self.buffer.split_at(self.buffer.len().min(8));
        let value: u8 = first.load();
        let second = second.to_bitvec();
        
        self.buffer = second;
        // self.clear();
        Some(value)
    }
    pub fn write_byte(&mut self, b: u8) {
        for bit in byte_to_bits(b) {
            self.write(bit);
        }
    }
    pub fn read(&mut self) -> Option<bool> {
        self.buffer.pop()
    }
    pub fn write(&mut self, bit: bool) {
        self.buffer.insert(0, bit);
    }
    pub fn bits_read(&self) -> usize {
        self.buffer.len()
    }
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

pub fn byte_to_bits(val: u8) -> BitVec<u8, Msb0> {
    BitVec::<u8, Msb0>::from_element(val)
}

impl Default for Port {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{byte_to_bits, Port};


    #[test]
    pub fn test_port_basic() {

        let mut port = Port::new();

        for bit in byte_to_bits(0b11010001) {
            port.write(bit);
        }

        assert_eq!(port.read_byte().unwrap(), 0b11010001);
        
    }
}