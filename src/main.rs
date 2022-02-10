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
    gpio::v2::{Alternate, Pin, Pins, C, D, PA04, PA05, PA16, PA17, PB08, PB09},
    hal::serial::Write,
    prelude::*,
    sercom::v2::{uart::*, IoSet1, IoSet2, IoSet3, Sercom0, Sercom1, Sercom4},
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
pub type Uart1Tx =
    Uart<Config<Pads<Sercom1, IoSet1, Pin<PA17, Alternate<C>>, Pin<PA16, Alternate<C>>>>, TxDuplex>;
pub type Uart1Rx =
    Uart<Config<Pads<Sercom1, IoSet1, Pin<PA17, Alternate<C>>, Pin<PA16, Alternate<C>>>>, RxDuplex>;
pub type Uart2Tx =
    Uart<Config<Pads<Sercom4, IoSet2, Pin<PB09, Alternate<D>>, Pin<PB08, Alternate<D>>>>, TxDuplex>;
pub type Uart2Rx =
    Uart<Config<Pads<Sercom4, IoSet2, Pin<PB09, Alternate<D>>, Pin<PB08, Alternate<D>>>>, RxDuplex>;

static mut UART0_TX: Option<Uart0Tx> = None;
static mut UART1_TX: Option<Uart1Tx> = None;
static mut UART2_TX: Option<Uart2Tx> = None;

#[app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [FREQM])]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {}

    #[local]
    struct LocalResources {
        stream: Stream<UartWriter, 4096, 8>,
        uart1_rx: Uart1Rx,
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

        let mut uart0 = Config::new(
            &mclk,
            device.SERCOM0,
            Pads::default().rx(pins.pa05).tx(pins.pa04),
            clocks.sercom0_core(&gclk0).unwrap().freq(),
        )
        .baud(4_000_000.hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();
        let mut uart1 = Config::new(
            &mclk,
            device.SERCOM1,
            Pads::default().rx(pins.pa17).tx(pins.pa16),
            clocks.sercom1_core(&gclk0).unwrap().freq(),
        )
        .baud(35_000.hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();
        uart1.enable_interrupts(Flags::RXC);
        let uart2 = Config::new(
            &mclk,
            device.SERCOM4,
            Pads::default().rx(pins.pb09).tx(pins.pb08),
            clocks.sercom4_core(&gclk0).unwrap().freq(),
        )
        .baud(4_000_000.hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();

        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "RTIC booted!\r\n"
        )
        .unwrap();

        let (_uart0_rx, uart0_tx) = uart0.split();
        let (uart1_rx, uart1_tx) = uart1.split();
        let (_uart2_rx, uart2_tx) = uart2.split();

        unsafe {
            UART0_TX.replace(uart0_tx);
            UART1_TX.replace(uart1_tx);
            UART2_TX.replace(uart2_tx);
        }

        let stream = Stream::new();

        (
            SharedResources {},
            LocalResources { stream, uart1_rx },
            init::Monotonics(),
        )
    }

    #[task(local = [
        stream
    ], capacity = 255)]
    fn uart_handle(mut cx: uart_handle::Context, uart_data: UartCommand) {
        let uart0_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        let uart2_tx = unsafe { &mut UART2_TX };
        let stream = &mut cx.local.stream;
        let mut uw = UartWriter {
            inner: uart2_tx.take().unwrap(),
        };
        match uart_data {
            UartCommand::Byte(byte) => {
                stream.write_all(&mut uw, &[byte]).unwrap();
                write!(
                    uart0_tx,
                    "uart_handle: Byte 0x{:>2X} | Status: {:?}\r\n",
                    byte,
                    stream.get_stream_status()
                )
                .unwrap();
                if stream.get_stream_status() == lzma_rs::decompress::StreamStatus::EosReached {
                    stream.finish(&mut uw).unwrap();
                    write!(
                        uart0_tx,
                        "uart_handle: /finished/ | Status: {:?}\r\n",
                        stream.get_stream_status()
                    )
                    .unwrap();
                }
            }

            // failure / not supported commands
            other => write!(uart0_tx, "uart_handle: /error/ | {:?}\r\n", other).unwrap(),
        }
        let UartWriter { inner } = uw;
        uart2_tx.replace(inner);
    }

    #[task(binds = SERCOM1_2, local = [uart1_rx], priority = 2)]
    fn uart_interrupt(cx: uart_interrupt::Context) {
        let rx = cx.local.uart1_rx;
        let r = match block!(rx.read()) {
            Ok(byte) => uart_handle::spawn(UartCommand::Byte(byte)),
            Err(e) => uart_handle::spawn(UartCommand::ReadError(e)),
        };
        r.unwrap();
    }
}

pub struct UartWriter {
    inner: Uart2Tx,
}

impl io::Write for UartWriter {
    fn write(&mut self, data: &[u8]) -> Result<usize, lzma_rs::io::Error> {
        let uart0_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        write!(
            uart0_tx,
            "UartWriter::write ({} bytes)",
            data.len()
        ).unwrap();
        for &value in data.iter() {
            write!(
                uart0_tx,
                ".",
            ).unwrap();
            block!(self.inner.write(value)).unwrap();
        }
        write!(
            uart0_tx,
            "\r\n",
        ).unwrap();
        Ok(data.len())
    }
    fn flush(&mut self) -> Result<(), lzma_rs::io::Error> {
        let uart0_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        write!(
            uart0_tx,
            "UartWriter::flush\r\n",
        ).unwrap();
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
