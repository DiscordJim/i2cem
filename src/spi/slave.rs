use std::{
    collections::HashMap, marker::PhantomData, sync::{Arc, Mutex}
};

use crate::
    core::{Port, Register}
;

use super::{master::{Connected, Disconnected}, wire::SpiMedium};



pub struct SpiSlave<S> {
    inner: Arc<SpiSlaveInner>,
    _type: PhantomData<S>
}

struct SpiSlaveInner {
    registers: Mutex<HashMap<u8, Register>>,
    port: Mutex<Port>,
    output: Mutex<Port>,

}

impl SpiSlave<Disconnected> {
    pub fn new(registers: HashMap<u8, Register>) -> Self {
        Self {
            inner: Arc::new(SpiSlaveInner {
                registers: registers.into(),
                port: Mutex::new(Port::new()),
                output: Mutex::new(Port::new()),
            }),
            _type: PhantomData
        }
    }
    pub fn accept_medium(self, medium: &Arc<SpiMedium>) -> SpiSlave<Connected> {
        std::thread::spawn({
            let medium = medium.clone();
            let inner = self.inner.clone();
            move || handle_medium(medium, inner)
        });

        SpiSlave { inner: self.inner, _type: PhantomData }
    }
}

enum SpiSlaveState {
    Idle,
    Writing(u8)
}

fn handle_medium(medium: Arc<SpiMedium>, inner: Arc<SpiSlaveInner>) {
    // Lets us do rising edge detection.
    let mut previous_value = false;
    let mut state = SpiSlaveState::Idle;
    loop {
        let clock = medium.clock.get_clock();

        if medium.kill.read() {
            // Kill the slave.
            break;
        }

        if clock && !previous_value {
            // Rising edge detected.
            on_rising_edge(&medium, &inner, &mut state);
        }
        previous_value = clock;
    }
}

fn on_rising_edge(medium: &SpiMedium, inner: &SpiSlaveInner, state: &mut SpiSlaveState) {

    if medium.cs_select.read() {
        // Chip select is set to high.
        *state = SpiSlaveState::Idle;
        return;
    }

    // If there is a bit to send out, we should send it.
    if inner.output.lock().unwrap().bits_read() > 0 {
        // Writes the output buffe.r
        medium.miso.pull(inner.output.lock().unwrap().read().unwrap());
    } else {
        // Read the MOSI line.
        read_mosi(inner, medium, state);        
    }
}

/// Writes the current buffer to MOSI.
fn read_mosi(
    inner: &SpiSlaveInner,
    medium: &SpiMedium,
    state: &mut SpiSlaveState
) {
    // Store the bit.
    let mut port_lock = inner.port.lock().unwrap();
    port_lock.write(medium.mosi.read());

    // We only read if we are not sending.
    // If we have read an entire byte, then we can send it out.
    if port_lock.bits_read() == 8 {
        let value = port_lock.read_byte().unwrap();
        drop(port_lock);
        handle_byte_read(inner, value, state);
    }
}

fn handle_byte_read(
    inner: &SpiSlaveInner,
    value: u8,
    state: &mut SpiSlaveState
) {
    println!("Byte read {:08b}", value);

    match state {
        SpiSlaveState::Idle => {
            let register = value & 0b00111111;
            if value & 0x80 != 0 {
                // Read call.
                println!("Read call for register: {:#x}", register);
                if let Some(register) = inner.registers.lock().unwrap().get_mut(&register) {
                    register.start_read();

                    while !register.is_done() {

                        // Write the register.
                        let rb = register.read_bit().unwrap();
                        println!("rb: {}", rb);
                        inner.output.lock().unwrap().write(rb);
                    }

                    register.finish_read();
      
                } else {
                    println!("No such register exists.");
                }
            } else {
                println!("Write call for register: {:#x}", register);
                if let Some(rgstr) = inner.registers.lock().unwrap().get_mut(&register) {
                    rgstr.start_write();
                    *state = SpiSlaveState::Writing(register);
                } else {
                    println!("No such register exists.");
                }
                
            }
        }
        SpiSlaveState::Writing(register) => {
            println!("Writing byte {:#x} to register [{:#x}]", value, *register);
            if let Some(register) = inner.registers.lock().unwrap().get_mut(register) {
                register.write_byte(value);
            } else {
                println!("No such register exists.");
            }
        }
    }
}
