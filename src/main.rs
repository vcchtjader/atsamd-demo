//! atsamd-demo
#![no_main]
#![no_std]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if let Some(u) = UART0_TX.as_mut() {
            write!(u as &mut dyn Write<_, Error = _>, "{}\r\n", info).unwrap()
        }
    }
    loop {
        cortex_m::asm::nop();
    }
}

use core::fmt::Write as _;

use atsamd_hal::{
    clock::GenericClockController,
    gpio::v2::{Alternate, Pin, Pins, D, PA04, PA05},
    hal::serial::Write,
    prelude::*,
    sercom::v2::{uart::*, IoSet3, Sercom0},
    time::U32Ext,
};
use lzma_rs::decompress::*;
use lzma_rs::*;
use nb::block;

use rtic::app;

pub type Uart0Tx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, TxDuplex>;
pub type Uart0Rx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, RxDuplex>;

static mut UART0_TX: Option<Uart0Tx> = None;

#[app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [FREQM])]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {
        uart0_rx: Uart0Rx,
    }

    #[local]
    struct LocalResources {
        stream: Stream<UartWriter, 4096, 8>,
    }

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

        loop {}

        let mut uart0 = Config::new(
            &mclk,
            device.SERCOM0,
            Pads::default().rx(pins.pa05).tx(pins.pa04),
            clocks.sercom0_core(&gclk0).unwrap().freq(),
        )
        .baud((1 << 20).hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();
        uart0.enable_interrupts(Flags::RXC);

        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "RTIC booted!\r\n"
        )
        .unwrap();

        let (uart0_rx, uart0_tx) = uart0.split();

        unsafe {
            UART0_TX.replace(uart0_tx);
        }

        let stream = Stream::new();

        (
            SharedResources { uart0_rx },
            LocalResources { stream },
            init::Monotonics(),
        )
    }

    #[task(local = [
        stream
    ], capacity = 5)]
    fn uart_handle(cx: uart_handle::Context, uart_data: UartCommand) {
        match uart_data {
            UartCommand::Byte(byte) => {
                cx.local
                    .stream
                    .write_all(&mut UartWriter {}, &[byte])
                    .unwrap();

                if let StreamStatus::Finished = cx.local.stream.get_stream_status() {
                    cx.local.stream.finish(&mut UartWriter {}).unwrap();
                }
            }

            // failure / not supported commands
            other => {
                let uart0_tx =
                    unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
                write!(
                    uart0_tx as &mut dyn Write<_, Error = _>,
                    "error: {:?}\r\n",
                    other
                )
                .unwrap()
            }
        }
    }

    #[task(binds = SERCOM0_2, shared = [uart0_rx], priority = 2)]
    fn uart_interrupt(cx: uart_interrupt::Context) {
        let mut rx = cx.shared.uart0_rx;
        match rx.lock(|rx| block!(rx.read())) {
            Ok(byte) => uart_handle::spawn(UartCommand::Byte(byte)),
            Err(e) => uart_handle::spawn(UartCommand::ReadError(e)),
        }
        .unwrap();
    }
}

pub struct UartWriter {}

impl io::Write for UartWriter {
    fn write(&mut self, data: &[u8]) -> Result<usize, lzma_rs::io::Error> {
        let uart_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        for &value in data.iter() {
            uart_tx.write(value).unwrap();
        }
        Ok(data.len())
    }
    fn flush(&mut self) -> Result<(), lzma_rs::io::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum UartCommand {
    CtrlC,
    Backspace,
    Byte(u8),
    Return,
    BufferFull,
    ReadError(Error),
}
