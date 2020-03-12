use feather_m0 as hal;

use hal::{
    gpio::{Output, Pa12, Pa2, Pb10, Pb11, PfD, PushPull},
    prelude::*,
    sercom::{SPIMaster4, Sercom4Pad0, Sercom4Pad2, Sercom4Pad3},
};

use solenoids::{
    actuators::Basic,
    pwm::{Channel, Configuration, Controller},
    Actuator, InputArray, InputData, SingleInput,
};

type Bus = SPIMaster4<Sercom4Pad0<Pa12<PfD>>, Sercom4Pad2<Pb10<PfD>>, Sercom4Pad3<Pb11<PfD>>>;
type LoadPin = Pa2<Output<PushPull>>;

pub struct Solenoids {
    pwm: Controller,
    input_array: InputArray,
    bus: Bus,
    load_pin: LoadPin,

    pin1: Basic,
    pin2: Basic,
}

impl Solenoids {
    pub fn new(pwm: Controller, input_bus: Bus, input_load_pin: LoadPin) -> Self {
        let mut input_array = InputArray::new();
        let pin1 = input_array.make_actuator(Configuration::Tc3).unwrap();
        let pin2 = input_array
            .make_actuator(Configuration::Tcc0(Channel::_0))
            .unwrap();
        Self {
            pwm,
            input_array,
            bus: input_bus,
            load_pin: input_load_pin,
            pin1,
            pin2,
        }
    }

    pub fn update_states(&mut self) {
        self.read_inputs();

        self.update_pin1(self.input_array.read(self.pin1.input_config()))
    }

    fn read_inputs(&mut self) {
        self.load_pin.set_low().unwrap();
        let mut buf = [0u8; 2];
        self.bus.transfer(&mut buf).unwrap();
        self.load_pin.set_high().unwrap();

        self.input_array.update(u16::from_le_bytes(buf));
    }

    fn update_pin1(&mut self, data: InputData<SingleInput>) {}
}
