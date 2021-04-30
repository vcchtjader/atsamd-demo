//! atsamd-demo
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{gclk, Tokens,  GclkConfig, Pclk, DpllConfig, GclkOut},
    gpio::v2::{Pins},
    time::U32Ext,
};

use rtic::app;

#[app(device = atsamd_hal::target_device, peripherals = true )]
mod app {

    use super::*;

    //struct Resources {
        ///// Clock controller
        //gcc: GenericClockController,
        ///// Frequency monitor
        //freqm: Freqm,

    //}

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

        //let gclk_in1 = GclkIn::new(tokens.sources.gclk_io.gclk_in1, pins.pa27, 24.mhz());
        // Enable pin PA14 and PA15 as an external source for XOSC0 at 8 MHz
        //let xosc0: XOscConfig<Osc0> = XOscConfig::from_crystal(pins.pa14, 8.mhz());
        // TODO, check this XOut second type
        //let xosc0: XOscConfig<Osc0, _> = XOscConfig::from_crystal(pins.pa14, pins.pa15, 8.mhz());

        //xosc0.enable();

        //let (gckl0, _xosc0_token) = GclkConfig::new(tokens.gclks.gclk0, xosc0);
        //let (gckl1, _xosc0_token) = GclkConfig::new(tokens.gclks.gclk1, xosc0);
        //let (gclk1, _gclk_in1) = GclkConfig::new(tokens.gclks.gclk1, 1);
        //let (dpll0, _xosc0) = DpllConfig::from_xosc(tokens.sources.dpll0, xosc0);

        // Configure gclk5 using DFLL as source to run at 2 MHz
        let (gclk5, dfll) = GclkConfig::new(tokens.gclks.gclk5, dfll);
        let gclk5 = gclk5.div(gclk::Div::Div(24)).enable();

        // Feed DPLL0 with gclk5, 2 * 60 = 120 MHz
        let (pclk_dpll0, gclk5) = Pclk::new(tokens.pclks.dpll0, gclk5);
        let dpll0 = DpllConfig::from_pclk(tokens.sources.dpll0, pclk_dpll0)
            .set_loop_div(60, 0)
            .enable();

        // Change Gclk0 from Dfll to Dpll0 and divide by 2 for 96 MHz
        let (_gclk0, _dfll, dpll0) = unsafe { gclk0.swap(dfll, dpll0) };

        // Output Gclk1 on pin PB11
        //let gclk_out1 = tokens.sources..gclk_out1;
        //let (_gclk_out1, _gclk1) = GclkOut::new(gclk_out1, pins.pb15, gclk1, false);

        // Set Gclk2 to use Dpll0 divided by 8 = 24 MHz
        let (gclk2, _dpll0) = GclkConfig::new(tokens.gclks.gclk2, dpll0);
        let gclk2 = gclk2.div(gclk::Div::DivPow2(8)).enable();
        //let gclk2 = gclk2.div(gclk::Div::MaxMinusOne).enable();
        //let gclk2 = gclk2.div(gclk::Div::Max).enable();

        // Output Gclk2 on pin PB16
        let gclk_out2 = tokens.sources.gclk_io.gclk_out2;
        let (_gclk_out2, _gclk2) = GclkOut::new(gclk_out2, pins.pb16, gclk2, false);

        // Output Gclk5 on pin PB11
        let gclk_out5 = tokens.sources.gclk_io.gclk_out5;
        let (_gclk_out5, _gclk5) = GclkOut::new(gclk_out5, pins.pb11, gclk5, false);

        // Set Gclk1 to use GclkIn1 divided by 10 = 2.4 MHz
        //let (gclk1, _gclk_in1) = GclkConfig::new(tokens.gclks.gclk1, gclk_in1);
        //let gclk1 = gclk1.div(gclk::Div::Div(10)).enable();

        // Set Dpll0 to use Gclk1 times 80 = 192 MHz
        //let (pclk_dpll0, gclk1) = Pclk::new(tokens.pclks.dpll0, gclk1);
        //let dpll0 = DpllConfig::from_pclk(tokens.sources.dpll0, pclk_dpll0)
            //.set_loop_div(80, 0)
            //.enable();

        // Change Gclk0 from Dfll to Dpll0 and divide by 2 for 96 MHz
        //let (mut gclk0, _dfll, dpll0) = unsafe { gclk0.swap(dfll, dpll0) };
        //unsafe { gclk0.div(gclk::Div::Div(2)) };

        // Set Gclk2 to use Dpll0 divided by 8 = 24 MHz
        //let (gclk2, _dpll0) = GclkConfig::new(tokens.gclks.gclk2, dpll0);
        //let _gclk2 = gclk2.div(gclk::Div::Div(8)).enable();

        // Set Gclk2 to use Gclk1 divided by 10 = 240 kHz
        //let (gclk3, _gclk1) = GclkConfig::new(tokens.gclks.gclk3, gclk1);
        //let gclk3 = gclk3.div(gclk::Div::Div(10)).enable();

        // Output Gclk3 on pin PB17
        //let gclk_out3 = tokens.sources.gclk_io.gclk_out3;
        //let (_gclk_out3, _gclk3) = GclkOut::new(gclk_out3, pins.pb17, gclk3, false);

        // Setup frequency monitor
        //let freqm = Freqm::new(device.FREQM, &mut mclk);

        (init::LateResources {
            //gcc: gcc,
            //freqm: freqm,
        }, init::Monotonics())
    }
}
