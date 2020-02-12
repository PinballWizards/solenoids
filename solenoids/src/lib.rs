#![cfg_attr(not(feature = "std"), no_std)]

use core::cell::Cell;
use embedded_hal::PwmPin;
use heapless::{consts::*, Vec};

pub mod controller;

#[derive(Debug)]
pub enum Error {
    TooManyInputs,
}

pub enum InputType {
    Single,
    Double,
    Triple,
}

pub struct InputData<'a> {
    location: &'a Cell<u16>,
    start_offset: u16,
    _type: InputType,
}

impl<'a> InputData<'a> {
    pub fn input1_is_high(&self) -> Option<bool> {
        match self._type {
            InputType::Single | InputType::Double | InputType::Triple => {
                Some(self.location.get() & (1 << (0 + self.start_offset)) != 0)
            }
        }
    }

    pub fn input2_is_high(&self) -> Option<bool> {
        match self._type {
            InputType::Single => None,
            InputType::Double | InputType::Triple => {
                Some(self.location.get() & (1 << (1 + self.start_offset)) != 0)
            }
        }
    }

    pub fn input3_is_high(&self) -> Option<bool> {
        match self._type {
            InputType::Single | InputType::Double => None,
            InputType::Triple => Some(self.location.get() & (1 << (2 + self.start_offset)) != 0),
        }
    }
}

// (start_offset, len)
type InputLayout = Vec<(u8, u8), U6>;

pub struct InputArray {
    raw: Cell<u16>,
    layout: Cell<InputLayout>,
}

impl InputArray {
    pub fn new() -> Self {
        Self {
            raw: Cell::new(0),
            layout: Cell::new(Vec::new()),
        }
    }

    pub fn update(&self, data: u16) {
        self.raw.replace(data);
    }

    pub fn get_input(&self, input: InputType) -> Result<InputData, Error> {
        let mut layout = self.layout.take();
        let size_used = layout.iter().map(|t| t.1).sum();
        if size_used >= 16 {
            self.layout.set(layout);
            return Err(Error::TooManyInputs);
        }
        let push_res = layout.push((
            size_used,
            match input {
                InputType::Single => 1,
                InputType::Double => 2,
                InputType::Triple => 3,
            },
        ));
        self.layout.set(layout);

        if push_res.is_err() {
            return Err(Error::TooManyInputs);
        }

        Ok(InputData {
            location: &self.raw,
            start_offset: size_used as u16,
            _type: input,
        })
    }
}

pub trait Actuator {
    fn update_state(&mut self);
}

/// BasicActuator checks input pin 1 for state. The actuator will be turned on at max
/// duty cycle when input pin 1 is high.
pub struct BasicActuator<'a, P: PwmPin> {
    input_data: InputData<'a>,
    output_pin: P,
}

impl<'a, P: PwmPin> BasicActuator<'a, P> {
    pub fn new(mut output_pin: P, input_data: InputData<'a>) -> Self {
        output_pin.disable();
        Self {
            input_data,
            output_pin,
        }
    }
}

impl<P: PwmPin> Actuator for BasicActuator<'_, P>
where
    P::Duty: core::ops::Div<Output = P::Duty>,
{
    fn update_state(&mut self) {
        if self.input_data.input1_is_high().unwrap() {
            self.output_pin.set_duty(self.output_pin.get_max_duty());
            self.output_pin.enable();
        } else {
            self.output_pin.disable();
        }
    }
}

pub struct TriStateActuator<'a, P: PwmPin> {
    input_data: InputData<'a>,
    output_pin: P,
}

impl<'a, P: PwmPin> TriStateActuator<'a, P> {
    pub fn new(mut output_pin: P, input_data: InputData<'a>) -> Self {
        output_pin.disable();
        Self {
            output_pin,
            input_data,
        }
    }
}

