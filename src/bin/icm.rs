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

use atsamd_hal_clockv1::{
    clock::GenericClockController,
    gpio::Pins,
    hal::serial::Write,
    icm::*,
    nvm::{smart_eeprom::SmartEepromMode, Nvm},
    prelude::*,
    time::U32Ext,
};
use nb::block;

use rtic::app;

// SHA Test data
static MESSAGE_REF0: [u32; 16] = [
    0x11111111, 0x22222222, 0x33333333, 0x44444444, 0x55555555, 0x66666666, 0x77777777, 0x88888888,
    0x99999999, 0xaaaaaaaa, 0xbbbbbbbb, 0xcccccccc, 0xdddddddd, 0xeeeeeeee, 0xffffffff, 0x00000000,
];

static MESSAGE_REF1: [u32; 16] = [
    0x80636261, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x18000000,
];

// Expected SHA1 sum result
static MESSAGE_SHA1_RES: [u32; 8] = [
    0x363e99a9, 0x6a810647, 0x71253eba, 0x6cc25078, 0x9dd8d09c, 0x00000000, 0x00000000, 0x00000000,
];

static MESSAGE_SHA224_RES: [u32; 8] = [
    0x227d0923, 0x22d80534, 0x77a44286, 0xb355a2bd, 0xe4bcad2a, 0xf7b3a0bd, 0xa79d6ce3, 0x00000000,
];
static MESSAGE_SHA256_RES: [u32; 8] = [
    0xbf1678ba, 0xeacf018f, 0xde404141, 0x2322ae5d, 0xa36103b0, 0x9c7a1796, 0x61ff10b4, 0xad1500f2,
];

static mut HASH: HashArea = HashArea::default();
static mut ICM_REGION_DESC: Regions = Regions::default();

static mut UART0_TX: Option<Uart0Tx> = None;

