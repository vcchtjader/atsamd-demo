#![no_main]
#![no_std]

macro_rules! clear_screen {
    ($tx:tt) => {{
        // Sequence:
        // `<ESC>[2J` (clear screen)
        // `<ESC>[H` (cursor to home position)
        // `<ESC>` == 0x1b
        $tx.write_str("\x1b").unwrap();
        $tx.write_str("[2J").unwrap();
        $tx.write_str("\x1b").unwrap();
        $tx.write_str("[H").unwrap();
    }};
}

macro_rules! clear_line {
    ($tx:tt) => {{
        // Sequence:
        // `<ESC>[2K` (clear line)
        // `\r`: CR
        // `<ESC>` == 0x1b
        $tx.write_str("\x1b").unwrap();
        $tx.write_str("[2K").unwrap();
        $tx.write_str("\r").unwrap();
    }};
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        match UART0_TX.as_mut() {
            Some(u) => write!(u as &mut dyn Write<_, Error = _>, "{}\r\n", info).unwrap(),
            None => {}
        }
    }
    loop {
        cortex_m::asm::nop();
    }
}

use core::fmt::Write as _;

use atsamd_hal::{
    clock::GenericClockController,
    dsu::Dsu,
    gpio::v2::{Alternate, Pin, Pins, D, PA04, PA05},
    hal::serial::Write,
    nvm::{smart_eeprom::SmartEepromMode, Nvm},
    prelude::*,
    sercom::v2::{uart::*, IoSet3, Sercom0},
    time::U32Ext,
};
use nb::block;

use rtic::app;
pub type Uart0Tx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, TxDuplex>;
pub type Uart0Rx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, RxDuplex>;

pub type String = heapless::String<256>;

static mut UART0_TX: Option<Uart0Tx> = None;

