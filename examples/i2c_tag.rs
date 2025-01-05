//! Scans for an NFC tag

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate stm32wb_hal as hal;
use panic_probe as _;
use defmt_rtt as _;

use crate::hal::i2c::I2c;
use crate::hal::prelude::*;
use crate::rt::entry;
use crate::rt::ExceptionFrame;

use embedded_hal::digital::v2::InputPin; // Import the trait

#[entry]
fn main() -> ! {
    defmt::info!("STM32WB55 i2c scanner");

    let dp = hal::stm32::Peripherals::take().unwrap();

    // Use default clock frequency of 4 MHz running from MSI
    let mut rcc = dp.RCC.constrain();

    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut gpioa = dp.GPIOA.split(&mut rcc);

    let gpo = gpioa.pa6.into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);

    let mut i2c1 = dp.I2C1;
    let scl = gpiob
        .pb8
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut scl = scl.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

    let sda = gpiob
        .pb9
        .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut sda = sda.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

    const USER_MEMORY_ADDRESS :u8 = 0x53;
    // const SYS_MEMORY_ADDRESS :u8 = 0x57;
    const MBOX_UPPER_ADDRESS :u8 = 0x20;
    const MBOX_LOWER_ADDRESS :u8 = 0x08;
    
    defmt::info!("Scanning addresses...");

    loop {
        while gpo.is_high().unwrap() {}

        // Use fresh I2C peripheral on the each iteration
        let mut i2c = I2c::i2c1(i2c1, (scl, sda), 100.khz(), &mut rcc);

        // Set start of mailbox address
        let mbox_addr: [u8; 2] = [MBOX_UPPER_ADDRESS, MBOX_LOWER_ADDRESS];
        let mut mbox_data: [u8; 256] = [0x0; 256];
        if let Ok(_) = i2c.write(USER_MEMORY_ADDRESS, &mbox_addr) {
            defmt::info!("Found a device with address 0x{:02x}", USER_MEMORY_ADDRESS);

            // the stm32 hals only support up to 255-byte transfers.  Break the transfer in two
            // to read the full 256 supported by the tag
            if let Ok(_) = i2c.read(USER_MEMORY_ADDRESS, &mut mbox_data[0..128]) {
                defmt::info!("Read mbox data OK (lower 128 bytes)");
            }
        }

        if let Ok(_) = i2c.write(USER_MEMORY_ADDRESS + 128, &mbox_addr) {
            if let Ok(_) = i2c.read(USER_MEMORY_ADDRESS, &mut mbox_data[128..256]) {
                defmt::info!("Read mbox data OK (upper 128 bytes)");
            }
        }


        // Decompose the I2C peripheral to re-build it again on the next iteration
        let (i2c, (scl_pin, sda_pin)) = i2c.free();
        i2c1 = i2c;
        scl = scl_pin;
        sda = sda_pin;
        defmt::info!("--- Done scanning --- ");
    }
}

#[exception]
#[allow(non_snake_case)]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