#[app(device = atsamd_hal_clockv1::pac, peripherals = true, dispatchers = [FREQM])]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {
        uart0_rx: Uart0Rx,
        nvm: Nvm,
        buffer: String,
        icm: Icm,
    }

    #[local]
    struct LocalResources {
        icm_region0: Region<Region0>,
        icm_region1: Region<Region1>,
        icm_region2: Region<Region2>,
        icm_region3: Region<Region3>,
        message_region0_sha1: [u32; 16],
        message_region1_sha1: [u32; 16],
        message_region2_sha224: [u32; 16],
        message_region3_sha256: [u32; 16],
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
            mclk,
            device.SERCOM0,
            Pads::default().rx(pins.pa05).tx(pins.pa04),
            clocks.sercom0_core(&gclk0).unwrap().freq(),
        )
        .baud(115_200.hz(), BaudMode::Arithmetic(Oversampling::Bits16))
        .enable();
        uart0.enable_interrupts(Flags::RXC);

        let nvm = Nvm::new(device.NVMCTRL);
        write!(
            &mut uart0 as &mut dyn Write<_, Error = _>,
            "RTIC booted!\r\n"
        )
        .unwrap();

        let (uart0_rx, uart0_tx) = uart0.split();

        unsafe {
            UART0_TX.replace(uart0_tx);
        }

        // Enable ICM apb clock
        // Clock v1
        mclk.apbcmask.modify(|_, w| w.icm_().set_bit());
        // Clock v2
        //tokens.apbs.icm.enable();

        // Create new ICM
        let mut icm = Icm::new(device.ICM);

        // Reset the ICM, clearing past error states
        icm.swrst();

        // End of Monitoring is permitted
        icm.set_eomdis(false);
        // Write Back is permitted
        icm.set_wbdis(false);
        // Secondary List branching is forbidden
        icm.set_slbdis(false);
        // Automatic Switch to Compare is disabled
        icm.set_ascd(false);

        // Test setting user initial hash value
        icm.set_user_initial_hash_value(&MESSAGE_SHA1_RES);

        // Region Descriptor create a new one with intention of
        // replacing ICM_REGION_DESC
        let mut icm_region_desc = Regions::default();

        // Get the interface for Region0 and enable monitoring
        let icm_region0: Region<Region0> = icm.get_region_handle();
        icm_region0.enable_monitoring();

        // Setup desired interrupts
        //
        // Region Hash Completed
        icm_region0.set_rhc_int();

        // Region0 raddr
        icm_region_desc
            .region0
            .set_region_address(MESSAGE_REF0.as_ptr());

        // Configure the RCFG

        // Some are default values, just as an example

        // Activate Write back (should be true when comparing memory)
        icm_region_desc.region0.rcfg.set_cdwbn(false);
        // Should the ICM controller loop back to DSCR after this region?
        icm_region_desc.region0.rcfg.set_wrap(false);
        // Set this as the end of descriptor linked list
        icm_region_desc.region0.rcfg.set_eom(false);
        // The RHC flag is set when the field NEXT = 0
        // in a descriptor of the main or second list
        icm_region_desc.region0.rcfg.set_rhien(false);
        // Set Algorithm to SHA1
        icm_region_desc.region0.rcfg.set_algo(icm_algorithm::SHA1);

        // Get the interface for region1
        let icm_region1 = icm.get_region1_handle();

        // Enable region monitoring
        icm_region1.enable_monitoring();

        // Setup desired interrupts
        //
        // Region Hash Completed
        icm_region1.set_rhc_int();

        // Region1 raddr
        icm_region_desc
            .region1
            .set_region_address(MESSAGE_REF1.as_ptr());

        // Configure the RCFG
        // The RHC flag is set when the field NEXT = 0
        // in a descriptor of the main or second list
        icm_region_desc.region1.rcfg.set_rhien(false);
        // Set Algorithm to SHA1
        icm_region_desc.region1.rcfg.set_algo(icm_algorithm::SHA1);

        // Get the interface for region2
        let icm_region2 = icm.get_region2_handle();

        // Enable region monitoring
        icm_region2.enable_monitoring();

        // Setup desired interrupts
        //
        // Region Hash Completed
        icm_region2.set_rhc_int();

        // Region2 raddr
        icm_region_desc
            .region2
            .set_region_address(MESSAGE_REF1.as_ptr());

        // Configure the RCFG
        // The RHC flag is set when the field NEXT = 0
        // in a descriptor of the main or second list
        icm_region_desc.region2.rcfg.set_rhien(false);
        // Set Algorithm to SHA224
        icm_region_desc.region2.rcfg.set_algo(icm_algorithm::SHA224);

        // Get the interface for region3
        let icm_region3 = icm.get_region3_handle();

        // Enable region monitoring
        icm_region3.enable_monitoring();

        // Setup desired interrupts
        //
        // Region Hash Completed
        icm_region3.set_rhc_int();

        // Region3 raddr
        icm_region_desc
            .region3
            .set_region_address(MESSAGE_REF1.as_ptr());

        // Configure the RCFG
        //
        // Set this as the end of descriptor linked list
        icm_region_desc.region3.rcfg.set_eom(true);
        // The RHC flag is set when the field NEXT = 0
        // in a descriptor of the main or second list
        icm_region_desc.region3.rcfg.set_rhien(false);
        // Set Algorithm to SHA256
        icm_region_desc.region3.rcfg.set_algo(icm_algorithm::SHA256);

        // Safe because Interrupts are disabled in RTIC Init
        unsafe {
            // Hash Area
            // Set HASH addr to the beginning of the Hash area
            icm.set_hash_addr(&HASH);
        }

        unsafe {
            // Move the icm_region_desc into static
            ICM_REGION_DESC = icm_region_desc;
            // Set DSCR to the beginning of the region descriptor
            icm.set_dscr_addr(&ICM_REGION_DESC.region0);
            // the same but via helper function
            //ICM_REGION_DESC.region0.set_dscr_addr(&icm);
        }

        // Start the ICM calculation
        icm.enable();

        // Setup the compare regions
        let message_region0_sha1 = MESSAGE_REF0;
        let message_region1_sha1 = MESSAGE_REF1;
        let message_region2_sha224 = MESSAGE_REF1;
        let message_region3_sha256 = MESSAGE_REF1;

        //cortex_m::asm::bkpt();
        (
            SharedResources {
                uart0_rx,
                nvm,
                buffer: heapless::String::new(),
                icm,
            },
            LocalResources {
                icm_region0,
                icm_region1,
                icm_region2,
                icm_region3,
                message_region0_sha1,
                message_region1_sha1,
                message_region2_sha224,
                message_region3_sha256,
            },
            init::Monotonics(),
        )
    }

    #[derive(Debug)]
    pub enum Action {
        Read,
        Write,
    }

    #[task(priority= 3, binds = ICM,
        shared = [buffer, icm],
        local = [icm_region0, icm_region1, icm_region2, icm_region3,
        message_region0_sha1, message_region1_sha1, message_region2_sha224,
        message_region3_sha256
        ])]
    fn icm(cx: icm::Context) {
        let uart0_tx = unsafe { UART0_TX.as_mut().unwrap() as &mut dyn Write<_, Error = _> };
        let mut icm = cx.shared.icm;
        let icm_region0 = cx.local.icm_region0;
        let icm_region1 = cx.local.icm_region1;
        let icm_region2 = cx.local.icm_region2;
        let icm_region3 = cx.local.icm_region3;

        uart0_tx.write_str("ICM Interrupt!\r\n").unwrap();

        // Get a parseable copy of the interrupt status vector
        let icminterrupt = icm.lock(|i| i.get_interrupt_status());
        //cortex_m::asm::bkpt();

        // Check that all hashes has been computed
        if icminterrupt.get_rhc_int().is_all() {
            // Use the RHC-mask to toggle between Write Back
            // and Digest Compare modes
            if icm_region0.get_rhc_int_mask() {
                // Disable RHC interrupts
                icm_region0.disable_rhc_int();
                icm_region1.disable_rhc_int();
                icm_region2.disable_rhc_int();
                icm_region3.disable_rhc_int();
            }

            uart0_tx
                .write_str("Region 0: Expected,  Actual - SHA1 (should mismatch)\r\n")
                .unwrap();

            for (index, val) in MESSAGE_SHA1_RES.iter().enumerate() {
                unsafe {
                    let cmp = HASH.region0[index];
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }
            uart0_tx
                .write_str("Region 1: Expected,  Actual - SHA1 (should match)\r\n")
                .unwrap();
            for (index, val) in MESSAGE_SHA1_RES.iter().enumerate() {
                unsafe {
                    let cmp = HASH.region1[index];
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }
            uart0_tx
                .write_str("Region 2: Expected,  Actual - SHA224 (should match)\r\n")
                .unwrap();
            for (index, val) in MESSAGE_SHA224_RES.iter().enumerate() {
                unsafe {
                    let cmp = HASH.region2[index];
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }

            uart0_tx
                .write_str("Region 3: Expected,  Actual - SHA256 (should match)\r\n")
                .unwrap();
            for (index, val) in MESSAGE_SHA256_RES.iter().enumerate() {
                unsafe {
                    let cmp = HASH.region3[index];
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }

            // Reconfigure ICM to watch and compare memory instead
            uart0_tx.write_str("Done!").unwrap();

            uart0_tx
                .write_str(" Switch to region monitoring mode")
                .unwrap();
            icm.lock(|i| i.swrst());

            // Create temporary Region
            let mut icm_region_desc = Regions::default();

            // Setup region 0 to monitor memory
            icm_region_desc
                .region0
                .set_region_address(cx.local.message_region0_sha1);
            icm_region_desc
                .region0
                .rcfg
                .reset_region_configuration_to_default();
            icm_region_desc.region0.rcfg.set_algo(icm_algorithm::SHA1);
            // Activate Compare Digest (should be true when comparing memory)
            icm_region_desc.region0.rcfg.set_cdwbn(true);
            // Digest Mismatch Interrupt Disable (enabled)
            icm_region_desc.region0.rcfg.set_dmien(false);

            // Set Region Mismatch Interrupt
            icm_region0.set_rdm_int();

            // Setup region 1 to monitor memory
            icm_region_desc
                .region1
                .set_region_address(cx.local.message_region1_sha1);
            icm_region_desc
                .region1
                .rcfg
                .reset_region_configuration_to_default();
            icm_region_desc.region1.rcfg.set_algo(icm_algorithm::SHA1);
            // Activate Compare Digest (should be true when comparing memory)
            icm_region_desc.region1.rcfg.set_cdwbn(true);
            // Digest Mismatch Interrupt Disable (enabled)
            icm_region_desc.region1.rcfg.set_dmien(false);

            // Set Region Mismatch Interrupt
            icm_region1.set_rdm_int();

            // Setup region 2 to monitor memory
            icm_region_desc
                .region2
                .set_region_address(cx.local.message_region2_sha224);
            icm_region_desc
                .region2
                .rcfg
                .reset_region_configuration_to_default();
            icm_region_desc.region2.rcfg.set_algo(icm_algorithm::SHA224);
            // Activate Compare Digest (should be true when comparing memory)
            icm_region_desc.region2.rcfg.set_cdwbn(true);
            // Digest Mismatch Interrupt Disable (enabled)
            icm_region_desc.region2.rcfg.set_dmien(false);

            // Set Region Mismatch Interrupt
            icm_region2.set_rdm_int();

            // Setup region 3 to monitor memory
            icm_region_desc
                .region3
                .set_region_address(cx.local.message_region3_sha256);
            icm_region_desc
                .region3
                .rcfg
                .reset_region_configuration_to_default();
            icm_region_desc.region3.rcfg.set_algo(icm_algorithm::SHA256);
            // Activate Compare Digest (should be true when comparing memory)
            icm_region_desc.region3.rcfg.set_cdwbn(true);
            // Digest Mismatch Interrupt Disable (enabled)
            icm_region_desc.region3.rcfg.set_dmien(false);
            // Wrap
            icm_region_desc.region3.rcfg.set_wrap(true);

            // Set Region Mismatch Interrupt
            icm_region3.set_rdm_int();

            // Modify regions to trigger interrupts
            uart0_tx.write_str("Manually modify region0\r\n").unwrap();
            cx.local.message_region0_sha1[3] = 0xDEAD_BEEF;
            uart0_tx.write_str("Manually modify region1\r\n").unwrap();
            cx.local.message_region1_sha1[4] = 0xDEAD_BEEF;
            uart0_tx.write_str("Manually modify region2\r\n").unwrap();
            cx.local.message_region2_sha224[5] = 0xDEAD_BEEF;
            uart0_tx.write_str("Manually modify region3\r\n").unwrap();
            cx.local.message_region3_sha256[6] = 0xDEAD_BEEF;

            // Copy the configured Regions into the static mut ICM is reading
            unsafe {
                ICM_REGION_DESC = icm_region_desc;
            }

            icm.lock(|i| i.enable());
        } else if icminterrupt.get_rdm_int().is_all() {
            if icminterrupt
                .get_rdm_int()
                .intersects(RegionDigestMismatch::R0)
            {
                uart0_tx.write_str("Region0 digest mismatch!\r\n").unwrap();
                // Disable the interrupt
                icm_region0.disable_rdm_int();

                uart0_tx.write_str("Region 0: Expected,  Actual\r\n").unwrap();
                for (index, val) in MESSAGE_REF0.iter().enumerate() {
                    let cmp = cx.local.message_region0_sha1[index];
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }
            if icminterrupt
                .get_rdm_int()
                .intersects(RegionDigestMismatch::R1)
            {
                uart0_tx.write_str("Region1 digest mismatch!\r\n").unwrap();
                // Disable the interrupt
                icm_region1.disable_rdm_int();

                uart0_tx.write_str("Region 1: Expected,  Actual\r\n").unwrap();
                for (index, val) in MESSAGE_REF1.iter().enumerate() {
                    let cmp = cx.local.message_region1_sha1[index];
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }
            if icminterrupt
                .get_rdm_int()
                .intersects(RegionDigestMismatch::R2)
            {
                uart0_tx.write_str("Region2 digest mismatch!\r\n").unwrap();
                // Disable the interrupt
                icm_region2.disable_rdm_int();

                uart0_tx.write_str("Region 2: Expected,  Actual\r\n").unwrap();
                for (index, val) in MESSAGE_REF1.iter().enumerate() {
                    let cmp = cx.local.message_region2_sha224[index];
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }
            if icminterrupt
                .get_rdm_int()
                .intersects(RegionDigestMismatch::R3)
            {
                uart0_tx.write_str("Region3 digest mismatch!\r\n").unwrap();
                // Disable the interrupt
                icm_region3.disable_rdm_int();

                uart0_tx.write_str("Region 3: Expected,  Actual\r\n").unwrap();
                for (index, val) in MESSAGE_REF1.iter().enumerate() {
                    let cmp = cx.local.message_region3_sha256[index];
                    //print_compare(uart0_tx, val, cmp);
                    if *val == cmp {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "   Match! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    } else {
                        write!(
                            uart0_tx as &mut dyn Write<_, Error = _>,
                            "mismatch! {:#010x} {:#010x}\r\n",
                            *val, cmp
                        )
                        .unwrap();
                    }
                }
            }

            // Get and clear
            let icminterrupt = icm.lock(|i| i.get_interrupt_status());

            let rdm_ints = icminterrupt.get_rdm_int();
            write!(
                uart0_tx as &mut dyn Write<_, Error = _>,
                "RDM interrupt vector {:04b}\r\n",
                rdm_ints
            )
            .unwrap();
        }
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

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}

//fn print_compare(val: &u32, cmp: u32) -> Result<(), _> {
    //Ok(())
//}
