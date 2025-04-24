# I2CEM
Emulates various communication protocols like I2C.


## I2C Example
```rust
const REAL_TEMP: f32 = 0.1234;

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

```