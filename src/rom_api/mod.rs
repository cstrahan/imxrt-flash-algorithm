mod bindings;
pub use bindings::*;

/// The root of the ROM API.
const BOOTLOADER_TREE_PTR: *const *const bootloader_api_entry_t =
    0x0020001c as *const *const bootloader_api_entry_t;

// Sanity check size of flexspi_nor_config_t, using size from NXP's flash algo's DWARF data.
const _: [u8; 512] = [0; core::mem::size_of::<flexspi_nor_config_t>()];

// Sanity check size of bootloader_api_entry_t.
const _: [u8; 40] = [0; core::mem::size_of::<bootloader_api_entry_t>()];

/// Initialize the Serial NOR device via FLEXSPI.
pub unsafe fn flexspi_nor_flash_init(
    instance: u32,
    config: *const flexspi_nor_config_t,
) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .init
        .unwrap_unchecked()(instance, config as _)
}

/// Program data to specified Flash address.
pub unsafe fn flexspi_nor_flash_page_program(
    instance: u32,
    config: *const flexspi_nor_config_t,
    dst_addr: u32,
    src: *const u32,
) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .program
        .unwrap_unchecked()(instance, config as _, dst_addr, src)
}

/// Erase the whole Flash array via FLEXSPI.
pub unsafe fn flexspi_nor_flash_erase_all(
    instance: u32,
    config: *const flexspi_nor_config_t,
) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .erase_all
        .unwrap_unchecked()(instance, config as _)
}

/// Get the Flash configuration block via the serial_nor_config_option_t block.
pub unsafe fn flexspi_nor_get_config(
    instance: u32,
    config: *mut flexspi_nor_config_t,
    option: *const serial_nor_config_option_t,
) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .get_config
        .unwrap_unchecked()(instance, config as _, option as _)
}

/// Erase specified Flash region, the minimum erase unit is one sector.
pub unsafe fn flexspi_nor_flash_erase(
    instance: u32,
    config: *const flexspi_nor_config_t,
    start: u32,
    length: u32,
) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .erase
        .unwrap_unchecked()(instance, config as _, start, length)
}

/// Read the FLASH via FLEXSPI using IP read command.
pub unsafe fn flexspi_nor_flash_read(
    instance: u32,
    config: *const flexspi_nor_config_t,
    dst: *mut u32,
    start: u32,
    bytes: u32,
) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .read
        .unwrap_unchecked()(instance, config as _, dst, start, bytes)
}

/// Update the specified LUT entries.
pub unsafe fn flexspi_update_lut(
    instance: u32,
    seq_index: u32,
    lut_base: *const u32,
    number_of_seq: u32,
) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .update_lut
        .unwrap_unchecked()(instance, seq_index, lut_base, number_of_seq)
}

/// Execute LUT sequence specified by xfer.
pub unsafe fn flexspi_command_xfer(instance: u32, xfer: *mut flexspi_xfer_t) -> spi_status_t {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .xfer
        .unwrap_unchecked()(instance, xfer)
}

/// Clear the AHB buffer in FLEXSPI module.
pub unsafe fn flexspi_clear_cache(instance: u32) {
    (*(*(*BOOTLOADER_TREE_PTR)).flexSpiNorDriver)
        .clear_cache
        .unwrap_unchecked()(instance)
}
