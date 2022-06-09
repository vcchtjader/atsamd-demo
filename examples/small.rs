//! LPA UART debug controller example
// Cargo doc workaround https://github.com/rust-lang/rust/issues/62184
#![cfg_attr(not(doc), no_main)]
#![no_std]

use panic_halt as _;

use rtic::app;

use atsamd_hal::{
    clock::GenericClockController,
    gpio::pin::{self, *},
    prelude::*,
};

#[app(device = atsamd_hal::pac,
    peripherals = true,
    dispatchers = [ TCC0_MC0, TCC1_MC0, TCC1_MC1],
    )]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics()) {
        let mut device = cx.device;

        // Clock setup
        let _gcc = GenericClockController::with_internal_32kosc(
            device.GCLK,
            &mut device.MCLK,
            &mut device.OSC32KCTRL,
            &mut device.OSCCTRL,
            &mut device.NVMCTRL,
        );

        // Get GPIO pins
        let pins = Pins::new(device.PORT);
        let mut pa08: pin::Pin<PA08, PushPullOutput> = pins.pa08.into();

        let _pa08 = pa08.set_high();

        (Shared {}, Local {}, init::Monotonics())
    }
}
