//! atsamd-demo
// #![deny(warnings)]
#![no_main]
#![no_std]
#![allow(unused_variables)]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{dpll::Dpll, gclk, gclkio::GclkOut, pclk::*, retrieve_clocks, xosc::*, xosc32k::*},
    freqm::Freqm,
    gpio::v2::*,
    gpio::Pin,
    pac::SERCOM0,
    prelude::*,
    sercom::*,
    time::U32Ext,
};
use core::fmt::Write;

use rtic::app;

#[app(device = atsamd_hal::target_device, peripherals = true )]
mod app {
    use super::*;

    #[resources]
    struct Resources {
        uart: UART0<
            Pad<SERCOM0, Pad1, Pin<PA05, AlternateD>>,
            Pad<SERCOM0, Pad0, Pin<PA04, AlternateD>>,
            (),
            (),
        >,
        freqm: Freqm,
        freqm_ref: Pclk<FreqmRef, gclk::Gen1>,
        freqm_msr: Pclk<FreqmMsr, gclk::Gen0>,
    }

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

        let (_, _, _, mut mclk) = unsafe { tokens.pac.steal() };

        // Get the pins
        let pins = Pins::new(device.PORT);

        // Enable pin PA14 and PA15 as an external source for XOSC0 at 8 MHz
        let xosc0 = Xosc::from_crystal(tokens.xosc0, pins.pa14, pins.pa15, 8.mhz()).enable();

        // Configure DPLL0 to 100 MHz fed from Xosc0
        let (dpll0, _xosc0) = Dpll::from_xosc(tokens.dpll0, xosc0, 1);

        // Configure DPLL0 with 8 / 4 * 50 = 120 MHz
        let dpll0 = dpll0.set_source_div(1).set_loop_div(95, 0).enable();

        //// Change Gclk0 from Dfll to Dpll0, MCLK = 100 MHz
        let (gclk0, _dfll, _dpll0) = gclk0.swap(dfll, dpll0);

        // Enable external 32k-oscillator
        let xosc32k = Xosc32k::from_crystal(tokens.xosc32k, pins.pa00, pins.pa01).enable();

        let (gclk1, _) = gclk::Gclk::new(tokens.gclks.gclk1, xosc32k);
        let gclk1 = gclk1.enable();

        let (sercom_pclk, gclk0) = Pclk::enable(tokens.pclks.sercom0, gclk0);
        let sercom_pclk = sercom_pclk.into();

        let pads: (Pad<SERCOM0, Pad1, _>, Pad<SERCOM0, Pad0, _>) =
            (pins.pa05.into(), pins.pa04.into());
        let mut uart = UART0::new(&sercom_pclk, 115_200.hz(), device.SERCOM0, &mut mclk, pads);
        uart.intenset(|w| {
            w.rxc().set_bit();
        });

        // FREQM
        // User input:
        let refnum: u8 = u8::MAX;
        // Question: In VCC impl, Pclk setup occurs a little bit later.
        // Keep in mind in case of problems.
        let (freqm_ref, _) = Pclk::enable(tokens.pclks.freqm_ref, gclk1);
        let (freqm_msr, _) = Pclk::enable(tokens.pclks.freqm_msr, gclk0);

        let freqm = Freqm::new(device.FREQM, tokens.apbs.freqm.enable());

        uart.write_str("Press any key to make a frequency measurement.\n").unwrap();

        (
            init::LateResources {
                uart,
                freqm,
                freqm_ref,
                freqm_msr,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = SERCOM0_2, resources = [uart, freqm, freqm_ref, freqm_msr])]
    fn uart(cx: uart::Context) {
        let mut uart = cx.resources.uart;
        let mut freqm = cx.resources.freqm;
        let mut freqm_ref = cx.resources.freqm_ref;
        let mut freqm_msr = cx.resources.freqm_msr;
        let _ = uart.lock(|u| { u.read().unwrap() });
        uart.lock(|u| {
            freqm.lock(|f| {
                freqm_ref.lock(|f_ref| {
                    freqm_msr.lock(|f_msr| {
                        match f.measure_frequency(f_msr, f_ref, 255) {
                            Ok(v) => writeln!(u, "Frequency measured: {}", v.0).unwrap(),
                            Err(_) => unimplemented!(),
                        }
                    })
                })
            })
        });
    }
}
