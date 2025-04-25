use super::{byte_to_bits, Port};


pub struct Register {
    buffer: Port,
    populator: Option<fn() -> Vec<u8>>,
    read_only: bool,
    // Allows us to restore the contents of the write register after.
    backing: Port
}

impl Register {
    pub fn new_read_only(populator: fn() -> Vec<u8>) -> Self {
        Self {
            buffer: Port::new(),
            populator: Some(populator),
            read_only: true,
            backing: Port::new()
        }
    }
    pub fn new_writeable() -> Self {
        let mut value = Self {
            buffer: Port::new(),
            populator: None,
            read_only: false,
            backing: Port::new()
        };
        value.backing.write_byte(0x00);
        value.refill_buffers();
        value
    }
    pub fn start_write(&mut self) {
        if self.read_only {
            return; // cannot write on a read-only register.
        }
        self.buffer.clear();
        self.backing.clear();
    }
    pub fn is_done(&self) -> bool {
        self.buffer.bits_read() == 0
    }
    pub fn write_byte(&mut self, value: u8) {
        for bit in byte_to_bits(value) {
            self.write(bit);
        }
    }
    pub fn write(&mut self, bit: bool) {
        if !self.read_only {
            // Not a read only register. We will write this bit.
            self.backing.write(bit);
            self.buffer.write(bit);
        }
    }
    pub fn start_read(&mut self) {
        if self.read_only {
            // Populate the value.
            let mut boof = (self.populator.as_ref().unwrap())();
            boof.reverse();
            for byte in  boof {
                self.buffer.write_byte(byte);
            }
        }
    }
    pub fn read_bit(&mut self) -> Option<bool> {
        if self.buffer.bits_read() == 0 {
            // If the buffer is empty we just send 0x00.
            None
        } else {
            self.buffer.read()
        }
    } 
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.buffer.bits_read() == 0 {
            // If the buffer is empty we just send 0x00.
            Some(0x00)
        } else {
            self.buffer.read_byte()
        }
    } 
    
    fn refill_buffers(&mut self) {
        self.buffer.clear();
        let mut values = vec![];
        while self.backing.bits_read() != 0 {
            values.insert(0, self.backing.read_byte().unwrap());
        }
        for v in values {
            self.buffer.write_byte(v);
            self.backing.write_byte(v);
        }
    }

    pub fn finish_read(&mut self) {
        self.refill_buffers();
    }

}



#[cfg(test)]
mod tests {
    use super::Register;


    #[test]
    pub fn test_writeable_register() {
        let mut register = Register::new_writeable();
        assert_eq!(register.read_byte().unwrap(), 0x00);

        register.start_write();
        register.write_byte(0x22);
        register.write_byte(0x21);
        

        register.start_read();
        assert_eq!(register.read_byte().unwrap(), 0x21);
        assert_eq!(register.read_byte().unwrap(), 0x22);
        register.finish_read();

        register.start_read();
        assert_eq!(register.read_byte().unwrap(), 0x21);
        assert_eq!(register.read_byte().unwrap(), 0x22);


    }

    #[test]
    pub fn test_readonly_register() {

        fn populato() -> Vec<u8> {
            vec![ 0x21, 0x22 ]
        } 

        let mut register = Register::new_read_only(populato);
        assert_eq!(register.read_byte().unwrap(), 0x00);

  

        register.start_read();
        assert_eq!(register.read_byte().unwrap(), 0x21);
        assert_eq!(register.read_byte().unwrap(), 0x22);
        register.finish_read();

        register.start_read();
        assert_eq!(register.read_byte().unwrap(), 0x21);
        assert_eq!(register.read_byte().unwrap(), 0x22);


    }
}