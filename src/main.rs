#![no_std]
#![no_main]

use flash_algorithm::*;

// use rtt_target::{rprintln, rtt_init_print};

use rom_api::{flexspi_nor_config_t, serial_nor_config_option_t};

use imxrt_ral as ral;

pub mod rom_api;

/// The ROM APIs only support 1 single FLASH device connected to PORTA and FLEXSPIA_SS0
const FLEXSPI_INSTANCE: u32 = 0;
const MEMORY_MAP_FLEXSPI_START_ADDRESS: u32 = 0x60000000;
const PAGE_SIZE: u32 = 256;
const SECTOR_SIZE: u32 = 65536;

struct Algorithm {
    nor_config: flexspi_nor_config_t,
}

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

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        // rtt_init_print!();
        // rprintln!("Init");

        unsafe {
            let mut this = Self {
                nor_config: core::mem::zeroed(),
            };

            disable_watchdog();

            // get ARM Cortex peripherals access (without taking/stealing them)
            let mut periphs: cortex_m::Peripherals = core::mem::transmute(());
            // (compile time sanity check)
            const _: [u8; 0] = [0; core::mem::size_of::<cortex_m::Peripherals>()];

            // Disable SysTick counter.
            periphs.SYST.disable_counter();

            clock_setup();

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
                rom_api::flexspi_nor_get_config(FLEXSPI_INSTANCE, &mut this.nor_config, &option);
            if status != 0 {
                return Err(ErrorCode::new_unchecked(status as _));
            }

            // rprintln!("Sector size:{}", nor_config.sectorSize);

            // By default, the ROM API will always use a 30 MHz clock for programming.
            // If we trust the Flash to keep up, we could change the frequency here:
            // nor_config.ipcmdSerialClkFreq = 8;

            // initialize flash
            let status = rom_api::flexspi_nor_flash_init(FLEXSPI_INSTANCE, &this.nor_config);
            if status != 0 {
                return Err(ErrorCode::new_unchecked(status as _));
            }
            Ok(this)
        }
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        // rprintln!("Erase All");

        let status =
            unsafe { rom_api::flexspi_nor_flash_erase_all(FLEXSPI_INSTANCE, &self.nor_config) };
        if status != 0 {
            return Err(unsafe { ErrorCode::new_unchecked(status as _) });
        }

        Ok(())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        // rprintln!("Erase sector addr:{}", addr);

        let flash_addr: u32 = addr - MEMORY_MAP_FLEXSPI_START_ADDRESS;
        let status = unsafe {
            rom_api::flexspi_nor_flash_erase(
                FLEXSPI_INSTANCE,
                &self.nor_config,
                flash_addr,
                SECTOR_SIZE,
            )
        };

        if status != 0 {
            return Err(unsafe { ErrorCode::new_unchecked(status as _) });
        }

        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        // rprintln!("Program Page addr:{} size:{}", addr, data.len());

        let flash_addr: u32 = addr - MEMORY_MAP_FLEXSPI_START_ADDRESS;
        let status = unsafe {
            rom_api::flexspi_nor_flash_page_program(
                FLEXSPI_INSTANCE,
                &self.nor_config,
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
    fn drop(&mut self) {
        // no-op
    }
}

unsafe fn disable_watchdog() {
    // Disable Power Down Counter of WDOG1.
    ral::modify_reg!(ral::wdog, WDOG1, WMCR, PDE: PDE_0);

    // Disable Power Down Counter of WDOG2.
    ral::modify_reg!(ral::wdog, WDOG2, WMCR, PDE: PDE_0);

    // If WDOG1 enabled (WDE set):
    // - Disable the Watchdog (WDE cleared)
    if (ral::read_reg!(ral::wdog, WDOG1, WCR, WDE) != 0) {
        ral::modify_reg!(ral::wdog, WDOG1, WCR,
            WDE: WDE_0,
        );
    }

    // If WDOG2 enabled (WDE set):
    // - Disable the Watchdog (WDE cleared)
    if (ral::read_reg!(ral::wdog, WDOG2, WCR, WDE) != 0) {
        ral::modify_reg!(ral::wdog, WDOG2, WCR,
            WDE: WDE_0,
        );
    }

    // Unlock RTWDOG
    // (58.3.2.2.1 Unlocking the Watchdog)
    if (ral::read_reg!(ral::rtwdog, RTWDOG, CS, CMD32EN) != 0) {
        ral::write_reg!(ral::rtwdog, RTWDOG, CNT, 0xd928c520);
    } else {
        ral::write_reg!(ral::rtwdog, RTWDOG, CNT, 0xdc520);
        ral::write_reg!(ral::rtwdog, RTWDOG, CNT, 0xdc520);
    }

    // Set Watchdog Timeout Value
    ral::write_reg!(ral::rtwdog, RTWDOG, TOVAL, 0xffff);

    // Disable RTWDOG (EN cleared),
    // and allow reconfiguring later (UPDATE set).
    ral::modify_reg!(
        ral::rtwdog,
        RTWDOG,
        CS,
        EN: EN_0,
        UPDATE: UPDATE_1
    );
}

/// LPB_BOOT: (Core / Bus)
///
/// Low-Power Boot
/// - '00' - Div by 1
/// - '01' - Div by 2
/// - '10' - Div by 4
/// - '11' - Div by 8
fn rom_ocotp_lpb_boot_value() -> u32 {
    const ROM_OCOTP_LPB_BOOT_MASK: u32 = 0x00600000;
    const ROM_OCOTP_LPB_BOOT_SHIFT: u32 = 21;

    unsafe {
        (ral::read_reg!(ral::ocotp, OCOTP, CFG6) & ROM_OCOTP_LPB_BOOT_MASK)
            >> ROM_OCOTP_LPB_BOOT_SHIFT
    }
}

/// BOOT_FREQ
///
/// Determines, ARM Core and Bus frequencies during boot
/// - 0 - (ARM) 396 / (Bus) 132 MHz
/// - 1 - (ARM) 528 / (Bus) 132 MHz
fn rom_ocotp_boot_freq_value() -> u32 {
    const ROM_OCOTP_BOOT_FREQ_MASK: u32 = 0x04;
    const ROM_OCOTP_BOOT_FREQ_SHIFT: u32 = 0x02;

    unsafe {
        (ral::read_reg!(ral::ocotp, OCOTP, CFG5) & ROM_OCOTP_BOOT_FREQ_MASK)
            >> ROM_OCOTP_BOOT_FREQ_SHIFT
    }
}

unsafe fn clock_setup() {
    let fuse_div = rom_ocotp_lpb_boot_value();
    let clock_freq = rom_ocotp_boot_freq_value();
    let clock_divider = 1 << fuse_div;

    // Bypass clock configurations if clock is configured
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_ARM,
        BYPASS: 1,
    );
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_SYS,
        BYPASS: 1,
    );
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_USB1,
        BYPASS: 1,
    );

    // Configure PLL_ARM: Reference clock = 24MHz
    // PLL_ARM = 24MHz * div / 2
    //    Core = PLL_ARM / 2 / clock_divider
    // To get 396MHz clock, PLL_ARM = 24 * 66 / 2 = 792, 792 / 2 = 396
    // To get 528MHz clock, PLL_ARM = 24 * 88 / 2 = 1056, 1056 / 2 = 528
    let div = if clock_freq == 1 { 88 } else { 66 };
    ral::write_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_ARM,
        BYPASS: 1,
        ENABLE: 1,
        DIV_SELECT: div,
    );

    // Wait Until clock is locked
    while ral::read_reg!(ral::ccm_analog, CCM_ANALOG, PLL_ARM, LOCK) == 0 {
        // wait
    }

    // Configure PLL_SYS
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_SYS,
        POWERDOWN: 0,
    );

    // Wait Until clock is locked
    while ral::read_reg!(ral::ccm_analog, CCM_ANALOG, PLL_SYS, LOCK) == 0 {
        // wait...
    }

    // Configure SYS_PLL PFD
    // PFD0 = 396MHz  - uSDHC CLOCK Source
    // PFD1 = 396MHz
    // PFD2 = 500MHz  - SEMC CLOCK Source
    // PFD3 = 396MHz
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PFD_528,
        PFD0_FRAC: 24,
        PFD1_FRAC: 24,
        PFD2_FRAC: 19,
        PFD3_FRAC: 24,
    );

    // Always configure USB1_PLL
    ral::write_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_USB1,
        DIV_SELECT: 0,
        POWER: 1,
        ENABLE: 1,
    );

    // Wait Until clock is locked
    while ral::read_reg!(ral::ccm_analog, CCM_ANALOG, PLL_USB1, LOCK) == 0 {
        // wait...
    }

    // Configure USB_PLL PFD
    // PFD0 = 247MHz  - FLEXSPI CLOCK Source
    // PFD1 = 247MHz  - LPSPI CLOCK Source
    // PFD2 = 332MHz
    // PFD3 = 576MHz
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PFD_480,
        PFD0_FRAC: 35,
        PFD1_FRAC: 35,
        PFD2_FRAC: 26,
        PFD3_FRAC: 15,
    );

    // Set up CPU_PODF
    ral::write_reg!(ral::ccm, CCM, CACRR, ARM_PODF: ARM_PODF_1);

    const FREQ_396MHZ: u32 = 396_000_000;
    const FREQ_528MHZ: u32 = 528_000_000;
    const MAX_AHB_CLOCK: u32 = 144_000_000;

    // Calculate the Final System Core Clock, it will be used to calculate the AHB / ARM Core divider later.
    let system_core_clock = if clock_freq == 0 {
        FREQ_396MHZ
    } else {
        FREQ_528MHZ
    } / clock_divider;

    // Calculate the AHB clock divider
    let mut ahb_divider = 1;
    while (system_core_clock / ahb_divider) > MAX_AHB_CLOCK {
        ahb_divider += 1;
    }

    // Set up AXI_PODF - SEMC clock root
    // Set up AHB_PODF - CORE clock
    // Set up IPG_PODF - BUS clock
    ral::modify_reg!(ral::ccm, CCM, CBCDR,
        SEMC_PODF: ahb_divider - 1,
        AHB_PODF: clock_divider - 1,
        IPG_PODF: ahb_divider - 1,
    );

    // LPUART clock configuration, peripheral clock 20MHz
    ral::modify_reg!(ral::ccm, CCM, CSCDR1,
        UART_CLK_SEL: UART_CLK_SEL_0,
        UART_CLK_PODF: DIVIDE_4,
    );

    // Pre-peripheral clock configuration
    ral::modify_reg!(ral::ccm, CCM, CBCMR,
        PRE_PERIPH_CLK_SEL: PRE_PERIPH_CLK_SEL_3,
    );

    // LPSPI clock configuration, Peripheral clock: 41MHz
    ral::modify_reg!(ral::ccm, CCM, CBCMR,
        LPSPI_CLK_SEL: LPSPI_CLK_SEL_0,
        LPSPI_PODF: LPSPI_PODF_5,
    );

    // FLEXSPI clock configuration, safe frequency: 30MHz
    ral::modify_reg!(ral::ccm, CCM, CSCMR1,
        FLEXSPI_CLK_SEL: FLEXSPI_CLK_SEL_3,
        FLEXSPI_PODF: FLEXSPI_PODF_7,
        PERCLK_CLK_SEL: PERCLK_CLK_SEL_0,
        PERCLK_PODF: DIVIDE_2,
    );

    // NOTE: SEMC clock configuration needs handshake, so it will be handled by SEMC driver itself
    // uSDHC1&2 clock configuration
    // SEL: PULLL2 PFD0; DIV: 1 (PFD/2, freq=200MHz)
    ral::modify_reg!(ral::ccm, CCM, CSCMR1,
        USDHC1_CLK_SEL: USDHC1_CLK_SEL_1,
        USDHC2_CLK_SEL: USDHC2_CLK_SEL_1,
    );
    ral::modify_reg!(ral::ccm, CCM, CSCDR1,
        USDHC1_PODF: USDHC1_PODF_1,
        USDHC2_PODF: USDHC2_PODF_1,
    );

    // Finally, Enable PLL_ARM, PLL_SYS and PLL_USB1
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_ARM,
        BYPASS: 0,
    );
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_SYS,
        BYPASS: 0,
    );
    ral::modify_reg!(
        ral::ccm_analog,
        CCM_ANALOG,
        PLL_USB1,
        BYPASS: 0,
    );
}
