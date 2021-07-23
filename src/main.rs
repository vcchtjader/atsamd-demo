//! atsamd-demo
#![no_main]
#![no_std]

use cortex_m_rt::exception;

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{
        dpll::Dpll, gclk, gclk::Gclk1Div, gclkio::GclkOut, pclk::Pclk, retrieve_clocks, xosc::*,
    },
    eic::v2::*,
    gpio::v2::*,
    prelude::*,
    sercom::v2::{
        pad::IoSet3,
        uart::{self, BaudMode, Duplex, Oversampling},
    },
    time::U32Ext,
};

use rtic::app;

#[app(device = atsamd_hal::target_device, peripherals = true)]
mod app {
    use atsamd_hal::target_device::eic::dprescaler::STATES0_A;
    use core::fmt::Write;

    use super::*;

    type Uart = crate::uart::Uart<
        crate::uart::Config<
            crate::uart::Pads<
                atsamd_hal::pac::SERCOM0,
                IoSet3,
                Pin<PA05, AlternateD>,
                Pin<PA04, AlternateD>,
            >,
        >,
        Duplex,
    >;

    #[shared]
    struct SharedResources {
        uart: Uart,
    }

    #[local]
    struct LocalResources {
        ext_int_00: ExtInt<PA16, Floating, Normal, WithClock<OscUlp32kDriven>, SenseHigh>,
        ext_int_01: ExtInt<PA17, Floating, Normal, WithClock<OscUlp32kDriven>, SenseLow>,
        ext_int_02: ExtInt<PA18, Floating, Normal, WithClock<OscUlp32kDriven>, SenseRise>,
        ext_int_03: ExtInt<PA19, Floating, Normal, WithClock<OscUlp32kDriven>, SenseFall>,
        ext_int_04: ExtInt<PA20, Floating, Normal, WithClock<OscUlp32kDriven>, SenseBoth>,
        ext_int_05: ExtInt<PA21, Floating, Normal, WithClock<OscUlp32kDriven>, SenseHigh>,
    }

