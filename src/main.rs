#![no_main]
#![no_std]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        match UART.as_mut() {
            Some(u) => writeln!(u as &mut dyn Write<_, Error = _>, "{}", info).unwrap(),
            None => {}
        }
    }
    loop {
        cortex_m::asm::nop();
    }
}

use core::fmt::Write as _;

use atsamd_hal::{
    clock::v2::{pclk::Pclk, retrieve_clocks, xosc::Xosc, dpll::Dpll},
    gpio::v2::{Alternate, Pin, Pins, D, PA04, PA05, PA08, PA09},
    hal::serial::Write,
    prelude::*,
    sercom::{
        v2::{uart::*, IoSet3, Sercom0},
        I2CMaster2,
    },
    time::U32Ext,
};

use eeprom24x::{addr_size::OneByte, page_size::B8, Eeprom24x, SlaveAddr};

use rtic::app;
pub type Uart0 =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, Duplex>;
pub type I2C = I2CMaster2<Pin<PA09, Alternate<D>>, Pin<PA08, Alternate<D>>>;

static mut UART: Option<Uart0> = None;

#[app(device = atsamd_hal::target_device, peripherals = true )]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {
        eeprom: Eeprom24x<I2C, B8, OneByte>,
    }

    #[local]
    struct LocalResources {}

    #[init]
    fn init(cx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
        let mut device = cx.device;

        let pins = Pins::new(device.PORT);

        let (gclk0, dfll, _, tokens) = retrieve_clocks(
            device.OSCCTRL,
            device.OSC32KCTRL,
            device.GCLK,
            device.MCLK,
            &mut device.NVMCTRL,
        );

        let (_, _, _, mut mclk) = unsafe { tokens.pac.steal() };
        let xosc0 = Xosc::from_crystal(tokens.xosc0, pins.pa14, pins.pa15, 8.mhz()).enable();

        let (dpll0, _xosc0) = Dpll::from_xosc(tokens.dpll0, xosc0, 1);
        let dpll0 = dpll0.set_loop_div(95, 0).enable();

        let (gclk0, _dfll, _dpll0) = gclk0.swap(dfll, dpll0);

        let (pclk_sercom0, gclk0) = Pclk::enable(tokens.pclks.sercom0, gclk0);
        let (pclk_sercom2, _gclk0) = Pclk::enable(tokens.pclks.sercom2, gclk0);
        let pclk_sercom2 = pclk_sercom2.into();

        let mut uart0 = Config::new(
            &mclk,
            device.SERCOM0,
            Pads::default().rx(pins.pa05).tx(pins.pa04),
            pclk_sercom0.freq(),
        )
        .baud(115_200.hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();
        uart0.enable_interrupts(Flags::RXC);

        writeln!(&mut uart0 as &mut dyn Write<_, Error = _>, "RTIC booted!").unwrap();

        unsafe {
            UART.replace(uart0);
        }

        let i2c = I2CMaster2::new_with_timeout(
            &pclk_sercom2,
            400.khz(),
            device.SERCOM2,
            &mut mclk,
            pins.pa09.into_mode::<Alternate<D>>(),
            pins.pa08.into_mode::<Alternate<D>>(),
            100,
        );

        let eeprom = Eeprom24x::new_24x02(i2c, SlaveAddr::Alternative(true, true, false));

        (
            SharedResources { eeprom },
            LocalResources {},
            init::Monotonics(),
        )
    }

    #[task(binds = SERCOM0_2, shared = [eeprom])]
    fn uart(cx: uart::Context) {
        let uart0 = unsafe { UART.as_mut().unwrap() };
        let _ = uart0.read().unwrap();
        let mut eeprom = cx.shared.eeprom;

        // eeprom.lock(|e| {
        //     e.write_page(0x0, &[0xDE, 0xAD, 0xBE, 0xEF, 0xFF, 0xFF, 0xFF, 0xFF])
        //         .unwrap()
        // });
        //
        let mut buffer = [0_u8; 8];

        match eeprom.lock(|e| e.read_data(0x0, &mut buffer)) {
            Ok(_) => writeln!(uart0 as &mut dyn Write<_, Error = _>, "{:#02X?}", buffer).unwrap(),
            Err(e) => writeln!(
                uart0 as &mut dyn Write<_, Error = _>,
                "Read data failure: {:#?}",
                e
            )
            .unwrap(),
        };
    }
}
