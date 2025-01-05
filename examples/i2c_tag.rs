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

    let button = gpiob.pb2.into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr);
    while button.is_high().unwrap() {
    }
    defmt::info!("Button is {}", button.is_high());

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
    const SYS_MEMORY_ADDRESS :u8 = 0x57;
    defmt::info!("Scanning addresses...");

    for address in [USER_MEMORY_ADDRESS, SYS_MEMORY_ADDRESS] {
        // Use fresh I2C peripheral on the each iteration
        let mut i2c = I2c::i2c1(i2c1, (scl, sda), 100.khz(), &mut rcc);

        let mut byte: [u8; 1] = [0; 1];
        if let Ok(_) = i2c.read(address, &mut byte) {
            defmt::info!("Found a device with address 0x{:02x}", address);
        } else {
            defmt::info!("Device with address 0x{:02x} NOT FOUND", address);
        }

        // Decompose the I2C peripheral to re-build it again on the next iteration
        let (i2c, (scl_pin, sda_pin)) = i2c.free();
        i2c1 = i2c;
        scl = scl_pin;
        sda = sda_pin;
    }
    defmt::info!("Done scanning");

    loop {
    }
}

#[exception]
#[allow(non_snake_case)]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
