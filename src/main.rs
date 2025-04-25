use bitvec::{order::Msb0, vec::BitVec};
use i2cem::{core::Register, i2c::{I2CSlave, Master}};


fn main() {


    fn bmi_reg0x21() -> Vec<u8> {
        vec![ 0x21, 0x22 ]
    }

    fn bmi2_reg0x32() -> Vec<u8> {
        vec![ 0x48, 0x29 ]
    }


    let mut slave = I2CSlave::new(0x68);
    let mut slave2 = I2CSlave::new(0x32);
    slave.create_register(0x12, Register::new_writeable());
    slave2.create_register(0x29, Register::new_read_only(bmi2_reg0x32));



    let mut master = Master::new();
    master.add_device(slave);
    master.add_device(slave2);

    // let values= master.read_block(0x68, 0x12, 2);
    // assert_eq!(&*values, [0x00, 0x00]);

    master.write_block(0x68, 0x12, vec![ 0x23, 0x48 ]);

    let values= master.read_block(0x68, 0x12, 2);
    assert_eq!(&*values, [0x23, 0x48]);

    // let values= master.read_block(0x68, 0x12, 2);
    // assert_eq!(&*values, [0x21, 0x22]);


    // // println!("Values: {:?}", values.into_iter().map(|v| format!("{:#x}", v)).collect::<Vec<_>>());


    // slave.write_byte(0b11010000, LineCondition::Start); // Write the Slave I2C ID and the R/W bit set to zero.

    // slave.write_byte(0b00010010, LineCondition::InProgress);
    // slave.write_byte(0b11010001, LineCondition::InProgress); // Request the r

    // println!("[Master] Now reading...");
    
    // let byte = slave.read_byte().unwrap();
    // println!("[Master] Read value {:#x}", byte);
    // slave.write_bit(false, LineCondition::InProgress);
    // let byte = slave.read_byte().unwrap();
    // println!("[Master] Read value {:#x}", byte);
    // println!("VALUE: {:?}", slave.state);
    // slave.write_bit(true, LineCondition::Stop);

    

    // assert_eq!(slave.read_bit(), Some(false));
    // println!("BV: {:?}", bv);

    // slave.recv_byte();
}
