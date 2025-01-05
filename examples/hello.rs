#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32wb_hal as hal;
use defmt_rtt as _;
use rt::entry;

#[entry]
fn main() -> ! {
    defmt::info!("Hello world");
    loop {}
}
