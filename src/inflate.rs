use core::mem::MaybeUninit;

use flash_algorithm::ErrorCode;
use miniz_oxide::inflate::core::{decompress, inflate_flags, DecompressorOxide};
use miniz_oxide::inflate::TINFLStatus;

static mut DECOMPRESSOR: Option<Decompressor> = None;

pub unsafe fn decompressor() -> &'static mut Decompressor {
    if DECOMPRESSOR.is_none() {
        DECOMPRESSOR = Some(Decompressor::new());
    }

    DECOMPRESSOR.as_mut().unwrap_unchecked()
}

struct OutBuffer {
    buffer: MaybeUninit<[u8; 32768]>,
    len: usize,
}

impl OutBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: MaybeUninit::uninit(),
            len: 0,
        }
    }

    fn space(&self) -> usize {
        unsafe { self.buffer.assume_init() }.len() - self.len
    }

    fn buffer(&mut self) -> &mut [u8] {
        unsafe { self.buffer.assume_init_mut() }
    }

    pub fn full(&self) -> bool {
        self.space() == 0
    }

    pub fn take<R>(&mut self, out: impl FnOnce(&[u8]) -> R) -> R {
        let data = unsafe {
            // self.len is always <= self.buffer.len()
            let len = core::mem::take(&mut self.len);
            self.buffer.assume_init_mut().get_unchecked(..len)
        };

        out(data)
    }
}

pub type DecompressorResult<T> = Result<T, DecompressorError>;

pub enum DecompressorError {
    MinizError(TINFLStatus),
    ProgramError(u32),
    Overrun,
    Underrun,
}

impl From<DecompressorError> for flash_algorithm::ErrorCode {
    fn from(err: DecompressorError) -> Self {
        let code: u32 = match err {
            DecompressorError::ProgramError(status) => status,
            DecompressorError::MinizError(status) => {
                // The lowest TINFLStatus is -4, so we add 4
                // so the resulting error codes start at 10000.
                10000 + (status as i8 + 4) as u32
            }
            DecompressorError::Overrun => 20000,
            DecompressorError::Underrun => 20001,
        };

        unsafe { ErrorCode::new_unchecked(code) }
    }
}

pub struct Decompressor {
    decompressor: DecompressorOxide,
    output: OutBuffer,
    image_start: u32,
    offset: u32,
    remaining_compressed: usize,
}

impl Decompressor {
    pub fn new() -> Self {
        Self {
            image_start: 0xFFFF_FFFF,
            offset: 0,
            output: OutBuffer::new(),
            remaining_compressed: 0,
            decompressor: DecompressorOxide::new(),
        }
    }

    fn reinit(&mut self, address: u32, compressed: u32) {
        self.image_start = address;
        self.offset = 0;

        self.remaining_compressed = compressed as usize;

        self.decompressor = DecompressorOxide::new();
        self.output.take(|_| {});
    }

    fn decompress(&mut self, input: &[u8]) -> DecompressorResult<()> {
        if self.remaining_compressed == 0 {
            return Err(DecompressorError::Overrun);
        }

        // We may have to cut off some padding bytes.
        let chunk_len = self.remaining_compressed.min(input.len());
        self.remaining_compressed -= chunk_len;

        // Signal tinfl_decompress() that this is the last chunk
        let last = self.remaining_compressed == 0;

        // Iterate through all the input
        let mut input = &input[..chunk_len];
        let mut status = TINFLStatus::NeedsMoreInput;

        while !input.is_empty() && status as i8 > TINFLStatus::Done as i8 {
            let flags = if last {
                inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
            } else {
                inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
                    | inflate_flags::TINFL_FLAG_HAS_MORE_INPUT
            };

            let next_out = self.output.len;
            let out_buf = self.output.buffer();
            let (new_status, in_bytes, out_bytes) =
                decompress(&mut self.decompressor, &mut input, out_buf, next_out, flags);
            status = new_status;

            assert!(
                in_bytes <= input.len(),
                "decompress() consumed more bytes than given"
            );

            // Consume processed input
            input = &input[in_bytes..];

            // Update output buffer
            self.output.len += out_bytes;

            if status == TINFLStatus::Done || self.output.full() {
                // We're either finished or the decompressor can't continue
                // until we flush the buffer.
                self.flush()?;
            }
        }

        if (status as i8) < TINFLStatus::Done as i8 {
            return Err(DecompressorError::MinizError(status));
        }

        Ok(())
    }

    pub fn flush(&mut self) -> DecompressorResult<()> {
        let mut offset = self.offset;
        let address = self.image_start + offset;

        // Take buffer contents, write to flash and update offset.
        let status = self.output.take(|data| {
            offset += data.len() as u32;
            let mut flash_addr: u32 = address - crate::MEMORY_MAP_FLEXSPI_START_ADDRESS;
            let mut status: u32 = 0;
            for page in data.chunks(crate::PAGE_SIZE as usize) {
                status = unsafe {
                    crate::rom_api::flexspi_nor_flash_page_program(
                        crate::FLEXSPI_INSTANCE,
                        crate::NOR_CONFIG.as_ptr(),
                        flash_addr,
                        page.as_ptr() as *const u32,
                    )
                };
                flash_addr += page.len() as u32;
                if status != 0 {
                    break;
                }
            }

            status
        });

        self.offset = offset;

        if status != 0 {
            return Err(DecompressorError::ProgramError(status));
        }

        Ok(())
    }

    pub fn program(&mut self, address: u32, mut data: &[u8]) -> DecompressorResult<()> {
        if self.image_start != address {
            // Finish previous image
            self.flush()?;

            if data.len() < 4 {
                // We don't have enough bytes to read the length
                return Err(DecompressorError::Underrun);
            }

            // Image length is prepended to the first chunk, cut it off.
            let (length_bytes, remaining) = data.split_at(4);
            data = remaining;

            let compressed_length = u32::from_le_bytes([
                length_bytes[0],
                length_bytes[1],
                length_bytes[2],
                length_bytes[3],
            ]);

            self.reinit(address, compressed_length);
        }

        self.decompress(data)
    }
}
