//! LPA UART debug controller example
// Cargo doc workaround https://github.com/rust-lang/rust/issues/62184
#![cfg_attr(not(doc), no_main)]
#![no_std]

use panic_halt as _;

use core::fmt::Write;
use rtic::app;

use atsamd_hal::{
    clock::v2::{pclk::Pclk, retrieve_clocks},
    gpio::v2::Pin,
    gpio::v2::*,
    prelude::*,
    sercom::*,
    time::U32Ext,
};

#[app(device = atsamd_hal::target_device,
    peripherals = true,
    dispatchers = [ TCC0_MC0, TCC1_MC0, TCC1_MC1],
    )]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        uart: UART0<Pin<PA05, AlternateD>, Pin<PA04, AlternateD>, (), ()>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics()) {
        let mut device = cx.device;

        // Get the clocks & tokens
        let (gclk0, dfll, _osculp32k, tokens) = retrieve_clocks(
            device.OSCCTRL,
            device.OSC32KCTRL,
            device.GCLK,
            device.MCLK,
            &mut device.NVMCTRL,
        );

        // Get the pins
        let pins = Pins::new(device.PORT);

        // Steal access to mclk for UART v1
        let (_, _, _, mut mclk) = unsafe { tokens.pac.steal() };

        let (gclk0, _gclk5, _gclk1, _xosc32k, _dpll0, _dfll) = atsamd_hal::clocking_preset_gclk0_120mhz_gclk5_2mhz_gclk1_external_32khz!(
            gclk0, dfll, pins.pa00, pins.pa01, tokens
        );

        let (sercom_pclk, _gclk0) = Pclk::enable(tokens.pclks.sercom0, gclk0);
        let sercom_pclk = sercom_pclk.into();

        let mut uart = UART0::new(
            &sercom_pclk,
            115_200.hz(),
            device.SERCOM0,
            &mut mclk,
            (pins.pa05.into(), pins.pa04.into()),
        );
        uart.intenset(|w| {
            w.rxc().set_bit();
        });

        cortex_m::asm::bkpt();

        uart.write_str("\n\rBooted RTIC.\n\r").unwrap();

        (Shared { uart }, Local {}, init::Monotonics())
    }

    #[task(binds = SERCOM0_2, shared = [uart], local = [])]
    fn uart(cx: uart::Context) {
        let mut uart = cx.shared.uart;

        // Basic echo
        let input = uart.lock(|u| u.read().unwrap());

        if input as char == '\r' {
            // Possible to handle newline differently
            uart.lock(|u| write!(u, "{}", input as char).unwrap());
        } else {
            uart.lock(|u| write!(u, "{}", input as char).unwrap());
        }
    }
}
