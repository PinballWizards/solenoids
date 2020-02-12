use embedded_hal::{blocking::spi::Transfer, digital::v2::OutputPin};

use crate::{InputArray, InputData, InputType};

pub trait Controllable {
    fn load_data(&mut self);
}

pub struct ControllerBuilder;

impl ControllerBuilder {
    pub fn new_spi<B: Transfer<u8>, P: OutputPin>(
        bus: B,
        load_pin: P,
    ) -> SPIControllerBuilder<B, P> {
        SPIControllerBuilder {
            bus,
            load_pin,
            input_array: InputArray::new(),
        }
    }
}

pub struct SPIControllerBuilder<B: Transfer<u8>, P: OutputPin> {
    bus: B,
    load_pin: P,
    input_array: InputArray,
}

impl<B: Transfer<u8>, P: OutputPin> SPIControllerBuilder<B, P> {
    pub fn build(self) -> Controller<SPIControllerBuilder<B, P>> {
        Controller { controller: self }
    }

    pub fn make_input(&self, input_type: InputType) -> InputData {
        self.input_array
            .get_input(input_type)
            .expect("failed to make input")
    }
}

impl<B: Transfer<u8>, P: OutputPin> Controllable for SPIControllerBuilder<B, P> {
    fn load_data(&mut self) {
        self.load_pin.set_low().unwrap_or_default();

        let mut buf = [0u8; 2];
        self.bus.transfer(&mut buf);

        self.load_pin.set_high().unwrap_or_default();

        self.input_array.update(u16::from_le_bytes(buf));
    }
}

pub struct Controller<C: Controllable> {
    controller: C,
}

impl<C: Controllable> Controllable for Controller<C> {
    fn load_data(&mut self) {
        self.controller.load_data();
    }
}
