//! atsamd-demo
#![no_main]
#![no_std]

use atsamd_hal::{clock::GenericClockController, gpio::v2::*, icm::*};
use panic_halt as _;
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
#[derive(Debug, PartialEq)]
pub struct RegionCount {
    pub r#match: u32,
    pub mismatch: u32,
}
#[derive(Debug, PartialEq)]
pub struct MatchCount {
    pub region0: RegionCount,
    pub region1: RegionCount,
    pub region2: RegionCount,
    pub region3: RegionCount,
}

#[app(device = atsamd_hal::target_device, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct SharedResources {
        icm: Icm,
    }

    #[local]
    struct LocalResources {
        icm_int_count: (bool, bool, bool, bool),
        icm_match_stats: MatchCount,
        icm_region0: Region<Region0>,
        icm_region1: Region<Region1>,
        icm_region2: Region<Region2>,
        icm_region3: Region<Region3>,
    }

    #[init]
    fn init(cx: init::Context) -> (SharedResources, LocalResources, init::Monotonics) {
        let mut peripherals = cx.device;
        let _core = cx.core;

        let _clocks = GenericClockController::with_external_32kosc(
            peripherals.GCLK,
            &mut peripherals.MCLK,
            &mut peripherals.OSC32KCTRL,
            &mut peripherals.OSCCTRL,
            &mut peripherals.NVMCTRL,
        );
        let _pins = Pins::new(peripherals.PORT);

        // Enable ICM apb clock
        // Clock v1
        let mclk = &peripherals.MCLK;
        mclk.apbcmask.modify(|_, w| w.icm_().set_bit());

        // Create new ICM
        let mut icm = Icm::new(peripherals.ICM);

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

        // Region Descriptor create a new one with intention of
        // replacing ICM_REGION_DESC
        let mut icm_region_desc = Regions::default();

        // Get the interface for Region0 and enable monitoring
        let icm_region0 = icm.enable_region0();
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
        let icm_region1 = icm.enable_region1();

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
        let icm_region2 = icm.enable_region2();

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
        let icm_region3 = icm.enable_region3();

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

        // For tracking interrupts
        let icm_int_count = (false, false, false, false);

        let icm_match_stats = MatchCount {
            region0: RegionCount {
                r#match: 0,
                mismatch: 0,
            },
            region1: RegionCount {
                r#match: 0,
                mismatch: 0,
            },
            region2: RegionCount {
                r#match: 0,
                mismatch: 0,
            },
            region3: RegionCount {
                r#match: 0,
                mismatch: 0,
            },
        };

        cortex_m::asm::bkpt();

        (
            SharedResources { icm },
            LocalResources {
                icm_int_count,
                icm_match_stats,
                icm_region0,
                icm_region1,
                icm_region2,
                icm_region3,
            },
            init::Monotonics(),
        )
    }

    #[task(priority= 3, binds = ICM,
        shared = [icm],
        local = [
                icm_int_count, icm_match_stats,
                icm_region0, icm_region1, icm_region2, icm_region3,
        ])]
    fn icm(cx: icm::Context) {
        let mut icm = cx.shared.icm;
        let icm_int_count = cx.local.icm_int_count;
        let icm_match_stats = cx.local.icm_match_stats;

        // Get a parseable copy of the interrupt status vector
        let icminterrupt = icm.lock(|i| i.get_interrupt_status());

        if icminterrupt
            .get_rhc_int()
            .intersects(RegionHashCompleted::R0)
        {
            icm_int_count.0 = true;
        }

        if icminterrupt
            .get_rhc_int()
            .intersects(RegionHashCompleted::R1)
        {
            icm_int_count.1 = true;
        }
        if icminterrupt
            .get_rhc_int()
            .intersects(RegionHashCompleted::R2)
        {
            icm_int_count.2 = true;
        }
        if icminterrupt
            .get_rhc_int()
            .intersects(RegionHashCompleted::R3)
        {
            icm_int_count.3 = true;
        }

        // Only run when all the hashes has been fully calculated
        if *icm_int_count == (true, true, true, true) {
            // Region0 should be 5 mismatches and 3 matches
            let region0_expected_values = RegionCount {
                mismatch: 5,
                r#match: 3,
            };
            // Region1 should be 0 mismatches and 8 matches
            let region1_expected_values = RegionCount {
                mismatch: 0,
                r#match: 8,
            };
            // Region2 should be 0 mismatches and 8 matches
            let region2_expected_values = RegionCount {
                mismatch: 0,
                r#match: 8,
            };
            // Region3 should be 0 mismatches and 8 matches
            let region3_expected_values = RegionCount {
                mismatch: 0,
                r#match: 8,
            };

            // Region0
            for (index, val) in MESSAGE_SHA1_RES.iter().enumerate() {
                unsafe {
                    if *val == HASH.region0[index] {
                        // Matched word
                        icm_match_stats.region0.r#match += 1;
                    } else {
                        // Mismatched word (u32)
                        icm_match_stats.region0.mismatch += 1;
                    }
                }
            }
            // Region1
            for (index, val) in MESSAGE_SHA1_RES.iter().enumerate() {
                unsafe {
                    if *val == HASH.region1[index] {
                        // Matched word
                        icm_match_stats.region1.r#match += 1;
                    } else {
                        // Mismatched word (u32)
                        icm_match_stats.region1.mismatch += 1;
                    }
                }
            }
            // Region2
            for (index, val) in MESSAGE_SHA224_RES.iter().enumerate() {
                unsafe {
                    if *val == HASH.region2[index] {
                        // Matched word
                        icm_match_stats.region2.r#match += 1;
                    } else {
                        // Mismatched word (u32)
                        icm_match_stats.region2.mismatch += 1;
                    }
                }
            }
            // Region3
            for (index, val) in MESSAGE_SHA256_RES.iter().enumerate() {
                unsafe {
                    if *val == HASH.region3[index] {
                        // Matched word
                        icm_match_stats.region3.r#match += 1;
                    } else {
                        // Mismatched word (u32)
                        icm_match_stats.region3.mismatch += 1;
                    }
                }
            }

            // GDB can further inspect the `icm_match_stats` variable
            // with
            //
            // `print icm_clockv1::app::__rtic_internal_local_resource_icm_match_stats`
            if icm_match_stats.region0 == region0_expected_values {
                cortex_m::asm::bkpt();
            }
            if icm_match_stats.region1 == region1_expected_values {
                cortex_m::asm::bkpt();
            }
            if icm_match_stats.region2 == region2_expected_values {
                cortex_m::asm::bkpt();
            }
            if icm_match_stats.region3 == region3_expected_values {
                cortex_m::asm::bkpt();
            }
            // Clear the accumulative interrupt state
            *icm_int_count = (false, false, false, false);
        }
    }
}
