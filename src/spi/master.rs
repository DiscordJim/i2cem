use std::{
    collections::VecDeque, marker::PhantomData, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::sleep, time::Duration
};

use rsevents::{AutoResetEvent, Awaitable, EventState};

use crate::core::Port;

use super::{
    clock::Clock,
    slave::SpiSlave,
    wire::{LiveWire, SpiMedium},
};


/// An SPI participant that is disconnected
pub struct Disconnected;

/// An SPI participant that is connected
pub struct Connected;

/// The master in the SPI communication
pub struct SpiMaster<S> {
    /// The inner master, contains the data.
    inner: Arc<SpiMasterInner>,
    /// The phantom type that allows us to use
    /// the type state pattern.
    _type: PhantomData<S>
}

/// The inner struct that stores the master state which is necessary for communication.
struct SpiMasterInner {
    /// The instructions being sent, this allows things to be sent
    /// in an ordered manner.
    instruction: Mutex<VecDeque<InstrVar>>,
    /// Bytes being read from the wire.
    read_buf: Mutex<Port>,
    /// A signal to kill the inner thread.
    kill_switch: AtomicBool
}

/// The internal instructions being sent to the SPI port.
enum InstrVar {
    /// Writes the port using the buffer given to it.
    Write(Port),

    /// Reads a certain amount of bits from the port.
    Read {
        /// The bits to read.
        size: usize,
        /// Skip the first bit.
        first_clear: bool
    }, 
    /// Wakes up a notifier.
    Wake(Arc<AutoResetEvent>)
}


impl SpiMaster<Disconnected> {
    /// Creates a new [SpiMaster] that is disconnected.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SpiMasterInner {
                instruction: Mutex::default(),
                read_buf: Port::new().into(),
                kill_switch: AtomicBool::new(false)
            }),
            _type: PhantomData
        }
    }
    /// Connects a SPI master to a lsave.
    pub fn connect(self, slave: SpiSlave<Disconnected>, clock_speed: Duration) -> (SpiMaster<Connected>, SpiSlave<Connected>) {
        let medium = Arc::new(SpiMedium {
            clock: Clock::new(),
            cs_select: LiveWire::new(),
            miso: LiveWire::new(),
            mosi: LiveWire::new(),
            kill: LiveWire::new()
        });

        // Pull the CS select high to disable until the master is ready.
        medium.cs_select.pull(true);

        let connected = slave.accept_medium(&medium);

        std::thread::spawn({
            let inner = self.inner.clone();
            move || handle_connection_master(inner, medium, clock_speed)
        });
        (SpiMaster { inner: self.inner.clone(), _type: PhantomData }, connected)
    }
}

impl SpiMaster<Connected> {
    /// Writes to register.
    pub fn write_register(&self, reg: u8, bytes: Vec<u8>) {
        let mut instruction_buffer = self.inner.instruction.lock().unwrap();

        instruction_buffer.push_front(InstrVar::Write(Port::from_byte(reg)));
        
        for byte in bytes {
            instruction_buffer.push_front(InstrVar::Write(Port::from_byte(byte)));
        }
        let waker = Arc::new(AutoResetEvent::new(EventState::Unset));
        instruction_buffer.push_front(InstrVar::Wake(waker.clone()));


        drop(instruction_buffer);

        // Wait for the notification.
        waker.wait();
    }
    /// Reads a register.
    pub fn read_register(&self, reg: u8, mut bytes: usize) -> Vec<u8> {
        let mut instruction_buffer = self.inner.instruction.lock().unwrap();

        instruction_buffer.push_front(InstrVar::Write(Port::from_byte(reg | 0x80)));
        instruction_buffer.push_front(InstrVar::Read {
            size: bytes * 8,
            first_clear: true
        });
        let waker = Arc::new(AutoResetEvent::new(EventState::Unset));
        instruction_buffer.push_front(InstrVar::Wake(waker.clone()));


        drop(instruction_buffer);

        // Wait for the notification.
        waker.wait();


        // Read out the buffers.
        let mut read_buffer = self.inner.read_buf.lock().unwrap();
        let mut buf = vec![];
        while bytes > 0 {
            buf.push(read_buffer.read_byte().unwrap());
            bytes -= 1;
        }
        buf
    }
    /// Disconnects the master from the slave.
    pub fn disconnect(self) -> SpiMaster<Disconnected> {
        self.inner.kill_switch.store(true, Ordering::SeqCst);
        SpiMaster {
            inner: self.inner.clone(),
            _type: PhantomData
        }
    }
}


/// Handles the connection from the master side.
fn handle_connection_master(
    master: Arc<SpiMasterInner>,
    medium: Arc<SpiMedium>,
    duration: Duration,
) {
    let mut ctx = None;

 
    loop {
        // tick the cloc.
        medium.clock.tick();

        if master.kill_switch.load(std::sync::atomic::Ordering::SeqCst) {
            medium.kill.pull(true); // Kill the slave.
            break;
        }

        if !medium.clock.get_line_value() {
            // Performs logic while the line is pulled down low.
            handle_low_level(&master, &medium, &mut ctx);
        }

        // Sleep
        sleep(duration);
    }
}

/// handles the low level of data.
fn handle_low_level(
    master: &SpiMasterInner,
    medium: &SpiMedium,
    ctx: &mut Option<InstrVar>
) {

    if let Some(InstrVar::Write(port)) = ctx {
        if port.bits_read() == 0 {
            *ctx = None;
        }
    }
    
    // Load the instruction
    if ctx.is_none() {
        *ctx = master.instruction.lock().unwrap().pop_back();
    } 


    if ctx.is_some() {
        match ctx.as_mut().unwrap() {
            InstrVar::Read {
                first_clear,
                size: count
            } => {
    
                if *first_clear {
                    *first_clear = false;
                } else {
                    medium.cs_select.pull(false); // pull line down.
    
                    if *count != 0 {
                        master.read_buf.lock().unwrap().write(medium.miso.read());
                        *count -= 1;
                        if *count == 0 {
                            *ctx = None;
                        }
                    } else {
                        *ctx = None;
                    }
                    
                }
    
    
                
                
            },
            InstrVar::Write(p) => {
                if p.bits_read() == 0 {
                    // We are done with the instruction.
                    *ctx = None;
                } else {
                    medium.cs_select.pull(false); // pull line down
                    medium.mosi.pull(p.read().unwrap());
                }  
            },
            InstrVar::Wake(wake) => {
                wake.set();
                *ctx = None;
            }
        }
    }
    



    if ctx.is_none() {
        medium.cs_select.pull(true); // Pull the CS line HIGH.
    }

}




impl Default for SpiMaster<Disconnected> {
    fn default() -> Self {
        Self::new()
    }
}