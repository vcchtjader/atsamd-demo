//! atsamd-demo
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{
        dpll::Dpll,
        gclk,
        gclk::{Gclk1Div, GclkDiv},
        gclkio::{GclkIn, GclkOut},
        pclk::Pclk,
        retrieve_clocks,
        rtc::*,
        xosc::*,
        xosc32k::*,
    },
    gpio::v2::Pins,
    time::U32Ext,
};

use rtic::app;

#[app(device = atsamd_hal::target_device, peripherals = true )]
mod app {
    use cortex_m::interrupt::disable;

    use super::*;

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics()) {
        let mut device = cx.device;

        // Get the clocks & tokens
        let (gclk0, dfll, osculp32k, tokens) = retrieve_clocks(
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

        // Take DFLL 48 MHz, divide down to 2 MHz for Gclk1
        let (gclk1, dfll) = gclk::Gclk::new(tokens.gclks.gclk1, dfll);
        let gclk1 = gclk1.div(Gclk1Div::Div(24)).enable();

        // Enable output for 2 MHz clock on PB15
        let (_gclk_out1, _gclk1) =
            GclkOut::enable(tokens.gclk_io.gclk_out1, pins.pb15, gclk1, false);

        // Either from pclk or xosc0
        // =========================

        // Setup DPLL0 using pclk of GCLK1
        //let (pclk_dpll0, _gclk1) = Pclk::enable(tokens.pclks.dpll0, gclk1);
        //let pclk_dpll0 = Dpll::from_pclk(tokens.dpll0, pclk_dpll0);
        //let dpll0 = pclk_dpll0.set_loop_div(50, 0).enable();

        // Configure DPLL0 to 100 MHz fed from Xosc0
        let (dpll0, _xosc0) = Dpll::from_xosc(tokens.dpll0, xosc0, 1);

        // Use the predivider, 8 MHz / 4 * 50 = 100 MHz

        let dpll0 = dpll0.set_source_div(1).set_loop_div(50, 0).enable();

        // Disable the DPLL0
        let dpll0 = dpll0.disable();

        // Configure DPLL0 with 8 / 4 * 60 = 120 MHz
        let dpll0 = dpll0.set_source_div(1).set_loop_div(60, 0).enable();

        //// Change Gclk0 from Dfll to Dpll0, MCLK = 100 MHz
        let (gclk0, _dfll, _dpll0) = gclk0.swap(dfll, dpll0);

        //// Output Gclk0 on pin PB14
        let (_gclk_out0, _gclk0) =
            GclkOut::enable(tokens.gclk_io.gclk_out0, pins.pb14, gclk0, false);

        //// ----
        //// Input for Gclk3 on pin PB17 (assumed frequency of 2 MHz)
        let gclk_in3 = GclkIn::enable(tokens.gclk_io.gclk_in3, pins.pb17, 2.mhz());
        let (gclk3, _gclk_in3) = gclk::Gclk::new(tokens.gclks.gclk3, gclk_in3);
        let gclk3 = gclk3.enable();

        // Setup DPLL1 with input from Gclk3, fed from external 2 MHz signal on pin PB17
        let (pclk_dpll1, _gclk3) = Pclk::enable(tokens.pclks.dpll1, gclk3);
        let dpll1 = Dpll::from_pclk(tokens.dpll1, pclk_dpll1);
        // Configure DPLL1 to run at 2 * 50 = 100 MHz
        let dpll1 = dpll1.set_loop_div(50, 0).enable();

        // Output DPLL1 on PB20 via Gclk6, divided by 200 resulting in 0.5 MHz output
        let (gclk6, _dpll1) = gclk::Gclk::new(tokens.gclks.gclk6, dpll1);
        let gclk6 = gclk6.div(GclkDiv::Div(200)).enable();
        let (_gclk_out6, _gclk6) =
            GclkOut::enable(tokens.gclk_io.gclk_out6, pins.pb20, gclk6, false);

        // ----
        // Enable external 32k-oscillator
        let xosc32k =
            Xosc32k::from_crystal(tokens.xosc32k, pins.pa00, pins.pa01).set_gain_mode(true);
        let xosc32k = xosc32k.set_start_up(StartUp32k::CYCLE2048);
        let xosc32k = xosc32k.set_on_demand(false).set_run_standby(true);
        let xosc32k = xosc32k.enable();
        let xosc32k = xosc32k.activate_1k();
        let xosc32k = xosc32k.activate_32k();
        cortex_m::asm::bkpt();
        let xosc32k = xosc32k.deactivate_1k();
        cortex_m::asm::bkpt();
        let xosc32k = xosc32k.activate_1k();
        let xosc32k = xosc32k.disable();
        let (xosc32ktoken, pinpa00, _pinpa01) = xosc32k.free();

        let xosc32k = Xosc32k::from_clock(xosc32ktoken, pinpa00);
        let xosc32k = xosc32k.enable();
        let xosc32k = xosc32k.activate_32k();

        let xosc32k = set_rtc_clock_32k(xosc32k);
        //let xosc32k = xosc32k.disable();
        let xosc32k = unset_rtc_clock_32k(xosc32k);

        let xosc32k = xosc32k.activate_1k();
        let xosc32k = set_rtc_clock_1k(xosc32k);
        //let xosc32k = xosc32k.disable();
        let xosc32k = unset_rtc_clock_1k(xosc32k);

        let (_gclk11, xosc32k) = gclk::Gclk::new(tokens.gclks.gclk11, xosc32k);

        // Xosc32k = 32kHz Expressed as MHz: >>> 32*1024/1000/1000 = 0.032768
        // 100 / 0.032768 = 3051.7578125
        //
        // Missing 0.7578...
        //
        // Use fractional divider: x / 32 = 0.75..
        // 24/32 = 0.75
        //
        // 3052 * (32*1024/1000/1000) = 100.007936
        //let (dpll1, xosc32k) = Dpll::from_xosc32k(tokens.dpll1, xosc32k);
        //let _dpll1 = dpll1.set_loop_div(3000, 24).enable();

        let (gclk2, _xosc32k) = gclk::Gclk::new(tokens.gclks.gclk2, xosc32k);
        let gclk2 = gclk2.div(gclk::GclkDiv::Div(2)).enable();
        let (_gclk_out2, _gclk2) =
            GclkOut::enable(tokens.gclk_io.gclk_out2, pins.pb16, gclk2, false);

        let osculp32k = osculp32k.set_calibration(1);
        let osculp32k = osculp32k.deactivate_32k();
        let osculp32k = osculp32k.activate_32k();
        let osculp32k = osculp32k.deactivate_1k();
        let osculp32k = osculp32k.activate_1k();
        let osculp32k = osculp32k.write_lock();

        let (gclk5, _osculp32k) = gclk::Gclk::new(tokens.gclks.gclk5, osculp32k);
        let gclk5 = gclk5.div(gclk::GclkDiv::Div(0)).enable();
        let (_gclk_out5, _gclk5) =
            GclkOut::enable(tokens.gclk_io.gclk_out5, pins.pb11, gclk5, false);

        (init::LateResources {}, init::Monotonics())
    }
}
