use core::convert::TryFrom;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Chunk {
    magic_start_zero: u32,
    magic_start_one: u32,
    flags: ChunkFlags,
    target_addr: u32,
    payload_size: u32,
    block_num: u32,
    block_count: u32,
    file_size_or_family_id: u32,
    data: [u8; 476],
    magic_end: u32,
}

impl Chunk {
    const MAGIC_START_ZERO: u32 = 0x0A324655;
    const MAGIC_START_ONE: u32 = 0x9E5D5157;
    const MAGIC_END: u32 = 0x0AB16F30;

    pub fn is_supported_chunk(data: &[u8]) -> bool {
        if data.len() != 512 {
            return false;
        }

        if &Self::MAGIC_START_ZERO.to_le_bytes() != &data[..4] {
            return false;
        }

        if &Self::MAGIC_START_ONE.to_le_bytes() != &data[4..8] {
            return false;
        }

        if &Self::MAGIC_END.to_le_bytes() != &data[508..] {
            return false;
        }

        let flags = ChunkFlags::from_bits_truncate(u32::from_le_bytes([
            data[8], data[9], data[10], data[11],
        ]));

        if flags.intersects(
            ChunkFlags::NOT_MAIN_FLASH
                | ChunkFlags::IS_FILE_CONTAINER
                | ChunkFlags::HAS_MD5_CHECKSUM
                | ChunkFlags::HAS_EXTENSION_TAGS,
        ) {
            return false;
        }

        true
    }
}

bitflags! {
    struct ChunkFlags: u32 {
        const NOT_MAIN_FLASH = 1 << 0;
        const IS_FILE_CONTAINER = 1 << 1;
        const HAS_FAMILY_ID = 1 << 2;
        const HAS_MD5_CHECKSUM = 1 << 3;
        const HAS_EXTENSION_TAGS = 1 << 4;
    }
}
