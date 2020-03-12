#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use heapless::{consts::*, Vec};

pub mod actuators;
pub mod pwm;

#[derive(Debug)]
pub enum Error {
    TooManyInputs,
    InvalidInputType,
}

pub trait InputType {
    fn new() -> Self;
    fn size(&self) -> u8;
}

pub struct SingleInput;
impl InputType for SingleInput {
    fn new() -> Self {
        SingleInput
    }

    fn size(&self) -> u8 {
        0
    }
}

pub struct DualInput;
impl InputType for DualInput {
    fn new() -> Self {
        DualInput
    }

    fn size(&self) -> u8 {
        1
    }
}

pub struct TriInput;
impl InputType for TriInput {
    fn new() -> Self {
        TriInput
    }

    fn size(&self) -> u8 {
        2
    }
}

#[derive(Clone)]
pub struct InputConfig<I: InputType> {
    start_offset: u16,
    input_type: I,
}

pub struct InputData<I: InputType> {
    start_offset: u16,
    data: u16,
    _type: PhantomData<I>,
}

impl<I: InputType> InputData<I> {
    fn new(config: &InputConfig<I>, data: u16) -> Self {
        Self {
            start_offset: config.start_offset,
            data,
            _type: PhantomData,
        }
    }

    pub fn is_input1_high(&self) -> bool {
        self.data & (1 << self.start_offset) != 0
    }
}

impl InputData<DualInput> {
    pub fn is_input2_high(&self) -> bool {
        self.data & (1 << (1 + self.start_offset)) != 0
    }
}

impl InputData<TriInput> {
    pub fn is_input2_high(&self) -> bool {
        self.data & (1 << (1 + self.start_offset)) != 0
    }

    pub fn is_input3_high(&self) -> bool {
        self.data & (1 << (2 + self.start_offset)) != 0
    }
}

// (start_offset, len)
type InputLayout = Vec<(u8, u8), U6>;

pub struct InputArray {
    raw: u16,
    layout: InputLayout,
}

impl InputArray {
    pub fn new() -> Self {
        Self {
            raw: 0,
            layout: Vec::new(),
        }
    }

    pub fn update(&mut self, data: u16) {
        self.raw = data;
    }

    fn get_input<I: InputType>(&mut self, input: I) -> Result<InputConfig<I>, Error> {
        let size_used = self.layout.iter().map(|t| t.1).sum();
        if size_used >= 16 {
            return Err(Error::TooManyInputs);
        }

        match self.layout.push((size_used, input.size())) {
            Err(_) => return Err(Error::TooManyInputs),
            _ => (),
        };

        Ok(InputConfig {
            start_offset: size_used as u16,
            input_type: input,
        })
    }

    pub fn read<I: InputType>(&self, input_config: &InputConfig<I>) -> InputData<I> {
        InputData::new(input_config, self.raw)
    }

    pub fn make_actuator<I: InputType, A: Actuator<I>>(
        &mut self,
        channel_config: pwm::Configuration,
    ) -> Result<A, Error> {
        Ok(A::new(self.get_input(I::new())?, channel_config))
    }
}

/// BasicActuator checks input pin 1 for state. The actuator will be turned on at max
/// duty cycle when input pin 1 is high.
pub trait Actuator<I>
where
    I: InputType,
{
    fn new(input_config: InputConfig<I>, pwm_config: pwm::Configuration) -> Self;
    fn input_config(&self) -> &InputConfig<I>;
    fn pwm_config(&self) -> &pwm::Configuration;
    fn update_state(&self, data: &InputData<I>, curr_state: pwm::State) -> pwm::State;
}

#[cfg(test)]
mod test {
    use crate::{DualInput, InputArray, InputType, SingleInput};

    #[test]
    fn borrow_checking() {
        let mut inputs = InputArray::new();
        let data = match inputs.get_input(SingleInput) {
            Ok(data) => data,
            Err(e) => panic!("failed to get data: {:?}", e),
        };

        // core::mem::drop(inputs);

        data.input1_is_high();
    }

    #[test]
    fn adding_single_input() {
        let mut inputs = InputArray::new();
        let data = match inputs.get_input(SingleInput) {
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
        let data = match inputs.get_input(DualInput) {
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
        let single_data = match inputs.get_input(SingleInput) {
            Ok(d) => d,
            Err(e) => panic!("failed to get data: {:?}", e),
        };
        let double_data = match inputs.get_input(DualInput) {
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