    #[init]
    fn init(cx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
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

        // Steal access to mclk for UART
        let (_, _, _, mclk) = unsafe { tokens.pac.steal() };

        let crystal = CrystalConfig::new(8.mhz()).unwrap();
        // Enable pin PA14 and PA15 as an external source for XOSC0 at 8 MHz
        let xosc0 = Xosc::from_crystal(tokens.xosc0, pins.pa14, pins.pa15, crystal).enable();

        // Take DFLL 48 MHz, divide down to 2 MHz for Gclk1
        let (gclk1, dfll) = gclk::Gclk::new(tokens.gclks.gclk1, dfll);
        let _gclk1 = gclk1.div(Gclk1Div::Div(24)).enable();

        // Configure DPLL0 to 100 MHz fed from Xosc0
        let (dpll0, _xosc0) = Dpll::from_xosc(tokens.dpll0, xosc0, 1);

        // Use 4 as source predivider, 8 MHz / (2 * ( 1 + prediv) * 50 = 100 MHz,
        // where prediv = 1
        let dpll0 = unsafe { dpll0.set_source_div(1).set_loop_div(50, 0).force_enable() };

        // Change Gclk0 from Dfll to Dpll0, MCLK = 100 MHz
        let (gclk0, _dfll, _dpll0) = gclk0.swap(dfll, dpll0);

        // Output Gclk0 on pin PB14
        let (_gclk_out0, gclk0) =
            GclkOut::enable(tokens.gclk_io.gclk_out0, pins.pb14, gclk0, false);

        let (_sercom_pclk, gclk0) = Pclk::enable(tokens.pclks.sercom0, gclk0);
        let (rx, tx) = (pins.pa05, pins.pa04);
        let pads = crate::uart::Pads::default().rx(rx).tx(tx);
        let baud = 19200.hz();
        let mut uart = crate::uart::Config::new(&mclk, device.SERCOM0, pads, baud)
            .baud(baud, BaudMode::Fractional(Oversampling::Bits16))
            .enable();

        // Setup EIC

        // Clock v2
        //tokens.apbs.eic.enable();
        let (eic_pclk, _gclk0) = Pclk::enable(tokens.pclks.eic, gclk0);

        // Driven by osculp32k
        let (eic, EICTokens, osculp32k) = EIController::from_osculp(device.EIC, osculp32k);
        let (eic_token, osculp32k) = eic.destroy(EICTokens, osculp32k);
        // Driven by Pclk<Eic, T>
        let (eic, EICTokens) = EIController::from_pclk(eic_token, eic_pclk);
        let (eic_token, _eic_pclk) = eic.destroy(EICTokens);

        // Driven by osculp32k
        let (mut eic, EICTokens, _osculp32k) = EIController::from_osculp(eic_token, osculp32k);

        // Test to Reset the EIC controller
        eic = eic.swrst();

        let (eic, ext_int_00) =
            eic.new_sync(EICTokens.ext_int_00, pins.pa16.into_floating_interrupt());

        let ext_int_00 = ext_int_00.enable_filtering(&eic);
        let ext_int_00 = ext_int_00.disable_filtering(&eic);
        let ext_int_00 = ext_int_00.enable_filtering_async(&eic);
        let ext_int_00 = ext_int_00.disable_filtering(&eic);

        let ext_int_00 = ext_int_00.set_sense_fall(&eic);
        let ext_int_00 = ext_int_00.enable_debouncing(&eic);
        let mut debounce_settings = DebouncerSettings::default();
        debounce_settings.set_states0(STATES0_A::LFREQ7);

        eic.set_debouncer_settings(&debounce_settings);

        let _pinstate = ext_int_00.pin_state();

        let ext_int_00 = ext_int_00.disable_debouncing(&eic);

        let ext_int_00 = ext_int_00.enable_debouncing_async(&eic);

        debounce_settings.set_states0(STATES0_A::LFREQ3);
        eic.set_debouncer_settings(&debounce_settings);
        //cortex_m::asm::bkpt()
        let ext_int_00 = ext_int_00.disable_debouncing(&eic);

        // Enable event output
        ext_int_00.enable_event_output(&eic);

        // Disable ExtInt0
        let (eic, ext_int_00, pa16) = eic.disable_ext_int(ext_int_00);
        // Re-enable ExtInt0
        let (eic, ext_int_00) = eic.new_async_only(ext_int_00, pa16.into_floating_interrupt());

        let (eic, ext_int_01) =
            eic.new_async_only(EICTokens.ext_int_01, pins.pa17.into_floating_interrupt());
        let (eic, ext_int_02) =
            eic.new_async_only(EICTokens.ext_int_02, pins.pa18.into_floating_interrupt());
        let (eic, ext_int_03) =
            eic.new_async_only(EICTokens.ext_int_03, pins.pa19.into_floating_interrupt());
        let (eic, ext_int_04) =
            eic.new_async_only(EICTokens.ext_int_04, pins.pa20.into_floating_interrupt());
        let (eic, ext_int_05) =
            eic.new_async_only(EICTokens.ext_int_05, pins.pa21.into_floating_interrupt());

        let eic = eic.finalize();

        let eic = eic.disable();

        let (eic, ext_int_nmi) =
            eic.new_sync_nmi(EICTokens.ext_int_nmi, pins.pa08.into_floating_interrupt());

        let (eic, ext_int_00, pa16) = eic.disable_ext_int(ext_int_00);
        let (eic, ext_int_01, pa17) = eic.disable_ext_int(ext_int_01);
        let (eic, ext_int_02, pa18) = eic.disable_ext_int(ext_int_02);
        let (eic, ext_int_03, pa19) = eic.disable_ext_int(ext_int_03);
        let (eic, ext_int_04, pa20) = eic.disable_ext_int(ext_int_04);
        let (eic, ext_int_05, pa21) = eic.disable_ext_int(ext_int_05);

        // Enable as Sync instead
        let (eic, ext_int_00) = eic.new_sync(ext_int_00, pa16.into_floating_interrupt());
        let (eic, ext_int_01) = eic.new_sync(ext_int_01, pa17.into_floating_interrupt());
        let (eic, ext_int_02) = eic.new_sync(ext_int_02, pa18.into_floating_interrupt());
        let (eic, ext_int_03) = eic.new_sync(ext_int_03, pa19.into_floating_interrupt());
        let (eic, ext_int_04) = eic.new_sync(ext_int_04, pa20.into_floating_interrupt());
        let (eic, ext_int_05) = eic.new_sync(ext_int_05, pa21.into_floating_interrupt());

        // Relies on the type annotations provided in RTIC resources
        let ext_int_00 = ext_int_00.set_sense_mode(&eic);
        let ext_int_01 = ext_int_01.set_sense_mode(&eic);
        let ext_int_02 = ext_int_02.set_sense_mode(&eic);
        let ext_int_03 = ext_int_03.set_sense_mode(&eic);
        let ext_int_04 = ext_int_04.set_sense_mode(&eic);
        let ext_int_05 = ext_int_05.set_sense_mode(&eic);

        let _ext_int_nmi: NmiExtInt<PA08, Floating, Normal, WithClock<OscUlp32kDriven>, SenseRise> =
            ext_int_nmi.set_sense_mode(&eic);

        writeln!(
            &mut uart as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
            "Booted RTIC"
        )
        .unwrap();
        (
            SharedResources { uart },
            LocalResources {
                ext_int_00,
                ext_int_01,
                ext_int_02,
                ext_int_03,
                ext_int_04,
                ext_int_05,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = SERCOM0_2, shared = [uart])]
    fn uart(cx: uart::Context) {
        let mut uart = cx.shared.uart;

        // Basic echo
        let input = uart.lock(|u| u.read().unwrap());
        uart.lock(|u| {
            writeln!(
                u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                "{}",
                input as char
            )
            .unwrap()
        });
    }

    #[task(binds = EIC_EXTINT_0, shared = [uart], local = [ext_int_00])]
    fn eic_00(cx: eic_00::Context) {
        let mut uart = cx.shared.uart;
        let ext_int = cx.local.ext_int_00;
        ext_int.clear_interrupt_status();

        let pin = ext_int.borrow_inner_pin();

        if pin.is_high().is_ok() {
            uart.lock(|u| {
                writeln!(
                    u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                    "EIC 0 pin is high!"
                )
                .unwrap()
            });
        } else {
            uart.lock(|u| {
                writeln!(
                    u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                    "EIC 0!"
                )
                .unwrap()
            });
        }
    }
    #[task(binds = EIC_EXTINT_1, shared = [uart], local = [ext_int_01])]
    fn eic_01(cx: eic_01::Context) {
        let mut uart = cx.shared.uart;
        let ext_int = cx.local.ext_int_01;
        ext_int.clear_interrupt_status();

        uart.lock(|u| {
            writeln!(
                u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                "EIC 1!"
            )
            .unwrap()
        });
    }
    #[task(binds = EIC_EXTINT_2, shared = [uart], local = [ext_int_02])]
    fn eic_02(cx: eic_02::Context) {
        let mut uart = cx.shared.uart;
        let ext_int = cx.local.ext_int_02;
        ext_int.clear_interrupt_status();

        uart.lock(|u| {
            writeln!(
                u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                "EIC 2!"
            )
            .unwrap()
        });
    }
    #[task(binds = EIC_EXTINT_3, shared = [uart], local = [ext_int_03])]
    fn eic_03(cx: eic_03::Context) {
        let mut uart = cx.shared.uart;
        let ext_int = cx.local.ext_int_03;
        ext_int.clear_interrupt_status();

        uart.lock(|u| {
            writeln!(
                u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                "EIC 3!"
            )
            .unwrap()
        });
    }
    #[task(binds = EIC_EXTINT_4, shared = [uart], local = [ext_int_04])]
    fn eic_04(cx: eic_04::Context) {
        let mut uart = cx.shared.uart;
        let ext_int = cx.local.ext_int_04;
        ext_int.clear_interrupt_status();

        uart.lock(|u| {
            writeln!(
                u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                "EIC 4!"
            )
            .unwrap()
        });
    }
    #[task(binds = EIC_EXTINT_5, shared = [uart], local = [ext_int_05])]
    fn eic_05(cx: eic_05::Context) {
        let mut uart = cx.shared.uart;
        let ext_int = cx.local.ext_int_05;
        ext_int.clear_interrupt_status();

        uart.lock(|u| {
            writeln!(
                u as &mut dyn atsamd_hal::ehal::serial::Write<_, Error = _>,
                "EIC 5!"
            )
            .unwrap()
        });
    }

    #[allow(non_snake_case)]
    #[exception]
    unsafe fn NonMaskableInt() {
        let _we_are_in_nmi = 0;
    }
}
