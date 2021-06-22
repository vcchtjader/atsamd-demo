//! atsamd-demo
// #![deny(warnings)]
#![no_main]
#![no_std]
#![allow(unused_variables)]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{
        dpll::Dpll, gclk, gclkio::GclkOut, pclk::Pclk, retrieve_clocks, xosc::*, xosc32k::*,
    },
    freqm::Freqm,
    gpio::v2::Pins,
    time::U32Ext,
};

use rtic::app;

static mut FINAL_MEASUREMENT: u32 = 0;

#[app(device = atsamd_hal::target_device, peripherals = true )]
mod app {

    //use cortex_m::interrupt::disable;

    use super::*;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics()) {
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

        // Enable pin PA14 and PA15 as an external source for XOSC0 at 8 MHz
        let xosc0 = Xosc::from_crystal(tokens.xosc0, pins.pa14, pins.pa15, 8.mhz()).enable();

        // Configure DPLL0 to 100 MHz fed from Xosc0
        let (dpll0, _xosc0) = Dpll::from_xosc(tokens.dpll0, xosc0, 1);

        // Configure DPLL0 with 8 / 4 * 60 = 120 MHz
        let dpll0 = dpll0.set_source_div(1).set_loop_div(55, 0).enable();

        //// Change Gclk0 from Dfll to Dpll0, MCLK = 100 MHz
        let (gclk0, _dfll, _dpll0) = gclk0.swap(dfll, dpll0);

        // Enable external 32k-oscillator
        let xosc32k = Xosc32k::from_crystal(tokens.xosc32k, pins.pa00, pins.pa01)
            .enable_32k(true)
            .enable();

        let (gclk1, _) = gclk::Gclk::new(tokens.gclks.gclk1, xosc32k);
        let gclk1 = gclk1.enable();

        // FREQM
        // User input:
        let refnum: u8 = u8::MAX;
        // Question: In VCC impl, Pclk setup occurs a little bit later.
        // Keep in mind in case of problems.
        let (freqm_ref, _) = Pclk::enable(tokens.pclks.freqm_ref, gclk1);
        let (freqm_msr, _) = Pclk::enable(tokens.pclks.freqm_msr, gclk0);

        let mut freqm = Freqm::new(device.FREQM, tokens.apbs.freqm.enable());

        unsafe {
            FINAL_MEASUREMENT = match freqm.measure_frequency(&freqm_msr, &freqm_ref, refnum) {
                Ok(v) => v.0,
                Err(_) => unimplemented!()
            }
        }

        (init::LateResources {}, init::Monotonics())
    }
}
