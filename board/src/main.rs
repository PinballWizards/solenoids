#![no_std]
#![no_main]

extern crate panic_halt;

use feather_m0 as hal;
use rtfm;

use hal::{
    clock::GenericClockController,
    delay::Delay,
    gpio::{Output, Pa16, Pa17, Pa5, PushPull},
    pac::Peripherals,
    prelude::*,
};
use palantir::{
    feather_bus as bus,
    messages::{DiscoveryAck, DiscoveryRequest, ReceivedMessage},
    Palantir,
};
use solenoids::InputArray;

use bus::UartBus;

const DEVICE_ADDRESS: u8 = 0x2;

type ReceiveEnablePin = Pa5<Output<PushPull>>;
type StatusLEDPin = Pa17<Output<PushPull>>;

#[rtfm::app(device = hal::pac)]
const APP: () = {
    struct Resources {
        palantir: Palantir<UartBus<ReceiveEnablePin>>,
        sercom0: hal::pac::SERCOM0,
        status_led: StatusLEDPin,
        delay: Delay,
    }
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut peripherals = Peripherals::take().unwrap();
        let mut clocks = GenericClockController::with_external_32kosc(
            peripherals.GCLK,
            &mut peripherals.PM,
            &mut peripherals.SYSCTRL,
            &mut peripherals.NVMCTRL,
        );
        let mut pins = hal::Pins::new(peripherals.PORT);

        let mut transmit_enable = pins.a4.into_push_pull_output(&mut pins.port);
        transmit_enable.set_low().unwrap();

        let uart = UartBus::easy_new(
            &mut clocks,
            peripherals.SERCOM0,
            &mut peripherals.PM,
            pins.d0,
            pins.d1,
            &mut pins.port,
            transmit_enable,
        );

        // Enable sercom0 receive complete interrupt and error interrupt.
        // This MUST be done AFTER
        uart.enable_rxc_interrupt();

        init::LateResources {
            palantir: Palantir::new_slave(DEVICE_ADDRESS, uart),
            sercom0: unsafe { Peripherals::steal().SERCOM0 },
            status_led: pins.d13.into_push_pull_output(&mut pins.port),
            delay: Delay::new(cx.core.SYST, &mut clocks),
        }
    }

    #[idle(resources = [status_led])]
    fn idle(cx: idle::Context) -> ! {
        loop {}
    }

    #[task(binds = SERCOM0, resources = [palantir, sercom0])]
    fn sercom0(cx: sercom0::Context) {
        let intflag = cx.resources.sercom0.usart_mut().intflag.read();
        if intflag.rxc().bit_is_set() {
            cx.resources.palantir.ingest();
        } else if intflag.error().bit_is_set() {
            cx.resources
                .sercom0
                .usart_mut()
                .intflag
                .write(|w| w.error().set_bit());
        }
    }

    extern "C" {
        fn SERCOM5();
    }
};