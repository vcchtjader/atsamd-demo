//! atsamd-demo
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{gclk, xosc::*, xosc32k::*, Dpll, GclkIn, GclkOut, Pclk, Tokens, GclkDiv},
    gpio::v2::Pins,
    time::U32Ext,
};

use rtic::app;

#[app(device = atsamd_hal::target_device, peripherals = true )]
mod app {

    use super::*;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics()) {
        let mut device = cx.device;

        // Get the clocks & tokens
        let (gclk0, dfll, osculp32k, tokens) = Tokens::new(
            device.OSCCTRL,
            device.OSC32KCTRL,
            device.GCLK,
            device.MCLK,
            &mut device.NVMCTRL,
        );

        // Get the pins
        let pins = Pins::new(device.PORT);

        // Enable pin PA14 and PA15 as an external source for XOSC0 at 8 MHz
        let xosc0 =
            Xosc::from_crystal(tokens.sources.xosc0, pins.pa14, pins.pa15, 8.mhz()).enable();

        // Configure DPLL0 to 100 MHz
        let (dpll0, _xosc0) = Dpll::from_xosc(tokens.sources.dpll0, xosc0);
        let dpll0 = dpll0.set_source_div(1).set_loop_div(50, 0).enable();

        // Change Gclk0 from Dfll to Dpll0, MCLK = 100 MHz
        let (gclk0, _dfll, _dpll0) = gclk0.swap(dfll, dpll0);

        // Output Gclk0 on pin PB14 (demo optional)
        let (_gclk_out0, _gclk0) =
            GclkOut::enable(tokens.sources.gclk_io.gclk_out0, pins.pb14, gclk0, false);

        // ----
        // Input for Gclk6 on pin PB20 (assumed frequency of 10 Mhz)
        let gclk_in6 = GclkIn::enable(tokens.sources.gclk_io.gclk_in6, pins.pb20, 10.mhz());

        let (gclk6, _gclk_in6) = gclk::Gclk::new(tokens.gclks.gclk6, gclk_in6);
        let gclk6 = gclk6.enable();

        let (pclk_dpll1, _gclk6) = Pclk::enable(tokens.pclks.dpll1, gclk6);

        // Configure DPLL1 to 200 MHz
        let dpll1 = Dpll::from_pclk(tokens.sources.dpll1, pclk_dpll1);
        let dpll1 = dpll1.set_source_div(1).set_loop_div(80, 0).enable();

        let (gclk1, dpll1) = gclk::Gclk::new(tokens.gclks.gclk1, dpll1);
        let gclk1 = gclk1.enable();
        let (_gclk_out1, _gclk1) =
            GclkOut::enable(tokens.sources.gclk_io.gclk_out1, pins.pb15, gclk1, false);

        let (gclk3, _dpll1) = gclk::Gclk::new(tokens.gclks.gclk3, dpll1);
        let gclk3 = gclk3.div(GclkDiv::Div(10)).enable();
        let (_gclk_out3, _gclk3) =
            GclkOut::enable(tokens.sources.gclk_io.gclk_out3, pins.pa17, gclk3, false);

        // ----
        // Enable external 32k-oscillator
        let xosc32k = Xosc32k::from_crystal(tokens.sources.xosc32k, pins.pa00, pins.pa01)
            .enable_32k(true)
            .enable();

        let (gclk2, _xosc32k) = gclk::Gclk::new(tokens.gclks.gclk2, xosc32k);
        let gclk2 = gclk2.div(gclk::GclkDiv::Div2Pow8).enable();
        let (_gclk_out2, _gclk2) =
            GclkOut::enable(tokens.sources.gclk_io.gclk_out2, pins.pb16, gclk2, false);

        let (gclk5, _osculp32k) = gclk::Gclk::new(tokens.gclks.gclk5, osculp32k);
        let gclk5 = gclk5.div(gclk::GclkDiv::Div(0)).enable();
        let (_gclk_out5, _gclk5) =
            GclkOut::enable(tokens.sources.gclk_io.gclk_out5, pins.pb11, gclk5, false);

        (init::LateResources {}, init::Monotonics())
    }
}
