#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct __BindgenBitfieldUnit<Storage> {
    storage: Storage,
}
impl<Storage> __BindgenBitfieldUnit<Storage> {
    #[inline]
    pub const fn new(storage: Storage) -> Self {
        Self { storage }
    }
}
impl<Storage> __BindgenBitfieldUnit<Storage>
where
    Storage: AsRef<[u8]> + AsMut<[u8]>,
{
    #[inline]
    pub fn get_bit(&self, index: usize) -> bool {
        debug_assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = self.storage.as_ref()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        byte & mask == mask
    }
    #[inline]
    pub fn set_bit(&mut self, index: usize, val: bool) {
        debug_assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = &mut self.storage.as_mut()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        if val {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }
    #[inline]
    pub fn get(&self, bit_offset: usize, bit_width: u8) -> u64 {
        debug_assert!(bit_width <= 64);
        debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
        debug_assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        let mut val = 0;
        for i in 0..(bit_width as usize) {
            if self.get_bit(i + bit_offset) {
                let index = if cfg!(target_endian = "big") {
                    bit_width as usize - 1 - i
                } else {
                    i
                };
                val |= 1 << index;
            }
        }
        val
    }
    #[inline]
    pub fn set(&mut self, bit_offset: usize, bit_width: u8, val: u64) {
        debug_assert!(bit_width <= 64);
        debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
        debug_assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        for i in 0..(bit_width as usize) {
            let mask = 1 << i;
            let val_bit_is_set = val & mask == mask;
            let index = if cfg!(target_endian = "big") {
                bit_width as usize - 1 - i
            } else {
                i
            };
            self.set_bit(index + bit_offset, val_bit_is_set);
        }
    }
}

/// FlexSPI LUT Sequence structure
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _lut_sequence {
    /// Sequence Number, valid number: 1-16
    pub seqNum: u8,
    /// Sequence Index, valid number: 0-15
    pub seqId: u8,
    pub reserved: u16,
}

/// FlexSPI LUT Sequence structure
pub type flexspi_lut_seq_t = _lut_sequence;

/// FlexSPI Memory Configuration Block
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _FlexSPIConfig {
    /// Tag, fixed value 0x42464346UL
    pub tag: u32,
    /// Version,[31:24] -'V', [23:16] - Major, [15:8] - Minor, [7:0] - bugfix
    pub version: u32,
    /// Reserved for future use
    pub reserved0: u32,
    /// Read Sample Clock Source, valid value: 0/1/3
    pub readSampleClkSrc: u8,
    /// CS hold time, default value: 3
    pub csHoldTime: u8,
    /// CS setup time, default value: 3
    pub csSetupTime: u8,
    /// Column Address with, for HyperBus protocol, it is fixed to 3, For
    pub columnAddressWidth: u8,
    /// Device Mode Configure enable flag, 1 - Enable, 0 - Disable
    pub deviceModeCfgEnable: u8,
    /// Specify the configuration command type:Quad Enable, DPI/QPI/OPI switch,
    pub deviceModeType: u8,
    /// Wait time for all configuration commands, unit: 100us, Used for
    pub waitTimeCfgCommands: u16,
    /// Device mode sequence info, [7:0] - LUT sequence id, [15:8] - LUt
    pub deviceModeSeq: flexspi_lut_seq_t,
    /// Argument/Parameter for device configuration
    pub deviceModeArg: u32,
    /// Configure command Enable Flag, 1 - Enable, 0 - Disable
    pub configCmdEnable: u8,
    /// Configure Mode Type, similar as deviceModeTpe
    pub configModeType: [u8; 3usize],
    /// Sequence info for Device Configuration command, similar as deviceModeSeq
    pub configCmdSeqs: [flexspi_lut_seq_t; 3usize],
    /// Reserved for future use
    pub reserved1: u32,
    /// Arguments/Parameters for device Configuration commands
    pub configCmdArgs: [u32; 3usize],
    /// Reserved for future use
    pub reserved2: u32,
    /// Controller Misc Options, see Misc feature bit definitions for more
    pub controllerMiscOption: u32,
    /// Device Type:  See Flash Type Definition for more details
    pub deviceType: u8,
    /// Serial Flash Pad Type: 1 - Single, 2 - Dual, 4 - Quad, 8 - Octal
    pub sflashPadType: u8,
    /// Serial Flash Frequencey, device specific definitions, See System Boot
    pub serialClkFreq: u8,
    /// LUT customization Enable, it is required if the program/erase cannot
    pub lutCustomSeqEnable: u8,
    /// Reserved for future use
    pub reserved3: [u32; 2usize],
    /// Size of Flash connected to A1
    pub sflashA1Size: u32,
    /// Size of Flash connected to A2
    pub sflashA2Size: u32,
    /// Size of Flash connected to B1
    pub sflashB1Size: u32,
    /// Size of Flash connected to B2
    pub sflashB2Size: u32,
    /// CS pad setting override value
    pub csPadSettingOverride: u32,
    /// SCK pad setting override value
    pub sclkPadSettingOverride: u32,
    /// data pad setting override value
    pub dataPadSettingOverride: u32,
    /// DQS pad setting override value
    pub dqsPadSettingOverride: u32,
    /// Timeout threshold for read status command
    pub timeoutInMs: u32,
    /// CS deselect interval between two commands
    pub commandInterval: u32,
    /// CLK edge to data valid time for PORT A and PORT B, in terms of 0.1ns
    pub dataValidTime: [u16; 2usize],
    /// Busy offset, valid value: 0-31
    pub busyOffset: u16,
    /// Busy flag polarity, 0 - busy flag is 1 when flash device is busy, 1 -
    pub busyBitPolarity: u16,
    /// Lookup table holds Flash command sequences
    pub lookupTable: [u32; 64usize],
    /// Customizable LUT Sequences
    pub lutCustomSeq: [flexspi_lut_seq_t; 12usize],
    /// Reserved for future use
    pub reserved4: [u32; 4usize],
}
/// FlexSPI Memory Configuration Block
pub type flexspi_mem_config_t = _FlexSPIConfig;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _flexspi_nor_config {
    /// Common memory configuration info via FlexSPI
    pub memConfig: flexspi_mem_config_t,
    /// Page size of Serial NOR
    pub pageSize: u32,
    /// Sector size of Serial NOR
    pub sectorSize: u32,
    /// Clock frequency for IP command
    pub ipcmdSerialClkFreq: u8,
    /// Sector/Block size is the same
    pub isUniformBlockSize: u8,
    /// Reserved for future use
    pub reserved0: [u8; 2usize],
    /// Serial NOR Flash type: 0/1/2/3
    pub serialNorType: u8,
    /// Need to exit NoCmd mode before other IP command
    pub needExitNoCmdMode: u8,
    /// Half the Serial Clock for non-read command: true/false
    pub halfClkForNonReadCmd: u8,
    /// Need to Restore NoCmd mode after IP commmand execution
    pub needRestoreNoCmdMode: u8,
    /// Block size
    pub blockSize: u32,
    /// Reserved for future use
    pub reserve2: [u32; 11usize],
}
pub type flexspi_nor_config_t = _flexspi_nor_config;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _serial_nor_config_option {
    pub option0: _serial_nor_config_option__bindgen_ty_1,
    pub option1: _serial_nor_config_option__bindgen_ty_2,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union _serial_nor_config_option__bindgen_ty_1 {
    pub B: _serial_nor_config_option__bindgen_ty_1__bindgen_ty_1,
    pub U: u32,
}
#[repr(C)]
#[repr(align(4))]
#[derive(Debug, Copy, Clone)]
pub struct _serial_nor_config_option__bindgen_ty_1__bindgen_ty_1 {
    pub _bitfield_align_1: [u8; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl _serial_nor_config_option__bindgen_ty_1__bindgen_ty_1 {
    #[inline]
    pub fn max_freq(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(0usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_max_freq(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn misc_mode(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(4usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_misc_mode(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(4usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn quad_mode_setting(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(8usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_quad_mode_setting(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(8usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn cmd_pads(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(12usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_cmd_pads(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(12usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn query_pads(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(16usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_query_pads(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(16usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn device_type(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(20usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_device_type(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(20usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn option_size(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(24usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_option_size(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(24usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn tag(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(28usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_tag(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(28usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn new_bitfield_1(
        max_freq: u32,
        misc_mode: u32,
        quad_mode_setting: u32,
        cmd_pads: u32,
        query_pads: u32,
        device_type: u32,
        option_size: u32,
        tag: u32,
    ) -> __BindgenBitfieldUnit<[u8; 4usize]> {
        let mut __bindgen_bitfield_unit: __BindgenBitfieldUnit<[u8; 4usize]> = Default::default();
        __bindgen_bitfield_unit.set(0usize, 4u8, {
            let max_freq: u32 = unsafe { ::core::mem::transmute(max_freq) };
            max_freq as u64
        });
        __bindgen_bitfield_unit.set(4usize, 4u8, {
            let misc_mode: u32 = unsafe { ::core::mem::transmute(misc_mode) };
            misc_mode as u64
        });
        __bindgen_bitfield_unit.set(8usize, 4u8, {
            let quad_mode_setting: u32 = unsafe { ::core::mem::transmute(quad_mode_setting) };
            quad_mode_setting as u64
        });
        __bindgen_bitfield_unit.set(12usize, 4u8, {
            let cmd_pads: u32 = unsafe { ::core::mem::transmute(cmd_pads) };
            cmd_pads as u64
        });
        __bindgen_bitfield_unit.set(16usize, 4u8, {
            let query_pads: u32 = unsafe { ::core::mem::transmute(query_pads) };
            query_pads as u64
        });
        __bindgen_bitfield_unit.set(20usize, 4u8, {
            let device_type: u32 = unsafe { ::core::mem::transmute(device_type) };
            device_type as u64
        });
        __bindgen_bitfield_unit.set(24usize, 4u8, {
            let option_size: u32 = unsafe { ::core::mem::transmute(option_size) };
            option_size as u64
        });
        __bindgen_bitfield_unit.set(28usize, 4u8, {
            let tag: u32 = unsafe { ::core::mem::transmute(tag) };
            tag as u64
        });
        __bindgen_bitfield_unit
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union _serial_nor_config_option__bindgen_ty_2 {
    pub B: _serial_nor_config_option__bindgen_ty_2__bindgen_ty_1,
    pub U: u32,
}
#[repr(C)]
#[repr(align(4))]
#[derive(Debug, Copy, Clone)]
pub struct _serial_nor_config_option__bindgen_ty_2__bindgen_ty_1 {
    pub _bitfield_align_1: [u8; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 4usize]>,
}
impl _serial_nor_config_option__bindgen_ty_2__bindgen_ty_1 {
    #[inline]
    pub fn dummy_cycles(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(0usize, 8u8) as u32) }
    }
    #[inline]
    pub fn set_dummy_cycles(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(0usize, 8u8, val as u64)
        }
    }
    #[inline]
    pub fn reserved0(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(8usize, 8u8) as u32) }
    }
    #[inline]
    pub fn set_reserved0(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(8usize, 8u8, val as u64)
        }
    }
    #[inline]
    pub fn pinmux_group(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(16usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_pinmux_group(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(16usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn reserved1(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(20usize, 8u8) as u32) }
    }
    #[inline]
    pub fn set_reserved1(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(20usize, 8u8, val as u64)
        }
    }
    #[inline]
    pub fn flash_connection(&self) -> u32 {
        unsafe { ::core::mem::transmute(self._bitfield_1.get(28usize, 4u8) as u32) }
    }
    #[inline]
    pub fn set_flash_connection(&mut self, val: u32) {
        unsafe {
            let val: u32 = ::core::mem::transmute(val);
            self._bitfield_1.set(28usize, 4u8, val as u64)
        }
    }
    #[inline]
    pub fn new_bitfield_1(
        dummy_cycles: u32,
        reserved0: u32,
        pinmux_group: u32,
        reserved1: u32,
        flash_connection: u32,
    ) -> __BindgenBitfieldUnit<[u8; 4usize]> {
        let mut __bindgen_bitfield_unit: __BindgenBitfieldUnit<[u8; 4usize]> = Default::default();
        __bindgen_bitfield_unit.set(0usize, 8u8, {
            let dummy_cycles: u32 = unsafe { ::core::mem::transmute(dummy_cycles) };
            dummy_cycles as u64
        });
        __bindgen_bitfield_unit.set(8usize, 8u8, {
            let reserved0: u32 = unsafe { ::core::mem::transmute(reserved0) };
            reserved0 as u64
        });
        __bindgen_bitfield_unit.set(16usize, 4u8, {
            let pinmux_group: u32 = unsafe { ::core::mem::transmute(pinmux_group) };
            pinmux_group as u64
        });
        __bindgen_bitfield_unit.set(20usize, 8u8, {
            let reserved1: u32 = unsafe { ::core::mem::transmute(reserved1) };
            reserved1 as u64
        });
        __bindgen_bitfield_unit.set(28usize, 4u8, {
            let flash_connection: u32 = unsafe { ::core::mem::transmute(flash_connection) };
            flash_connection as u64
        });
        __bindgen_bitfield_unit
    }
}

pub type serial_nor_config_option_t = _serial_nor_config_option;
/// FlexSPI operation: Only command, both TX and RX buffer are ignored.
pub const _FlexSPIOperationType_kFlexSpiOperation_Command: _FlexSPIOperationType = 0;
//// FlexSPI operation: Configure device mode, the TX FIFO size is fixed in LUT.
pub const _FlexSPIOperationType_kFlexSpiOperation_Config: _FlexSPIOperationType = 1;
/// FlexSPI operation: Write, only TX buffer is effective.
pub const _FlexSPIOperationType_kFlexSpiOperation_Write: _FlexSPIOperationType = 2;
/// FlexSPI operation: Read, only Rx Buffer is effective.
pub const _FlexSPIOperationType_kFlexSpiOperation_Read: _FlexSPIOperationType = 3;
pub type _FlexSPIOperationType = ::core::ffi::c_uint;
pub use self::_FlexSPIOperationType as flexspi_operation_t;

/// FlexSPI Transfer Context
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _FlexSpiXfer {
    pub operation: flexspi_operation_t,
    pub baseAddress: u32,
    pub seqId: u32,
    pub seqNum: u32,
    pub isParallelModeEnable: bool,
    pub txBuffer: *mut u32,
    pub txSize: u32,
    pub rxBuffer: *mut u32,
    pub rxSize: u32,
}

/// FlexSPI Transfer Context
pub type flexspi_xfer_t = _FlexSpiXfer;
pub const spi_status_kSPI_Status_Success: spi_status = 0;
pub const spi_status_kSPI_Status_Fail: spi_status = 1;
pub const spi_status_kSPI_Status_InvalidArgument: spi_status = 4;
pub const spi_status_kSPI_Status_Timeout: spi_status = 5;
pub const spi_status_kSPI_Status_FLEXSPI_SequenceExecutionTimeout: spi_status = 7000;
pub const spi_status_kSPI_Status_FLEXSPI_InvalidSequence: spi_status = 7001;
pub const spi_status_kSPI_Status_FLEXSPI_DeviceTimeout: spi_status = 7002;
pub const spi_status_kSPI_Status_FLEXSPINOR_ProgramFail: spi_status = 20100;
pub const spi_status_kSPI_Status_FLEXSPINOR_EraseSectorFail: spi_status = 20101;
pub const spi_status_kSPI_Status_FLEXSPINOR_EraseAllFail: spi_status = 20102;
pub const spi_status_kSPI_Status_FLEXSPINOR_WaitTimeout: spi_status = 20103;
pub const spi_status_kSPI_Status_FlexSPINOR_NotSupported: spi_status = 20104;
pub const spi_status_kSPI_Status_FlexSPINOR_WriteAlignmentError: spi_status = 20105;
pub const spi_status_kSPI_Status_FlexSPINOR_CommandFailure: spi_status = 20106;
pub const spi_status_kSPI_Status_FlexSPINOR_SFDP_NotFound: spi_status = 20107;
pub const spi_status_kSPI_Status_FLEXSPINOR_Flash_NotFound: spi_status = 20109;
pub const spi_status_kSPI_Status_FLEXSPINOR_DTRRead_DummyProbeFailed: spi_status = 20110;
pub type spi_status = u32;
pub use self::spi_status as spi_status_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flexspi_nor_driver_interface_t {
    pub version: u32,
    pub init: ::core::option::Option<
        unsafe extern "C" fn(instance: u32, config: *mut flexspi_nor_config_t) -> spi_status_t,
    >,
    pub program: ::core::option::Option<
        unsafe extern "C" fn(
            instance: u32,
            config: *mut flexspi_nor_config_t,
            dst_addr: u32,
            src: *const u32,
        ) -> spi_status_t,
    >,
    pub erase_all: ::core::option::Option<
        unsafe extern "C" fn(instance: u32, config: *mut flexspi_nor_config_t) -> spi_status_t,
    >,
    pub erase: ::core::option::Option<
        unsafe extern "C" fn(
            instance: u32,
            config: *mut flexspi_nor_config_t,
            start: u32,
            lengthInBytes: u32,
        ) -> spi_status_t,
    >,
    pub read: ::core::option::Option<
        unsafe extern "C" fn(
            instance: u32,
            config: *mut flexspi_nor_config_t,
            dst: *mut u32,
            addr: u32,
            lengthInBytes: u32,
        ) -> spi_status_t,
    >,
    pub clear_cache: ::core::option::Option<unsafe extern "C" fn(instance: u32)>,
    pub xfer: ::core::option::Option<
        unsafe extern "C" fn(instance: u32, xfer: *mut flexspi_xfer_t) -> spi_status_t,
    >,
    pub update_lut: ::core::option::Option<
        unsafe extern "C" fn(
            instance: u32,
            seqIndex: u32,
            lutBase: *const u32,
            seqNumber: u32,
        ) -> spi_status_t,
    >,
    pub get_config: ::core::option::Option<
        unsafe extern "C" fn(
            instance: u32,
            config: *mut flexspi_nor_config_t,
            option: *mut serial_nor_config_option_t,
        ) -> spi_status_t,
    >,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bootloader_api_entry_t {
    /// Bootloader version number
    pub version: u32,
    /// Bootloader Copyright
    pub copyright: *const ::core::ffi::c_char,
    /// Function to start the bootloader executing
    pub runBootloader: ::core::option::Option<unsafe extern "C" fn(arg: *mut ::core::ffi::c_void)>,
    pub reserved0: *const u32,
    /// FlexSPI NOR Flash API
    pub flexSpiNorDriver: *const flexspi_nor_driver_interface_t,
    pub reserved1: *const u32,
    pub reserved2: *const u32,
    pub reserved3: *const u32,
    pub reserved4: *const u32,
    pub reserved5: *const u32,
}
