use feather_m0 as hal;

use crate::pwm::{AllChannels, ChannelPin, Tcc2Channels};
use hal::pwm::Pwm2;
use solenoids::{Actuator, BasicActuator, InputArray, InputType};

pub struct Solenoids<'a, 'b> {
    // Whenever you add a new pin here you must also add it to the new() and
    // update_states() functions!!!
    pin1: BasicActuator<'a, ChannelPin<'b, Pwm2>>,
}

impl<'a, 'b> Solenoids<'a, 'b> {
    pub fn new(input_array: &'a mut InputArray, channels: AllChannels<'b>) -> Self {
        let input1 = input_array.get_input(InputType::Single).unwrap();
        let channel_pin1 = channels.2.cc0;
        Self {
            pin1: BasicActuator::new(channel_pin1, input1),
        }
    }

    pub fn update_states(&mut self) {
        self.pin1.update_state();
    }
}
