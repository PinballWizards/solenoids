use crate::pwm::{Configuration, State};
use crate::{pwm, Actuator, InputConfig, InputData, SingleInput};

pub struct Basic {
    input_config: InputConfig<SingleInput>,
    pwm_config: pwm::Configuration,
}

impl Actuator<SingleInput> for Basic {
    fn new(input_config: InputConfig<SingleInput>, pwm_config: Configuration) -> Self {
        Self {
            input_config,
            pwm_config,
        }
    }

    fn input_config(&self) -> &InputConfig<SingleInput> {
        &self.input_config
    }

    fn pwm_config(&self) -> &Configuration {
        &self.pwm_config
    }

    fn update_state(&self, data: &InputData<SingleInput>, curr_state: State) -> State {
        if data.is_input1_high() {
            State {
                enabled: true,
                duty_cycle: core::u32::MAX,
            }
        } else {
            State {
                enabled: false,
                duty_cycle: curr_state.duty_cycle,
            }
        }
    }
}
