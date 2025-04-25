use std::sync::atomic::{AtomicBool, Ordering};

use super::clock::Clock;

pub struct LiveWire {
    signal: AtomicBool
}

impl LiveWire {
    pub fn new() -> Self {
        Self {
            signal: AtomicBool::default()
        }
    }
    pub fn flip(&self) {
        self.signal.fetch_not(Ordering::Relaxed);
    }
    pub fn pull(&self, signal: bool) {
        self.signal.store(signal, Ordering::Relaxed);
    }
    pub fn read(&self) -> bool {
        self.signal.load(Ordering::Relaxed)
    }
}

impl Default for LiveWire {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SpiMedium {
    pub mosi: LiveWire,
    pub miso: LiveWire,
    pub cs_select: LiveWire,
    pub kill: LiveWire,
    pub clock: Clock
}