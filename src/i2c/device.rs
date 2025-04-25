use std::collections::HashMap;
use crate::core::{byte_to_bits, Port, Register};
use super::LineCondition;



#[derive(Clone, Copy, Debug)]
pub enum SlaveState {
    Idle,
    ReadingAddress,
    WaitingForRegisterAddress,
    WaitingForContinuation,
    StartRead,
    StartWrite,
    WaitingAckRead
}

pub struct I2CSlave {
    address: u8,
    registers: HashMap<u8, Register>,
    pub state: SlaveState,
    output: Port,
    input_buffer: Port,
    reg_select: Option<u8>,
    /// This will be set to 
    disengaged: bool
}



impl I2CSlave {
    pub fn new(address: u8) -> Self {
        if address & 0x80 != 0 {
            panic!("7-bit addressing is the only mode supported.");
        }
        Self {
            address,
            registers: HashMap::default(),
            state: SlaveState::Idle,
            output: Port::new(),
            input_buffer: Port::new(),
            reg_select: None,
            disengaged: false
        }
    }
    pub fn create_register(&mut self, address: u8, register: Register) {
        self.registers.insert(address, register);
    }
    pub fn write_byte(&mut self, val: u8, condition: LineCondition) {
        if self.disengaged && condition != LineCondition::Start {
            
            return; // We are disengaged.
        }
        // println!("Writing {:#x} {condition:?} {:?}", self.address, self.disengaged);
        for bit in byte_to_bits(val) {
            self.write_bit(bit, condition);
        }
    }
    pub fn write_bit(&mut self, bit: bool, condition: LineCondition) {
        match self.state {
            SlaveState::Idle => {
            
                if self.disengaged && condition == LineCondition::Start {
                    self.disengaged = false;
                    println!("Device [{:#x}] is reengaging the I2C bus.", self.address);
                } else if self.disengaged {
                    return; // We are disengaged.
                }
                self.state = SlaveState::ReadingAddress;
                println!("[{:#x}] Device is beginning to receive data.", self.address);
                self.write_bit(bit, condition); // call this but in the new state.
            }
            SlaveState::ReadingAddress => {
                self.input_buffer.write(bit);
        
                if self.input_buffer.bits_read() == 8 {
                    let addr = self.input_buffer.read_byte().unwrap();
                    
                    println!("[{:#x}] The device requested address: {:08b}", self.address, addr);
                    if addr >> 1 == self.address {
                        // We are being addressed.
                        self.state = SlaveState::WaitingForRegisterAddress;
                        self.output.write(false); // Acknowledge.
                        if addr & 0x01 == 1 {
                            panic!("The R/W bit should have been zero.");
                        }
                        println!("[{:#x}] Device is being addressed.", self.address);
                    } else {
                        println!("[{:#x}] Device is disengaging the I2C bus.", self.address);
                        self.disengaged = true; // Disengage this I2C device.
                        self.state = SlaveState::Idle; // Return to the IDLE state.
                    }
                }
            }
            SlaveState::WaitingForRegisterAddress => {
                self.input_buffer.write(bit);
                if self.input_buffer.bits_read() == 8 {
                    let register_address = self.input_buffer.read_byte().unwrap();
                    println!("[{:#x}] Device requested register [{:#x}]", self.address, register_address);

                    if !self.registers.contains_key(&register_address) {
                        panic!("No such register is on this device!");
                    }

                    self.reg_select = Some(register_address);
                    // println!("Input: {:08b}", self.input_buffer.load::<u8>());

                    self.output.write(false); // ack
                    self.state = SlaveState::WaitingForContinuation;
                }
                
            },
            SlaveState::WaitingForContinuation => {
                if condition == LineCondition::InProgress {
                    // If we are in-progress, we want to direct this call to the writing
                    // methods.
                    println!("[{:#x}] Device is starting write.", self.address);
                    self.state = SlaveState::StartWrite;
                    self.write_bit(bit, condition);
                    return;
                }
                self.input_buffer.write(bit);
                if self.input_buffer.bits_read() == 8 {
                    let received = self.input_buffer.read_byte().unwrap();

                    
                    if (received & !(0x01)) >> 1 != self.address {
                        panic!("The I2C slave did not receive the correct address.");
                    }

                    self.output.write(false);

                    if received & 0x01 == 1 {
                        println!("[{:#x}] Starting read on register [{:#x}]", self.address, self.reg_select.unwrap());

                        // We simulate a register read by filling an internal register value.
                        self.registers.get_mut(&self.reg_select.unwrap()).unwrap().start_read();
                        
                        self.state = SlaveState::StartRead;
                        self.write_bit(false, condition); // We want to write some data immediately to the output port.
           
                    }

       
                }
            }
            SlaveState::StartRead => {
                // Read from the registers.
            
                let byte = self.registers.get_mut(&self.reg_select.unwrap()).unwrap().read_byte();
                self.output.write_byte(byte.unwrap());
                self.state = SlaveState::WaitingAckRead; // Waiting for an ACK
            }
            SlaveState::StartWrite => {

                // Write the bit to the register
                self.registers.get_mut(&self.reg_select.unwrap()).unwrap().write(bit);
                
                // Every seven bits send an acknowlegement.
                self.input_buffer.write(false);
                if self.input_buffer.bits_read() == 8 {
                    println!("[{:#x}] Write ACK", self.address);
                    self.output.write(false); // Write an acknowledgement.
                    self.input_buffer.clear();

                    if condition == LineCondition::Stop {
                        println!("[{:#x}] Device has received stop signal, write complete.", self.address);
                        self.state = SlaveState::Idle;
                    }
                }

                
            }
            SlaveState::WaitingAckRead => {
                
                if bit {
                    println!("[{:#x}] Received a NACK from master.", self.address);
                    self.registers.get_mut(&self.reg_select.unwrap()).unwrap().finish_read();
                    self.state = SlaveState::Idle; // Return to the IDLE state.
                } else {
                    println!("[{:#x}] Byte transmission acknowledged.", self.address);
                    self.state = SlaveState::StartRead; // Continue reading out registers.
                    self.write_bit(false, condition); // Send out some more data.
                }

            }
        }
    }
    
    pub fn read_bit(&mut self) -> Option<bool> {
        self.output.read()
    }
    pub fn read_byte(&mut self) -> Option<u8> {
        self.output.read_byte()
    }
}