#[app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [FREQM])]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {
        uart0_rx: Uart0Rx,
        buffer: String,
    }

    #[local]
    struct LocalResources {}

    #[init]
    fn init(cx: init::Context) -> (SharedResources, LocalResources, init::Monotonics()) {
        let mut device = cx.device;

        let pins = Pins::new(device.PORT);

        let mclk = &mut device.MCLK;

        let mut clocks = GenericClockController::with_external_32kosc(
            device.GCLK,
            mclk,
            &mut device.OSC32KCTRL,
            &mut device.OSCCTRL,
            &mut device.NVMCTRL,
        );

        let gclk0 = clocks.gclk0();

        let mut uart0 = Config::new(
            &mclk,
            device.SERCOM0,
            Pads::default().rx(pins.pa05).tx(pins.pa04),
            clocks.sercom0_core(&gclk0).unwrap().freq(),
        )
        .baud(115_200.hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();
        uart0.enable_interrupts(Flags::RXC);

        use atsamd_hal::pukcc::Pukcc;
        let pukcc = Pukcc::enable(mclk).unwrap();
        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "RTIC booted!\r\n"
        )
        .unwrap();

        let modulus = [
            0xd2, 0xf4, 0x9b, 0xde, 0x08, 0x0f, 0x57, 0x0f, 0xc2, 0x4d, 0x4b, 0x59, 0xff, 0x72,
            0xf1, 0xbc, 0x08, 0xd0, 0xbe, 0xde, 0x5f, 0xac, 0xab, 0xa7, 0xa6, 0xc9, 0x6b, 0xec,
            0xff, 0xe4, 0x85, 0xde, 0xfd, 0x8e, 0x93, 0xe3, 0x76, 0x9d, 0xc2, 0x8c, 0x5b, 0xac,
            0x3f, 0xf8, 0x2b, 0xf7, 0xd8, 0x30, 0x3f, 0xf6, 0xc6, 0xde, 0x3e, 0xdf, 0x69, 0x4d,
            0x12, 0x97, 0x71, 0xc1, 0xb2, 0x30, 0xb0, 0x74, 0x07, 0x82, 0x45, 0x15, 0xcc, 0x48,
            0x96, 0xac, 0xb3, 0xe8, 0xad, 0x4b, 0xbf, 0x95, 0xdd, 0x4c, 0xd2, 0xae, 0x2b, 0xe0,
            0x13, 0x49, 0x71, 0x9f, 0x34, 0x65, 0xc0, 0x4b, 0xb0, 0x86, 0x3e, 0x69, 0x87, 0xa9,
            0x42, 0xa7, 0x28, 0x69, 0xf1, 0xa8, 0x30, 0x7e, 0x14, 0xc1, 0x30, 0xa3, 0xc2, 0x36,
            0x3d, 0xcc, 0x27, 0x46, 0x80, 0x60, 0x22, 0xdc, 0xec, 0x94, 0x58, 0x83, 0xaa, 0x4a,
            0x8f, 0xaa, 0x8b, 0x94, 0x0b, 0xad, 0xe0, 0x02, 0xa0, 0x47, 0x58, 0x7f, 0x5a, 0x1a,
            0xc8, 0x71, 0xfa, 0xfc, 0x4c, 0x2e, 0x72, 0xd9, 0xb1, 0x15, 0xf9, 0x88, 0xf9, 0xaf,
            0xd1, 0xc3, 0x36, 0xd0, 0x7e, 0x14, 0x74, 0xb4, 0xd4, 0x36, 0x30, 0xce, 0x02, 0xf8,
            0x86, 0x9f, 0x28, 0x06, 0xe1, 0x5f, 0x93, 0x6e, 0x21, 0xa0, 0xe0, 0xf5, 0xbe, 0x3d,
            0xd7, 0xce, 0xc0, 0x1d, 0x94, 0xba, 0x00, 0xe9, 0xf3, 0x59, 0xa4, 0xa8, 0x5c, 0xfb,
            0xb7, 0x67, 0x34, 0xa8, 0x9a, 0xd9, 0x07, 0xc7, 0x7d, 0x1f, 0xfe, 0xce, 0x24, 0x23,
            0xfe, 0x43, 0xe5, 0x7a, 0x89, 0x38, 0xa4, 0xb5, 0x98, 0x71, 0xbb, 0x01, 0xa0, 0x08,
            0x36, 0x80, 0xd4, 0x4d, 0xfc, 0x1e, 0x2b, 0xcc, 0xb6, 0x40, 0x12, 0xd4, 0x9c, 0xbb,
            0x06, 0x3f, 0x4d, 0x62, 0xc5, 0x6e, 0x8f, 0xbf, 0x01, 0x9d, 0x0e, 0xca, 0xd0, 0x1c,
            0x36, 0x19, 0x42, 0x35_u8,
        ];
        let mut cns = [0_u8; 300];
        pukcc.zp_calculate_cns(&mut cns, &modulus).unwrap();
        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "{:#?}\r\n",
            &cns[..256 + 13]
        )
        .unwrap();

        let (uart0_rx, uart0_tx) = uart0.split();

        unsafe {
            UART0_TX.replace(uart0_tx);
        }

        loop {}

        (
            SharedResources {
                uart0_rx,
                buffer: heapless::String::new(),
            },
            LocalResources {},
            init::Monotonics(),
        )
    }

    #[derive(Debug)]
    pub enum Action {
        Read,
        Write,
    }

    #[task(shared = [buffer], capacity = 10)]
    fn uart_handle(cx: uart_handle::Context, uart_data: UartCommand) {
        let mut buffer = cx.shared.buffer;
        let uart0_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        match uart_data {
            UartCommand::Return => buffer.lock(|b| {}),
            UartCommand::DataChar(character) => buffer.lock(|b| match b.push(character) {
                Ok(_) => {
                    clear_line!(uart0_tx);
                    uart0_tx.write_str(b.as_str()).unwrap();
                }
                Err(_) => uart_handle::spawn(UartCommand::BufferFull).unwrap(),
            }),
            UartCommand::Backspace => buffer.lock(|b| match b.pop() {
                Some(_) => {
                    clear_line!(uart0_tx);
                    uart0_tx.write_str(b.as_str()).unwrap();
                }
                None => (),
            }),
            UartCommand::CtrlC => buffer.lock(|b| {
                b.clear();
                clear_screen!(uart0_tx);
                uart0_tx.write_str(b.as_str()).unwrap();
            }),
            // Failure / Not supported commands
            other => write!(
                uart0_tx as &mut dyn Write<_, Error = _>,
                "Error: {:?}\r\n",
                other
            )
            .unwrap(),
        }
    }

    #[task(binds = SERCOM0_2, shared = [uart0_rx], priority = 2)]
    fn uart_interrupt(cx: uart_interrupt::Context) {
        let mut rx = cx.shared.uart0_rx;
        match rx.lock(|rx| block!(rx.read())) {
            Ok(byte) => match byte as char {
                '\u{7f}' => uart_handle::spawn(UartCommand::Backspace),
                '\u{3}' => uart_handle::spawn(UartCommand::CtrlC),
                '\r' => uart_handle::spawn(UartCommand::Return),
                byte => uart_handle::spawn(UartCommand::DataChar(byte)),
            },
            Err(e) => uart_handle::spawn(UartCommand::ReadError(e)),
        }
        .unwrap();
    }
}

#[derive(Debug)]
pub enum UartCommand {
    CtrlC,
    Backspace,
    DataChar(char),
    Return,
    BufferFull,
    ReadError(Error),
}
