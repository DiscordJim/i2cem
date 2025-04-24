use crate::device::I2CSlave;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineCondition {
    Start,
    InProgress,
    Stop
}

pub struct I2CBus {
    devices: Vec<I2CSlave>,
    line: Option<u8>,
    line_bit: Option<bool>
}

impl I2CBus {
    pub fn new() -> Self {
        Self {
            devices: vec![],
            line: None,
            line_bit: None
        }
    }
    pub fn add_device(&mut self, device: I2CSlave) {
        self.devices.push(device);
    }
    pub fn write_byte(&mut self, value: u8, condition: LineCondition) {
        for device in &mut self.devices {
            device.write_byte(value, condition);
        }
        assert_eq!(self.read_bit(), Some(false)); // get the ack
    }
    pub fn write_bit(&mut self, bit: bool, condition: LineCondition) {
        for device in &mut self.devices {
            device.write_bit(bit, condition);
        }
    }
    pub fn read_bit(&mut self) -> Option<bool> {
        // println!("ENTERING CALL");
        for value in &mut self.devices {
            let read = value.read_bit();
            // println!("READING BIT: {:?} {:?}", read, self.line_bit);
            if read.is_some() && self.line_bit.is_some() {
                panic!("Multiple devices are writing to the bus!");
            }

            if read.is_some() {
                self.line_bit = read;
            }

            // println!("READING BIT 2: {:?} {:?}", read, self.line_bit);
            
        }
        self.line_bit.take()

    }
    pub fn read_byte(&mut self) -> Option<u8> {
        for value in &mut self.devices {
            let read = value.read_byte();
            if read.is_some() && self.line.is_some() {
                panic!("Multiple devices are writing to the bus!");
            }

            if read.is_some() {
                self.line = read;
            }

        }
        self.line.take()

    }
}