
pub mod core;
pub mod i2c;
pub mod spi;


#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};
    use crate::{core::Register, i2c::{I2CSlave, Master}, spi::{master::{Disconnected, SpiMaster}, slave::SpiSlave}};


    #[test]
    pub fn test_bus_read() {
        fn bmi_reg0x21() -> Vec<u8> {
            vec![ 0x21, 0x22 ]
        }
    
        fn bmi2_reg0x32() -> Vec<u8> {
            vec![ 0x48, 0x29 ]
        }
    
    
        let mut slave = I2CSlave::new(0x68);
        let mut slave2 = I2CSlave::new(0x32);
        slave.create_register(0x12, Register::new_read_only(bmi_reg0x21));
        slave2.create_register(0x29, Register::new_read_only(bmi2_reg0x32));
    
    
    
        let mut master = Master::new();
        master.add_device(slave);
        master.add_device(slave2);
    
        let values= master.read_block(0x68, 0x12, 2);
        assert_eq!(&*values, [0x21, 0x22]);
    
        let values= master.read_block(0x32, 0x29, 2);
        assert_eq!(&*values, [0x48, 0x29]);
    
        let values= master.read_block(0x68, 0x12, 2);
        assert_eq!(&*values, [0x21, 0x22]);
    
    }

    #[test]
    pub fn test_read_bmi270() {

        const REAL_TEMP: f32 = 0.1234;


        // println!("Temperature: {:?}", temperature);

        fn bmi_reg_temp_lsb() -> Vec<u8> {
            vec![ ((REAL_TEMP * 10000.0) as u16).to_be_bytes()[0] ]
        }
    
        fn bmi_reg_temp_msb() -> Vec<u8> {
            vec![ ((REAL_TEMP * 10000.0) as u16).to_be_bytes()[1] ]
        }
    
    
        let mut slave = I2CSlave::new(0x68);
        slave.create_register(0x22, Register::new_read_only(bmi_reg_temp_lsb));
        slave.create_register(0x23, Register::new_read_only(bmi_reg_temp_msb));

    
        let mut master = Master::new();
        master.add_device(slave);
   
        let fst = master.read_block(0x68, 0x22, 1)[0];
        let snd = master.read_block(0x68, 0x23, 1)[0];
        // assert_eq!(&*values, [0x21, 0x22]);


        let reconstructed = ((snd as u16) | ((fst as u16) << 8)) as f32 / 10000.0;

        assert_eq!(reconstructed, REAL_TEMP);
    
    }

    #[test]
    pub fn basic_write() {
        let mut slave = I2CSlave::new(0x68);
        slave.create_register(0x12, Register::new_writeable());


        // Make a master.
        let mut master = Master::new();
        master.add_device(slave);

        // Write the bytes.
        master.write_block(0x68, 0x12, vec![ 0x23, 0x48 ]);
    
        // Verify they were written.
        let values= master.read_block(0x68, 0x12, 2);
        assert_eq!(&*values, [0x23, 0x48]);
    
        let values= master.read_block(0x68, 0x12, 2);
        assert_eq!(&*values, [0x23, 0x48]);

        // Write the bytes.
        master.write_block(0x68, 0x12, vec![ 0x24, 0x58 ]);

        // Verify they were written.
        let values= master.read_block(0x68, 0x12, 2);
        assert_eq!(&*values, [0x24, 0x58]);
    

    
    }


    #[test]
    pub fn basic_spi_write() {
        let master: SpiMaster<Disconnected> = SpiMaster::new();
        let slave: SpiSlave<Disconnected> = SpiSlave::new(HashMap::from([
            (0x15, Register::new_writeable())
        ]));

        let (master, _) = master.connect(slave, Duration::from_millis(1));

        master.write_register(0x15, vec![ 0x21 ]);
        assert_eq!(master.read_register(0x15, 1), vec![ 0x21 ]);

    }

    #[test]
    pub fn basic_spi_write_readonly_register() {

        fn reg_fn() -> Vec<u8> {
            vec![ 0x21, 0x59 ]
        }

        let master: SpiMaster<Disconnected> = SpiMaster::new();
        let slave: SpiSlave<Disconnected> = SpiSlave::new(HashMap::from([
            (0x15, Register::new_read_only(reg_fn))
        ]));

        

        let (master, _) = master.connect(slave, Duration::from_millis(1));

        assert_eq!(master.read_register(0x15, 2), vec![ 0x21, 0x59 ]);

    }
}