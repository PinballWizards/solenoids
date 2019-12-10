#![no_std]
#![no_main]

extern crate atsamd21g18a as device;
extern crate cortex_m;
extern crate cortex_m_semihosting;
extern crate feather_m0 as hal;
extern crate panic_halt;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::prelude::*;

use hal::entry;

mod sysclock;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    unsafe {
        device::NVIC::get_priority(interrupt::USB);
        core.NVIC.set_priority(interrupt::USB, 1);
        device::NVIC::unmask(interrupt::USB);
    }

    loop {}
}

#[interrupt]
fn USB() {
    device::NVIC::unpend(interrupt::USB);
}
