use bitflags::bitflags;
use embedded_hal::{blocking::spi::Transfer, digital::v2::OutputPin, Pwm, PwmPin};
use feather_m0 as hal;
use hal::{
    clock::GenericClockController,
    pwm::{Channel, Pwm0, Pwm1, Pwm2, Pwm3},
    time::Hertz,
};
use solenoids::{Actuator, BasicActuator, InputArray, InputData};

pub struct Solenoids<B: Transfer<u8>, P: OutputPin, S: Actuator> {
    spi_bus: B,
    load_pin: P,
    pwm_controller: crate::pwm::Controller,
    input_array: InputArray,

    pin1: BasicActuator,
}

impl<B: Transfer<u8>, P: OutputPin, S: Actuator> Solenoids<B, P, S> {
    pub fn new(spi_bus: B, load_pin: P, pwm_controller: crate::pwm::Controller) -> Self {
        Self {
            spi_bus,
            load_pin,
            pwm_controller,
            input_array: InputArray::new(),
            solenoids: None,
        }
    }

    pub fn add_solenoids<F: FnOnce(&InputArray, &mut crate::pwm::Controller) -> S>(
        &mut self,
        f: F,
    ) {
        self.solenoids = Some(f(&self.input_array, &mut self.pwm_controller));
    }

    pub fn update_states(&mut self) {
        self.read_inputs();
    }

    fn read_inputs(&mut self) {
        self.load_pin.set_low().unwrap_or_default();
        let mut buf = [0u8; 2];
        self.spi_bus.transfer(&mut buf);
        self.load_pin.set_high().unwrap_or_default();

        self.input_array.update(u16::from_le_bytes(buf))
    }
}
