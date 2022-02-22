//! UART Related types and config

cfg_if::cfg_if! {
    if #[cfg(feature = "clockv1")] {
        pub use atsamd_hal_clockv1::{
            gpio::v2::{Alternate, Pin, Pins, D, PA04, PA05},
            hal::serial::Write,
            sercom::v2::{uart::*, IoSet3, Sercom0},
        };

    } else if #[cfg(feature = "hal-aes")] {
        pub use atsamd_hal_aes::{
            gpio::v2::{Alternate, Pin, Pins, D, PA04, PA05},
            hal::serial::Write,
            sercom::v2::{uart::*, IoSet3, Sercom0},
        };
    } else {
        pub use atsamd_hal::{
            gpio::v2::{Alternate, Pin, Pins, D, PA04, PA05},
            hal::serial::Write,
            sercom::v2::{uart::*, IoSet3, Sercom0},
        };
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

pub type Uart0Tx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, TxDuplex>;
pub type Uart0Rx =
    Uart<Config<Pads<Sercom0, IoSet3, Pin<PA05, Alternate<D>>, Pin<PA04, Alternate<D>>>>, RxDuplex>;

pub type String = heapless::String<256>;

#[macro_export]
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

#[macro_export]
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
