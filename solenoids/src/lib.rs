#![cfg_attr(not(feature = "std"), no_std)]

use core::cell::Cell;
use embedded_hal::PwmPin;
use heapless::{consts::*, Vec};

#[derive(Debug)]
pub enum Error {
    TooManyInputs,
}

pub enum InputType {
    Single,
    Double,
    Triple,
}

#[derive(Clone)]
struct InputData {
    start_offset: u16,
    _type: InputType,
}

pub struct InputRead(bool, bool, bool);

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
            start_offset: size_used as u16,
            _type: input,
        })
    }

    pub fn read(&self, input_data: &InputData) -> InputRead {
        InputRead(
            self.raw.get() & (1 << (0 + input_data.start_offset)) != 0,
            self.raw.get() & (1 << (1 + input_data.start_offset)) != 0,
            self.raw.get() & (1 << (2 + input_data.start_offset)) != 0,
        )
    }
}

pub trait Actuator<P: PwmPin> {
    fn update_state(&mut self, input_array: &InputArray, pwm_pin: &mut P);
}

/// BasicActuator checks input pin 1 for state. The actuator will be turned on at max
/// duty cycle when input pin 1 is high.
pub struct BasicActuator<'a> {
    input_data: InputData,
}

impl<'a> BasicActuator<'a> {
    pub fn new(input_data: InputData) -> Self {
        Self { input_data }
    }
}

impl<P: PwmPin> Actuator for BasicActuator<'_>
where
    P::Duty: core::ops::Div<Output = P::Duty>,
{
    fn update_state(&self, input_array: &InputArray, output_pin: &mut P) {
        let res = input_array.read(&self.input_data);
        if res.0 {
            output_pin.set_duty(output_pin.get_max_duty());
            output_pin.enable();
        } else {
            output_pin.disable();
        }
    }
}

pub struct TriStateActuator {
    input_data: InputData,
}

impl TriStateActuator {
    pub fn new(input_data: InputData) -> Self {
        Self { input_data }
    }
}

impl<P: PwmPin> Actuator for TriStateActuator
where
    P::Duty: core::ops::Div<u32, Output = P::Duty>,
{
    fn update_state(&self, input_array: &InputArray, output_pin: &mut P) {
        let res = input_array.read(&self.input_data);
        if res.1 {
            self.output_pin.set_duty(self.output_pin.get_max_duty() / 2);
            self.output_pin.enable();
        } else if res.0 {
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
