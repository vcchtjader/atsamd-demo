//! atsamd-demo using clockv2
#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
use atsamd_hal::{
    clock::v2::{
        dpll::Dpll, gclk, gclk::Gclk1Div, gclkio::GclkOut, por_state, xosc::*, xosc32k::*, Source,
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

        // Get the clock power-on-reset state
        let (_buses, clocks, tokens) = por_state(
            device.OSCCTRL,
            device.OSC32KCTRL,
            device.GCLK,
            device.MCLK,
            &mut device.NVMCTRL,
        );

        // Get the pins
        let pins = Pins::new(device.PORT);

        // Enable pin PA14 and PA15 as an external source for XOSC0 at 8 MHz
        let xosc0 = Xosc::from_crystal(
            tokens.xosc0,
            pins.pa14,
            pins.pa15,
            8.mhz(),
            CrystalCurrent::Medium,
        )
        .enable();

        // Take DFLL 48 MHz, divide down to 2 MHz for Gclk1
        let (gclk1, dfll) = gclk::Gclk::new(tokens.gclks.gclk1, clocks.dfll);
        let _gclk1 = gclk1.div(Gclk1Div::Div(24)).enable();

        // Configure DPLL0 to 100 MHz fed from Xosc0 with a predivider of 1
        let (dpll0, _xosc0) = Dpll::from_xosc0(tokens.dpll0, xosc0, 1);

        // Use 4 as source predivider, 8 MHz / (2 * ( 1 + prediv) * 50 = 100 MHz,
        // where prediv = 1
        let dpll0 = dpll0.set_loop_div(50, 0).enable().ok().unwrap();

        // Change Gclk0 from Dfll to Dpll0, MCLK = 100 MHz
        let (gclk0, _dfll, _dpll0) = clocks.gclk0.swap(dfll, dpll0);

        // Output Gclk0 on pin PB14
        let (_gclk_out0, gclk0) =
            GclkOut::enable(tokens.gclk_io.gclk_out0, pins.pb14, gclk0, false);

        // Enable external 32k-oscillator
        let token = tokens.xosc32k.base;
        let base = XoscBase::from_crystal(token, pins.pa00, pins.pa01).enable();
        let (_xosc32k, _base) = Xosc32k::enable(tokens.xosc32k.xosc32k, base);

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
        // Should be 1 second
        let _now = monotonics::now();
        let _ = periodic::spawn_after(TimerDurationU32::from_ticks(SCHEDULE_FREQ));
    }
}
