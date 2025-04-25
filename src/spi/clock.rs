use rsevents::{AutoResetEvent, Awaitable};
use super::wire::LiveWire;

/// The clock used for SPI communications
pub struct Clock {
    /// Thread wakeup mechanism.
    mechanism: AutoResetEvent,
    /// The clock wire.
    line: LiveWire
}

impl Clock {
    /// Creates a new clock.
    pub fn new() -> Self {
        Self {
            mechanism: AutoResetEvent::new(rsevents::EventState::Unset),
            line: LiveWire::new()
        }
    }
    /// Gets the line value without waiting.
    pub fn get_line_value(&self) -> bool {
        self.line.read()
    }
    /// Gets the clock value but waits for it to be update.
    pub fn get_clock(&self) -> bool {
        self.mechanism.wait();
        self.get_line_value()
    }
    /// Ticks the clock.
    pub fn tick(&self) {
        self.line.flip();
        self.mechanism.set();
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}