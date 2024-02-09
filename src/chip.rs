use imxrt_ral as ral;

/// One-time chip setup.
pub unsafe fn init() {
    static mut INIT: bool = false;
    if !INIT {
        disable_watchdog();
        disable_systick();
        clock_setup();
        INIT = true;
    }
}

unsafe fn disable_systick() {
    // get ARM Cortex peripherals access (without taking/stealing them)
    let mut periphs: cortex_m::Peripherals = core::mem::transmute(());
    // (compile time sanity check)
    const _: [u8; 0] = [0; core::mem::size_of::<cortex_m::Peripherals>()];
    // Disable SysTick counter.
    periphs.SYST.disable_counter();
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
