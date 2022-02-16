//! atsamd-demo
#![no_main]
#![no_std]

use panic_halt as _;

use atsamd_hal::{
    clock::v2::{
        dpll::Dpll, gclk, gclk::Gclk1Div, gclkio::GclkOut, retrieve_clocks, xosc::*, xosc32k::*,
        Source,
    },
    gpio::v2::Pins,
    time::U32Ext,
};

use dwt_systick_monotonic::fugit::TimerDurationU32;

const SCHEDULE_FREQ: u32 = 100_000_000;

use rtic::app;

#[app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [TCC1_MC1]
 )]
mod app {
    use super::*;
    use dwt_systick_monotonic::*;
    use rtic::Monotonic;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<SCHEDULE_FREQ>;

    #[shared]
    struct SharedResources {}

    #[local]
    struct LocalResources {}

    #[init]
    fn init(mut cx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
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

        // Enable external 32k-oscillator
        let xosc32k =
            Xosc32k::from_crystal(tokens.xosc32k, pins.pa00, pins.pa01).set_gain_mode(true);
        let xosc32k = xosc32k.enable();
        let xosc32k = xosc32k.activate_1k();
        let _xosc32k = xosc32k.activate_32k();

        // Initialize the monotonic
        let mono = DwtSystick::new(&mut cx.core.DCB, cx.core.DWT, cx.core.SYST, gclk0.freq().0);

        let _ = periodic::spawn();

        (
            SharedResources {},
            LocalResources {},
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task]
    fn periodic(_: periodic::Context) {
        //let _now = monotonics::now();
        //
        // Should be 1 second
        let _ = periodic::spawn_after(TimerDurationU32::from_ticks(SCHEDULE_FREQ));
    }
}
