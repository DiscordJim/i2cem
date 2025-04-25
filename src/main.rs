use std::{collections::HashMap, time::Duration};

use i2cem::{
    core::Register,
    spi::{
        master::{Connected, SpiMaster},
        slave::SpiSlave,
    },
};

fn main() {
    let master = SpiMaster::new();
    let slave = SpiSlave::new(HashMap::from([(0xF, Register::new_writeable())]));

    let (master, _slave): (SpiMaster<Connected>, SpiSlave<Connected>) =
        master.connect(slave, Duration::from_millis(5));

    master.write_register(0xF, vec![0x21]);

    assert_eq!(master.read_register(0xF, 1), vec![ 0x21 ]);


    master.disconnect();
}
