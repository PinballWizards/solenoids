//File:     ~/solenoids/board/src/main.rs
//Authors:  Will Tekulve + Patrick Taliaferro
//Date:     February-March 2020

//Real-Time OS for the masses (rtfm) is used
//and evoked to compile the program.
#![no_std]
#![no_main]

//Set the panicking behavior to halt
extern crate panic_halt;

//Mask the specific board used
//Specify rtfm is used
use feather_m0 as hal;
use rtfm;

//Assign hal several things that an MCU needs
//for this program.
use hal::{
    clock::GenericClockController,
    delay::Delay,
    gpio::{Output, Pa17, Pa5, PushPull},
    pac::Peripherals,
    prelude::*,
    spi_master,
};

//Create a comms object to interact with the other boards.
use palantir::{feather_bus as bus, Palantir};
use solenoids;

//Set up the Uartbus for use with palantir
use bus::UartBus;

//bring in periphs.rs module
mod periphs;

//Set the device address, this is used by
//palantir to create a slave process later on
const DEVICE_ADDRESS: u8 = 0x2;

//Alias the pin names
type ReceiveEnablePin = Pa5<Output<PushPull>>;
type StatusLEDPin = Pa17<Output<PushPull>>;

//Start rtfm
#[rtfm::app(device = hal::pac)]
const APP: () = {
    //Define MCU resources needed within rtfm
    struct Resources<'a> {
        palantir: Palantir<UartBus<ReceiveEnablePin>>,
        sercom0: hal::pac::SERCOM0,
        status_led: StatusLEDPin,
        delay: Delay,
        solenoids: periphs::Solenoids,
    }
    //Initialization sequence/Object definition
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

        let mut transmit_enable = pins.d5.into_push_pull_output(&mut pins.port);
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

        let spi = spi_master(
            &mut clocks,
            1.mhz(),
            peripherals.SERCOM4,
            &mut peripherals.PM,
            pins.sck,
            pins.mosi,
            pins.miso,
            &mut pins.port,
        );

        //load a0 to bring in a latch output
        let load_pin = pins.a0.into_push_pull_output(&mut pins.port);


        let pwm_controller = solenoids::pwm::Controller::new(
            &mut clocks,
            100.hz(),
            peripherals.TCC0,
            peripherals.TCC1,
            peripherals.TCC2,
            peripherals.TC3,
            &mut peripherals.PM,
        );

        //bring in another group of resources

        init::LateResources {
            palantir: Palantir::new_slave(DEVICE_ADDRESS, uart),
            sercom0: unsafe { Peripherals::steal().SERCOM0 },
            status_led: pins.d13.into_push_pull_output(&mut pins.port),
            delay: Delay::new(cx.core.SYST, &mut clocks),
            solenoids: periphs::Solenoids::new(pwm_controller, spi, load_pin),
        }
    }

    //This is where stuff will occur
    #[idle(resources = [status_led])]
    fn idle(cx: idle::Context) -> ! {
        loop {}
    }

    //comms stuff
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
