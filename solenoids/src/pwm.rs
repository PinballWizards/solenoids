use embedded_hal::{Pwm, PwmPin};
use feather_m0 as hal;
use hal::{
    clock::GenericClockController,
    pac::{PM, TC3, TCC0, TCC1, TCC2},
    pwm::{self, Pwm0, Pwm1, Pwm2, Pwm3},
    time::Hertz,
};

pub enum Configuration {
    Tcc0(Channel),
    Tcc1(Channel),
    Tcc2(Channel),
    Tc3,
}

pub struct State {
    pub enabled: bool,
    pub duty_cycle: u32,
}

#[derive(Clone, Copy)]
pub enum Channel {
    _0,
    _1,
    _2,
    _3,
}

impl From<pwm::Channel> for Channel {
    fn from(c: pwm::Channel) -> Self {
        match c {
            pwm::Channel::_0 => Channel::_0,
            pwm::Channel::_1 => Channel::_1,
            pwm::Channel::_2 => Channel::_2,
            pwm::Channel::_3 => Channel::_3,
        }
    }
}

impl Into<pwm::Channel> for Channel {
    fn into(self) -> pwm::Channel {
        match self {
            Channel::_0 => pwm::Channel::_0,
            Channel::_1 => pwm::Channel::_1,
            Channel::_2 => pwm::Channel::_2,
            Channel::_3 => pwm::Channel::_3,
        }
    }
}

pub struct Controller {
    tcc0: Pwm0,
    tcc1: Pwm1,
    tcc2: Pwm2,
    tc3: Pwm3,
}

impl Controller {
    pub fn new<F: Into<Hertz> + Copy>(
        clocks: &mut GenericClockController,
        period: F,
        tcc0: TCC0,
        tcc1: TCC1,
        tcc2: TCC2,
        tc3: TC3,
        pm: &mut PM,
    ) -> Self {
        let gclk0 = clocks.gclk0();
        let tcc0tcc1clock = clocks.tcc0_tcc1(&gclk0).unwrap();
        let tcc2tc3clock = clocks.tcc2_tc3(&gclk0).unwrap();
        Self {
            tcc0: Pwm0::new(&tcc0tcc1clock, period, tcc0, pm),
            tcc1: Pwm1::new(&tcc0tcc1clock, period, tcc1, pm),
            tcc2: Pwm2::new(&tcc2tc3clock, period, tcc2, pm),
            tc3: Pwm3::new(&tcc2tc3clock, period, tc3, pm),
        }
    }

    pub fn tcc0_channel(&mut self, channel: Channel) -> ChannelPin<Pwm0> {
        ChannelPin {
            controller: &mut self.tcc0,
            channel,
        }
    }

    pub fn tcc1_channel(&mut self, channel: Channel) -> ChannelPin<Pwm1> {
        ChannelPin {
            controller: &mut self.tcc1,
            channel,
        }
    }

    pub fn tcc2_channel(&mut self, channel: Channel) -> ChannelPin<Pwm2> {
        ChannelPin {
            controller: &mut self.tcc2,
            channel,
        }
    }

    pub fn tc3_channel(&mut self) -> &mut Pwm3 {
        &mut self.tc3
    }
}

pub struct ChannelPin<'a, P: Pwm> {
    controller: &'a mut P,
    channel: Channel,
}

impl<P: Pwm<Channel = pwm::Channel>> PwmPin for ChannelPin<'_, P> {
    type Duty = P::Duty;

    fn disable(&mut self) {
        self.controller.disable(self.channel.into());
    }

    fn enable(&mut self) {
        self.controller.enable(self.channel.into());
    }

    fn get_duty(&self) -> Self::Duty {
        self.controller.get_duty(self.channel.into())
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.controller.get_max_duty()
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        self.controller.set_duty(self.channel.into(), duty);
    }
}
