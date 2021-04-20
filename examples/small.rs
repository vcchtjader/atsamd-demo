//! LPA UART debug controller example
// #![deny(warnings)]
//#![no_main]
// Cargo doc workaround https://github.com/rust-lang/rust/issues/62184
#![cfg_attr(not(doc), no_main)]
#![no_std]

// #[allow(unused_extern_crates)]
extern crate panic_semihosting;
// extern crate panic_halt;

use cortex_m_semihosting::hprintln;
use rtic::app;

use atsamd_hal::gpio::{Output, Pd0, PushPull};
use atsamd_hal::{
    clock::GenericClockController, common::pad::PadPin, gpio::GpioExt, target_device as pac,
};
use embedded_hal::digital::v2::OutputPin;

#[app(device = atsamd_hal::target_device, peripherals = true )]
const APP: () = {
    extern "C" {
        fn TCC1_INTREQ_1();
        fn TCC1_INTREQ_2();
        fn TCC2_INTREQ_2();
    }

    #[init]
    fn init(cx: init::Context) -> () {
        cortex_m::asm::delay(3 * 12_000_000);
        //cortex_m::asm::bkpt();
        let mut device = cx.device;

        // == Clock setup
        let mut gcc = GenericClockController::with_internal_32kosc(
            device.GCLK,
            &mut device.MCLK,
            &mut device.OSC32KCTRL,
            &mut device.OSCCTRL,
            &mut device.NVMCTRL,
        );

        // == Pin I/O port setup
        let mut port = device.PORT.split();

        pub type En5V0Pin = Pd0<Output<PushPull>>;
        let mut en5v0: En5V0Pin = port.pd0.into_push_pull_output(&mut port.port);

        en5v0.set_high().ok();
    }
};
