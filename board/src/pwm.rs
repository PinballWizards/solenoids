use core::marker::PhantomData;
use core::ptr::NonNull;
use embedded_hal::{Pwm, PwmPin};
use feather_m0 as hal;
use hal::{
    clock::GenericClockController,
    pac::{PM, TC3, TCC0, TCC1, TCC2},
    pwm::{self, Pwm0, Pwm1, Pwm2, Pwm3},
    time::Hertz,
};

#[derive(Clone, Copy)]
enum Channel {
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

pub type AllChannels<'a> = (
    Tcc0Channels<'a>,
    Tcc1Channels<'a>,
    Tcc2Channels<'a>,
    Tc3Channels<'a>,
);

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

    pub fn make_channels(&mut self) -> AllChannels {
        (
            Tcc0Channels {
                cc0: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc0) },
                    channel: pwm::Channel::_0.into(),
                    phantom: PhantomData,
                }
                .into(),
                cc1: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc0) },
                    channel: pwm::Channel::_1.into(),
                    phantom: PhantomData,
                }
                .into(),
                cc2: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc0) },
                    channel: pwm::Channel::_2.into(),
                    phantom: PhantomData,
                }
                .into(),
                cc3: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc0) },
                    channel: pwm::Channel::_3.into(),
                    phantom: PhantomData,
                }
                .into(),
            },
            Tcc1Channels {
                cc0: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc1) },
                    channel: Channel::_0,
                    phantom: PhantomData,
                }
                .into(),
                cc1: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc1) },
                    channel: Channel::_1,
                    phantom: PhantomData,
                }
                .into(),
            },
            Tcc2Channels {
                cc0: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc2) },
                    channel: Channel::_0,
                    phantom: PhantomData,
                }
                .into(),
                cc1: ChannelPin {
                    controller: unsafe { NonNull::new_unchecked(&mut self.tcc2) },
                    channel: Channel::_1,
                    phantom: PhantomData,
                }
                .into(),
            },
            Tc3Channels { cc0: &mut self.tc3 },
        )
    }
}

pub struct Tcc0Channels<'a> {
    pub cc0: ChannelPin<'a, Pwm0>,
    pub cc1: ChannelPin<'a, Pwm0>,
    pub cc2: ChannelPin<'a, Pwm0>,
    pub cc3: ChannelPin<'a, Pwm0>,
}

pub struct Tcc1Channels<'a> {
    pub cc0: ChannelPin<'a, Pwm1>,
    pub cc1: ChannelPin<'a, Pwm1>,
}

pub struct Tcc2Channels<'a> {
    pub cc0: ChannelPin<'a, Pwm2>,
    pub cc1: ChannelPin<'a, Pwm2>,
}

pub struct Tc3Channels<'a> {
    pub cc0: &'a mut Pwm3,
}

pub struct ChannelPin<'a, P> {
    controller: NonNull<P>,
    channel: Channel,
    phantom: PhantomData<&'a ()>,
}

impl<P: Pwm<Channel = pwm::Channel>> PwmPin for ChannelPin<'_, P> {
    type Duty = P::Duty;

    fn disable(&mut self) {
        unsafe { self.controller.as_mut().disable(self.channel.into()) };
    }

    fn enable(&mut self) {
        unsafe { self.controller.as_mut().enable(self.channel.into()) };
    }

    fn get_duty(&self) -> Self::Duty {
        unsafe { self.controller.as_ref().get_duty(self.channel.into()) }
    }

    fn get_max_duty(&self) -> Self::Duty {
        unsafe { self.controller.as_ref().get_max_duty() }
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        unsafe { self.controller.as_mut().set_duty(self.channel.into(), duty) };
    }
}
