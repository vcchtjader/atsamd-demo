//! atsamd-demo
// #![deny(warnings)]
#![no_main]
#![no_std]
#![allow(unused_variables)]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{dpll::Dpll, gclk, pclk::*, retrieve_clocks, xosc::*, xosc32k::*},
    gpio::v2::Pin,
    gpio::v2::*,
    prelude::*,
    pukcc::*,
    sercom::*,
    time::U32Ext,
};
use core::fmt::Write;

use rtic::app;

type Uart0 = UART0<Pin<PA05, AlternateD>, Pin<PA04, AlternateD>, (), ()>;

#[app(device = atsamd_hal::target_device, peripherals = true )]
mod app {
    use super::*;

    #[resources]
    struct Resources {
        uart: Uart0,
        pukcc: Pukcc,
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
        let xosc32k = Xosc32k::from_crystal(tokens.xosc32k, pins.pa00, pins.pa01)
            .enable()
            .activate_32k();

        let (gclk1, _) = gclk::Gclk::new(tokens.gclks.gclk1, xosc32k);
        let gclk1 = gclk1.enable();

        let (sercom_pclk, gclk0) = Pclk::enable(tokens.pclks.sercom0, gclk0);
        let sercom_pclk = sercom_pclk.into();

        let pukcc = Pukcc::enable(&mut mclk).unwrap();

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

        writeln!(uart, "RTIC init() done").unwrap();

        (init::LateResources { pukcc, uart }, init::Monotonics())
    }

    #[task(binds = SERCOM0_2, resources = [uart, pukcc])]
    fn uart(cx: uart::Context) {
        let mut uart = cx.resources.uart;
        let mut pukcc = cx.resources.pukcc;
        let v: u8 = uart.lock(|u| u.read().unwrap());

        let mut k = [0_u8; 32];
        *(k.last_mut().unwrap()) = v.saturating_sub(65);

        let hash = &[
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];

        let private_key = &[
            0x30, 0x8d, 0x6c, 0x77, 0xcc, 0x43, 0xf7, 0xb8, 0x4f, 0x44, 0x74, 0xdc, 0x2f, 0x99,
            0xf6, 0x33, 0x3e, 0x26, 0x8a, 0x0c, 0x94, 0x4c, 0xde, 0x56, 0xff, 0xb5, 0x27, 0xb7,
            0x7f, 0xa6, 0x11, 0x0c,
        ];

        let public_key = &[
            0x16, 0xa6, 0xbd, 0x9a, 0x66, 0x66, 0x36, 0xd0, 0x72, 0x86, 0xde, 0x78, 0xb9, 0xa1,
            0xe7, 0xf6, 0xdd, 0x67, 0x75, 0xb2, 0xc6, 0xf4, 0x2c, 0xcf, 0x83, 0x2d, 0xe4, 0x5e,
            0x1e, 0x22, 0x9d, 0x84, 0x0a, 0xca, 0x0d, 0xdd, 0xe8, 0xf5, 0xc8, 0x2f, 0x84, 0x10,
            0xb5, 0x62, 0xc2, 0x3a, 0x46, 0xde, 0xcd, 0xcb, 0x59, 0x6e, 0x40, 0x02, 0xcb, 0x10,
            0xc6, 0x2f, 0x5b, 0x5e, 0xb5, 0xf2, 0xa7, 0xd7,
        ];

        let mut generated_signature = [0_u8; 64];

        uart.lock(|u| {
            pukcc.lock(|p| {
                match p.zp_ecdsa_sign::<curves::Nist256p>(
                    &mut generated_signature,
                    hash,
                    private_key,
                    &k,
                ) {
                    Ok(_) => writeln!(
                        u,
                        "Signing succeeded\nSignature: {:02X?}",
                        generated_signature
                    )
                    .unwrap(),
                    Err(e) => writeln!(u, "signing failed: {:?}", e).unwrap(),
                }
            })
        });

        uart.lock(|u| {
            pukcc.lock(|p| {
                match p.zp_ecdsa_verify_signature::<curves::Nist256p>(
                    &generated_signature,
                    hash,
                    public_key,
                ) {
                    Ok(_) => writeln!(u, "1st signature validation succeeded").unwrap(),
                    Err(e) => writeln!(u, "1st signature validation failed: {:?}", e).unwrap(),
                }
            })
        });

        generated_signature[20] = 0xff;

        uart.lock(|u| {
            pukcc.lock(|p| {
                match p.zp_ecdsa_verify_signature::<curves::Nist256p>(
                    &generated_signature,
                    hash,
                    public_key,
                ) {
                    Ok(_) => writeln!(u, "1st + bitflip signature validation succeeded").unwrap(),
                    Err(e) => {
                        writeln!(u, "1st + bitflip signature validation failed: {:?}", e).unwrap()
                    }
                }
            })
        });

        let mut validated_signature = [
            0xab, 0x1c, 0x38, 0x17, 0x65, 0x8c, 0x2e, 0x20, 0x9d, 0x76, 0x2a, 0xb5, 0x99, 0x3c,
            0x91, 0x77, 0xc4, 0xca, 0xa4, 0x8b, 0xa9, 0xc6, 0x62, 0x6, 0xd0, 0x65, 0xe5, 0xbb,
            0x9a, 0x83, 0x65, 0xe1, 0x6a, 0xd7, 0x33, 0x89, 0x3f, 0x11, 0x98, 0xbd, 0xf3, 0xa0,
            0xfb, 0x6e, 0x27, 0x25, 0x71, 0xf8, 0xa8, 0xd5, 0x2b, 0xbc, 0xed, 0xd2, 0xcf, 0xd2,
            0x9a, 0x43, 0xfc, 0xdf, 0x24, 0x8e, 0x98, 0xcd,
        ];

        uart.lock(|u| {
            pukcc.lock(|p| {
                match p.zp_ecdsa_verify_signature::<curves::Nist256p>(
                    &validated_signature,
                    hash,
                    public_key,
                ) {
                    Ok(_) => writeln!(u, "2nd signature validation succeeded").unwrap(),
                    Err(e) => writeln!(u, "2nd signature validation failed: {:?}", e).unwrap(),
                }
            })
        });

        validated_signature[6] = 0x1;

        uart.lock(|u| {
            pukcc.lock(|p| {
                match p.zp_ecdsa_verify_signature::<curves::Nist256p>(
                    &validated_signature,
                    hash,
                    public_key,
                ) {
                    Ok(_) => writeln!(u, "2nd + bitflip signature validation succeeded").unwrap(),
                    Err(e) => {
                        writeln!(u, "2nd + bitflip signature validation failed: {:?}", e).unwrap()
                    }
                }
            })
        });
    }
}