impl<P: PwmPin> Actuator for TriStateActuator<'_, P>
where
    P::Duty: core::ops::Div<u32, Output = P::Duty>,
{
    fn update_state(&mut self) {
        if self.input_data.input2_is_high().unwrap() {
            self.output_pin.set_duty(self.output_pin.get_max_duty() / 2);
            self.output_pin.enable();
        } else if self.input_data.input1_is_high().unwrap() {
            self.output_pin.set_duty(self.output_pin.get_max_duty());
            self.output_pin.enable();
        } else {
            self.output_pin.disable();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{InputArray, InputType};

    #[test]
    fn borrow_checking() {
        let inputs = InputArray::new();
        let data = match inputs.get_input(InputType::Single) {
            Ok(data) => data,
            Err(e) => panic!("failed to get data: {:?}", e),
        };

        // core::mem::drop(inputs);

        data.input1_is_high();
    }

    #[test]
    fn adding_single_input() {
        let inputs = InputArray::new();
        let data = match inputs.get_input(InputType::Single) {
            Ok(data) => data,
            Err(e) => panic!("failed to get data: {:?}", e),
        };

        assert_eq!(data.input1_is_high().is_some(), true);
        assert!(data.input2_is_high().is_none());
        assert!(data.input3_is_high().is_none());

        assert_eq!(data.input1_is_high().unwrap(), false);
        inputs.update(1);
        assert_eq!(data.input1_is_high().unwrap(), true);
    }

    #[test]
    fn add_double_input() {
        let inputs = InputArray::new();
        let data = match inputs.get_input(InputType::Double) {
            Ok(data) => data,
            Err(e) => panic!("failed to get data: {:?}", e),
        };

        assert!(data.input1_is_high().is_some());
        assert!(data.input2_is_high().is_some());
        assert!(data.input3_is_high().is_none());

        assert_eq!(data.input1_is_high().unwrap(), false);
        assert_eq!(data.input2_is_high().unwrap(), false);
        inputs.update(1);
        assert_eq!(data.input1_is_high().unwrap(), true);
        assert_eq!(data.input2_is_high().unwrap(), false);

        inputs.update(0);

        assert_eq!(data.input1_is_high().unwrap(), false);
        assert_eq!(data.input2_is_high().unwrap(), false);
        inputs.update(1 << 1);
        assert_eq!(data.input1_is_high().unwrap(), false);
        assert_eq!(data.input2_is_high().unwrap(), true);
    }

    #[test]
    fn add_single_double_inputs() {
        let inputs = InputArray::new();
        let single_data = match inputs.get_input(InputType::Single) {
            Ok(d) => d,
            Err(e) => panic!("failed to get data: {:?}", e),
        };
        let double_data = match inputs.get_input(InputType::Double) {
            Ok(d) => d,
            Err(e) => panic!("failed to get data: {:?}", e),
        };

        inputs.update(1 << 0);
        assert!(single_data.input1_is_high().unwrap());
        assert!(!double_data.input1_is_high().unwrap());
        assert!(!double_data.input2_is_high().unwrap());

        inputs.update(1 << 1);
        assert!(!single_data.input1_is_high().unwrap());
        assert!(double_data.input1_is_high().unwrap());
        assert!(!double_data.input2_is_high().unwrap());

        inputs.update(1 << 2);
        assert!(!single_data.input1_is_high().unwrap());
        assert!(!double_data.input1_is_high().unwrap());
        assert!(double_data.input2_is_high().unwrap());

        inputs.update(1 << 0 | 1 << 1);
        assert!(single_data.input1_is_high().unwrap());
        assert!(double_data.input1_is_high().unwrap());
        assert!(!double_data.input2_is_high().unwrap());

        inputs.update(1 << 0 | 1 << 1 | 1 << 2);
        assert!(single_data.input1_is_high().unwrap());
        assert!(double_data.input1_is_high().unwrap());
        assert!(double_data.input2_is_high().unwrap());
    }
}
