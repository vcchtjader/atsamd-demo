//! atsamd-demo using clockv1
#![no_std]
#![no_main]

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

use atsamd_demo::{clear_line, clear_screen, uart::*};

use core::fmt::Write as _;

use atsamd_hal::{
    clock::GenericClockController,
    dsu::Dsu,
    gpio::v2::Pins,
    hal::serial::Write,
    nvm::{smart_eeprom::SmartEepromMode, Nvm},
    prelude::*,
    time::U32Ext,
};
use nb::block;

use rtic::app;

static mut UART0_TX: Option<Uart0Tx> = None;

#[app(device = atsamd_hal::target_device, peripherals = true, dispatchers = [FREQM])]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {
        uart0_rx: Uart0Rx,
        nvm: Nvm,
        dsu: Dsu,
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

        let nvm = Nvm::new(device.NVMCTRL);
        let dsu = Dsu::new(device.DSU, &device.PAC).unwrap();
        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "RTIC booted!\r\n"
        )
        .unwrap();

        let (uart0_rx, uart0_tx) = uart0.split();

        unsafe {
            UART0_TX.replace(uart0_tx);
        }

        (
            SharedResources {
                uart0_rx,
                nvm,
                dsu,
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

    #[task(shared = [buffer, nvm], capacity = 10)]
    fn uart_handle(cx: uart_handle::Context, uart_data: UartCommand) {
        let mut buffer = cx.shared.buffer;
        let mut nvm = cx.shared.nvm;
        let uart0_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        match uart_data {
            UartCommand::Return => {
                buffer.lock(|b| {
                    uart0_tx.write_str("\r\n").unwrap();
                    // custom action start

                    let mut iterator = b.split_whitespace();
                    let (action, arg1, arg2) = match iterator
                        .next()
                        .and_then(|v| match v {
                            "r" => Some(Action::Read),
                            "w" => Some(Action::Write),
                            _ => None,
                        })
                        .and_then(|action| {
                            iterator
                                .next()
                                .and_then(|arg1| arg1.parse().ok())
                                .and_then(|arg1| {
                                    iterator
                                        .next()
                                        .and_then(|arg2| arg2.parse().ok())
                                        .map(|arg2| (action, arg1, arg2))
                                })
                        }) {
                        Some(v) => v,
                        None => {
                            uart0_tx.write_str("argument parsing failure\r\n").unwrap();
                            b.clear();
                            return;
                        }
                    };

                    nvm.lock(|n| {
                        let mut se = match n.smart_eeprom().unwrap() {
                            SmartEepromMode::Unlocked(se) => se,
                            SmartEepromMode::Locked(se) => se.unlock(),
                        };
                        match action {
                            Action::Read => {
                                se.iter::<u8>().enumerate().skip(arg1).take(arg2).for_each(
                                    |(i, v)| {
                                        write!(
                                            uart0_tx as &mut dyn Write<_, Error = _>,
                                            "{:0x}: {:0x}\r\n",
                                            i, v
                                        )
                                        .unwrap()
                                    },
                                )
                            }
                            Action::Write => {
                                use core::convert::TryInto;
                                uart0_tx.write_str("writing to eeprom..\r\n").unwrap();
                                se.iter_mut::<u8>()
                                    .skip(arg1)
                                    .take(arg2)
                                    .for_each(|v| *v = (arg2 & 0xff_usize).try_into().unwrap());
                                uart0_tx.write_str("done\r\n").unwrap();
                            }
                        }
                    });

                    b.clear();
                })
            }
            UartCommand::DataChar(character) => buffer.lock(|b| match b.push(character) {
                Ok(_) => {
                    clear_line!(uart0_tx);
                    uart0_tx.write_str(b.as_str()).unwrap();
                }
                Err(_) => uart_handle::spawn(UartCommand::BufferFull).unwrap(),
            }),
            UartCommand::Backspace => buffer.lock(|b| {
                if b.pop().is_some() {
                    clear_line!(uart0_tx);
                    uart0_tx.write_str(b.as_str()).unwrap();
                }
            }),
            UartCommand::CtrlC => buffer.lock(|b| {
                b.clear();
                clear_screen!(uart0_tx);
                uart0_tx.write_str(b.as_str()).unwrap();
            }),
            // failure / not supported commands
            other => write!(
                uart0_tx as &mut dyn Write<_, Error = _>,
                "error: {:?}\r\n",
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

    // Only assign dsu to silence an unused warning
    #[idle(shared = [dsu])]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
