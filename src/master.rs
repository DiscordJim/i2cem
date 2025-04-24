use crate::{bus::{I2CBus, LineCondition}, device::I2CSlave};

pub struct Master {
    bus: I2CBus
}

impl Master {
    pub fn new() -> Self {
        Self {
            bus: I2CBus::new()
        }
    }
    pub fn add_device(&mut self, device: I2CSlave) {
        self.bus.add_device(device);
    }
    /// Writes a block of bytes from the I2C device.
    pub fn write_block(&mut self, device_addr: u8, reg_addr: u8, bytes: Vec<u8>) {
        assert!(device_addr & 0x80 == 0, "Only 7-bit addressing is supported.");
        assert!(reg_addr & 0x80 == 0, "Only 7-bit addressing is supported.");
        // [ Slave Addr (7-bit) ] [ R/W bit = 0 ]
        self.bus.write_byte(device_addr << 1, crate::bus::LineCondition::Start);
        
        // [ Register addr (7-bit) ]
        self.bus.write_byte(reg_addr, crate::bus::LineCondition::InProgress);
        


        let mut i = (bytes.len() - 1) as isize;
        while i >= 0 {

            if i == 0 {
                self.bus.write_byte(bytes[i as usize], crate::bus::LineCondition::Stop);
            } else {
                self.bus.write_byte(bytes[i as usize], crate::bus::LineCondition::InProgress);
            }
            i -= 1;
        }

       
        


        

    }
    /// Reads a block of bytes from the I2C device specified.
    pub fn read_block(&mut self, device_addr: u8, reg_addr: u8, bytes: u8) -> Vec<u8> {
        assert!(device_addr & 0x80 == 0, "Only 7-bit addressing is supported.");
        assert!(reg_addr & 0x80 == 0, "Only 7-bit addressing is supported.");
        // [ Slave Addr (7-bit) ] [ R/W bit = 0 ]
        self.bus.write_byte(device_addr << 1, crate::bus::LineCondition::Start);
        

        // println!("ODNE");

        // [ 0 bit ] [ Register addr (7-bit) ]
        self.bus.write_byte(reg_addr, crate::bus::LineCondition::InProgress);
        

        // [ Slave Addr (7-bit) ] [ R/W bit = 1 ]
        self.bus.write_byte((device_addr << 1) | 0x01, crate::bus::LineCondition::Start);

        let mut result = vec![];
        for i in 0..bytes {
            result.push(self.bus.read_byte().unwrap());
            if i >= bytes - 1 {
                // We are done.
                self.bus.write_bit(true, crate::bus::LineCondition::Stop);
            } else {
                // More data.
                self.bus.write_bit(false, LineCondition::InProgress);
            }
        }


        result
    }
}