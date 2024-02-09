#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use flash_algorithm::*;

use rom_api::{flexspi_nor_config_t, serial_nor_config_option_t};

mod chip;
mod rom_api;

#[cfg(feature = "miniz")]
mod inflate;

#[cfg(feature = "log")]
mod log;

#[cfg(feature = "log")]
#[allow(unused_macros)]
macro_rules! dprintln {
    () => {
        ufmt::uwriteln!($crate::log::LOG, "").ok()
    };
    ($fmt:literal) => {
        ufmt::uwriteln!($crate::log::LOG, $fmt).ok()
    };
    ($fmt:literal, $($arg:tt)*) => {
        ufmt::uwriteln!($crate::log::LOG, $fmt, $($arg)*).ok()
    };
}

#[cfg(not(feature = "log"))]
#[allow(unused_macros)]
macro_rules! dprintln {
    () => {};
    ($fmt:expr) => {};
    ($fmt:expr, $($arg:tt)*) => {};
}

pub(crate) use dprintln;

/// The ROM APIs only support 1 single FLASH device connected to PORTA and FLEXSPIA_SS0
const FLEXSPI_INSTANCE: u32 = 0;
const MEMORY_MAP_FLEXSPI_START_ADDRESS: u32 = 0x60000000;
const PAGE_SIZE: u32 = 256;
const SECTOR_SIZE: u32 = 65536;

struct Algorithm {}

algorithm!(Algorithm, {
    device_name: "imxrt-flash-algorithm",
    device_type: DeviceType::Onchip,
    flash_address: MEMORY_MAP_FLEXSPI_START_ADDRESS,
    flash_size: 0x00800000,
    page_size: PAGE_SIZE,
    empty_value: 0xFF,
    program_time_out: 2000,
    erase_time_out: 6000,
    sectors: [{
        size: SECTOR_SIZE,
        address: 0x0,
    }]
});

static mut NOR_CONFIG: MaybeUninit<flexspi_nor_config_t> = MaybeUninit::uninit();

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        unsafe {
            chip::init();

            #[cfg(feature = "log")]
            log::init();

            dprintln!("Initializing flash algorithm...");

            // Flash options
            let mut option: serial_nor_config_option_t = core::mem::zeroed();

            // Set required tag
            option.option0.B.set_tag(0x0C);

            // from serialClkFreq docs under "Table 9-15. FlexSPI Configuration block"
            // 1 – 30 MHz
            // 2 – 50 MHz
            // 3 – 60 MHz
            // 4 – 75 MHz
            // 5 – 80 MHz
            // 6 – 100 MHz
            // 7 – 120 MHz
            // 8 – 133 MHz
            // 9 - 166 MHz
            // Other value: 30 MH

            // QuadSPI NOR - Quad SDR Read (100MHz) (what the NXP flashloader uses)
            option.option0.B.set_max_freq(6);

            // Get NOR configuration block
            let status =
                rom_api::flexspi_nor_get_config(FLEXSPI_INSTANCE, NOR_CONFIG.as_mut_ptr(), &option);
            if status != 0 {
                return Err(ErrorCode::new_unchecked(status as _));
            }

            // By default, the ROM API will always use a 30 MHz clock for programming.
            // If we trust the Flash to keep up, we could change the frequency here:
            // (*NOR_CONFIG.as_mut_ptr()).ipcmdSerialClkFreq = 6;

            // initialize flash
            let status = rom_api::flexspi_nor_flash_init(FLEXSPI_INSTANCE, NOR_CONFIG.as_ptr());
            if status != 0 {
                return Err(ErrorCode::new_unchecked(status as _));
            }
            Ok(Self {})
        }
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        // dprintln!("Erase All");

        let status =
            unsafe { rom_api::flexspi_nor_flash_erase_all(FLEXSPI_INSTANCE, NOR_CONFIG.as_ptr()) };
        if status != 0 {
            return Err(unsafe { ErrorCode::new_unchecked(status as _) });
        }

        Ok(())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        // dprintln!("Erase sector addr:{}", addr);

        let flash_addr: u32 = addr - MEMORY_MAP_FLEXSPI_START_ADDRESS;
        let status = unsafe {
            rom_api::flexspi_nor_flash_erase(
                FLEXSPI_INSTANCE,
                NOR_CONFIG.as_ptr(),
                flash_addr,
                SECTOR_SIZE,
            )
        };

        if status != 0 {
            return Err(unsafe { ErrorCode::new_unchecked(status as _) });
        }

        Ok(())
    }

    #[cfg(feature = "miniz")]
    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        // dprintln!("Program Page addr:{} size:{}", addr, data.len());
        unsafe { inflate::decompressor() }.program(addr, data)?;
        Ok(())
    }

    #[cfg(not(feature = "miniz"))]
    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        // dprintln!("Program Page addr:{} size:{}", addr, data.len());
        let flash_addr: u32 = addr - crate::MEMORY_MAP_FLEXSPI_START_ADDRESS;
        let status = unsafe {
            crate::rom_api::flexspi_nor_flash_page_program(
                crate::FLEXSPI_INSTANCE,
                crate::NOR_CONFIG.as_ptr(),
                flash_addr,
                data.as_ptr() as *const u32,
            )
        };
        if status != 0 {
            return Err(unsafe { ErrorCode::new_unchecked(status as _) });
        }
        Ok(())
    }
}

impl Drop for Algorithm {
    #[cfg(feature = "miniz")]
    fn drop(&mut self) {
        // TODO: don't swallow possible error
        let _ = unsafe { inflate::decompressor() }.flush();
    }

    #[cfg(not(feature = "miniz"))]
    fn drop(&mut self) {
        // no-op
    }
}
