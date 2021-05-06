//! atsamd-demo
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{gclk, xosc::*, DpllConfig, GclkConfig, GclkOut, Tokens},
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
        let (gclk0, dfll, tokens) = Tokens::new(
            device.OSCCTRL,
            device.OSC32KCTRL,
            device.GCLK,
            device.MCLK,
            &mut device.NVMCTRL,
        );

        // Get the pins
        let pins = Pins::new(device.PORT);

        // Enable pin PA14 and PA15 as an external source for XOSC0 at 8 MHz
        let xosc_config0: XOscConfig<Osc0, _> =
            XOscConfig::from_crystal(tokens.sources.xosc0, pins.pa14, pins.pa15, 8.mhz());

        // Configure the external oscillator
        let xosc0 = xosc_config0
            //.set_on_demand(false)
            //.set_run_standby(true)
            //.set_crystal_current(CrystalCurrent::BaseFreq)
            .enable();

        // Configure DPLL0 to 100 MHz
        let (dpll0, xosc0) = DpllConfig::from_xosc(tokens.sources.dpll0, xosc0);
        let dpll0 = dpll0.set_source_div(1).set_loop_div(50, 0).enable();

        // Configure DPLL1 to 90 MHz
        let (dpll1, xosc0) = DpllConfig::from_xosc(tokens.sources.dpll1, xosc0);
        let dpll1 = dpll1.set_source_div(1).set_loop_div(49, 0).enable();

        // Change Gclk0 from Dfll to Dpll0, MCLK = 100 MHz
        let (_gclk0, _dfll, dpll0) = unsafe { gclk0.swap(dfll, dpll0) };

        //
        // Gckl1
        //
        // From 8 MHz oscillator XOSC0
        let (gclk1, _xosc0) = GclkConfig::new(tokens.gclks.gclk1, xosc0);

        // Divide gclk1 down to 2 MHz
        let gclk1 = gclk1.div(gclk::Div::Div(4)).enable();

        // Output Gclk1 on pin PB15
        let gclk_out1 = tokens.sources.gclk_io.gclk_out1;
        let (_gclk_out1, gclk1) = GclkOut::new(gclk_out1, pins.pb15, gclk1, false);

        //
        // Gckl2
        //
        // Set Gclk2 to use Dpll0 divided by 100 = 1 MHz
        let (gclk2, _dpll0) = GclkConfig::new(tokens.gclks.gclk2, dpll0);
        let gclk2 = gclk2.div(gclk::Div::Div(100)).enable();
        //let gclk2 = gclk2.div(gclk::Div::DivPow2(8)).enable();
        //let gclk2 = gclk2.div(gclk::Div::MaxMinusOne).enable();
        //let gclk2 = gclk2.div(gclk::Div::Max).enable();

        // Output Gclk2 on pin PB16
        let gclk_out2 = tokens.sources.gclk_io.gclk_out2;
        let (_gclk_out2, _gclk2) = GclkOut::new(gclk_out2, pins.pb16, gclk2, false);

        //
        // Gckl3
        //
        let (gclk3, _dpll1) = GclkConfig::new(tokens.gclks.gclk3, dpll1);
        let gclk3 = gclk3.div(gclk::Div::Div(10)).enable();

        // Output Gclk3 on pin PB17
        let gclk_out3 = tokens.sources.gclk_io.gclk_out3;
        let (_gclk_out3, _gclk3) = GclkOut::new(gclk_out3, pins.pb17, gclk3, false);

        //
        // Gckl5
        //

        // Configure gclk5 using GCLK1 as source to run at 200 kHz
        let (gclk5, _gclk1) = GclkConfig::new(tokens.gclks.gclk5, gclk1);
        let gclk5 = gclk5.div(gclk::Div::Div(4)).enable();

        // Output Gclk5 on pin PB11
        let gclk_out5 = tokens.sources.gclk_io.gclk_out5;
        let (_gclk_out5, _gclk5) = GclkOut::new(gclk_out5, pins.pb11, gclk5, false);

        // Setup frequency monitor
        //let freqm = Freqm::new(device.FREQM, &mut mclk);

        (
            init::LateResources {
            //freqm: freqm,
        },
            init::Monotonics(),
        )
    }
}
