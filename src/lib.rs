#![cfg_attr(not(feature = "std"), no_std)]

use core::cell::UnsafeCell;
use embedded_hal::digital::OutputPin;
use embedded_hal::PwmPin;

#[derive(Debug)]
pub enum Error {
    TooManyInputs,
}

pub enum InputType {
    Single,
    Double,
    Triple,
}

pub struct InputData {
    location: *mut u16,
    start_offset: u16,
    _type: InputType,
}

impl InputData {
    pub fn input1_is_high(&self) -> Option<bool> {
        match self._type {
            InputType::Single | InputType::Double | InputType::Triple => {
                Some(unsafe { self.location.read() } & (1 << (0 + self.start_offset)) != 0)
            }
        }
    }

    pub fn input2_is_high(&self) -> Option<bool> {
        match self._type {
            InputType::Single => None,
            InputType::Double | InputType::Triple => {
                Some(unsafe { self.location.read() } & (1 << (1 + self.start_offset)) != 0)
            }
        }
    }

    pub fn input3_is_high(&self) -> Option<bool> {
        match self._type {
            InputType::Single | InputType::Double => None,
            InputType::Triple => {
                Some(unsafe { self.location.read() } & (1 << (2 + self.start_offset)) != 0)
            }
        }
    }
}

// (start_offset, len)
type InputLayout = [(u8, u8); 16];

pub struct InputArray {
    raw: UnsafeCell<u16>,
    layout: InputLayout,
    input_count: UnsafeCell<u16>,
}

impl InputArray {
    pub fn new() -> Self {
        Self {
            raw: UnsafeCell::new(0),
            layout: [(0, 0); 16],
            input_count: UnsafeCell::new(0),
        }
    }

    pub fn update(&mut self, data: u16) {
        unsafe {
            self.raw.get().replace(data);
        }
    }

    pub fn get_input(&mut self, input: InputType) -> Result<InputData, Error> {
        let curr_input_count = unsafe { self.input_count.get().read() } as usize;
        if curr_input_count == 15 {
            return Err(Error::TooManyInputs);
        }

        let size_used = self.layout[0..curr_input_count].iter().map(|t| t.1).sum();
        if size_used >= 16 {
            return Err(Error::TooManyInputs);
        }
        self.layout[curr_input_count].0 = size_used;
        self.layout[curr_input_count].1 = match input {
            InputType::Single => 1,
            InputType::Double => 2,
            InputType::Triple => 3,
        };

        unsafe {
            self.input_count.get().replace(curr_input_count as u16 + 1);
        }

        Ok(InputData {
            location: self.raw.get(),
            start_offset: size_used as u16,
            _type: input,
        })
    }
}

pub trait Actuator<P: PwmPin> {
    fn update_state(&mut self);
}

/// BasicActuator checks input pin 1 for state. The actuator will be turned on at max
/// duty cycle when input pin 1 is high.
pub struct BasicActuator<P: PwmPin> {
    input_data: InputData,
    output_pin: P,
}

impl<P: PwmPin> BasicActuator<P> {
    pub fn new(mut output_pin: P, input_data: InputData) -> Self {
        output_pin.disable();
        Self {
            input_data,
            output_pin,
        }
    }
}

impl<P: PwmPin> Actuator for BasicActuator<P> {
    fn update_state(&mut self) {
        if self.input_data.input1_is_high().unwrap() {
            self.output_pin.set_duty(self.output_pin.get_max_duty());
            self.output_pin.enable();
        } else {
            self.output_pin.disable();
        }
    }
}

pub struct TwoStateActuator<P: PwmPin> {
    input_data: InputData,
    output_pin: P,
}

impl<P: PwmPin> TwoStateActuator<P> {
    pub fn new(mut output_pin: P, input_data: InputData) -> Self {
        output_pin.disable();
        Self {
            output_pin,
            input_data,
        }
    }
}

impl<P: PwmPin> Actuator for TwoStateActuator<P> {
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
    fn adding_single_input() {
        let mut inputs = InputArray::new();
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
        let mut inputs = InputArray::new();
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
        let mut inputs = InputArray::new();
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
