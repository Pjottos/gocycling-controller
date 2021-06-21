use crate::{uf2, binding::*};

use p256::ecdsa;

use core::{mem, slice};

pub enum Status {
    Progress,
    ChunkDone,
    Complete,
    ChunkInvalid,
}

pub enum Error {
    TooManyChunks,
}

pub struct FirmwareDownloader {
    chunk_buf: [uf2::Chunk; Self::MAX_CHUNK_COUNT],
    chunk_count: usize,
    chunk_buf_offset: usize,
    expected_chunk_count: usize,
    cur_crc: u32,
    cur_crc_offset: u32,
}

impl FirmwareDownloader {
    const MAX_CHUNK_COUNT: usize = 128;

    pub fn new(expected_chunk_count: usize) -> Result<Self, Error> {
        if expected_chunk_count > Self::MAX_CHUNK_COUNT {
            Err(Error::TooManyChunks)
        } else {
            Ok(Self {
                chunk_buf: unsafe { mem::MaybeUninit::zeroed().assume_init() },
                chunk_count: 0,
                chunk_buf_offset: 0,
                expected_chunk_count,
                cur_crc: 0,
                cur_crc_offset: 0,
            })
        }
    }

    pub fn feed(&mut self, byte: u8) -> Status {
        if self.chunk_count == self.expected_chunk_count {
            return Status::Complete;
        }

        if self.cur_crc_offset < 4 {
            self.cur_crc |= (byte as u32) << (self.cur_crc_offset * 8);
            self.cur_crc_offset += 1;
            return Status::Progress;
        }

        let chunk_bytes = unsafe {
            let chunk_data = self.chunk_buf.as_mut_ptr();
            slice::from_raw_parts_mut(
                chunk_data as *mut u8,
                self.chunk_buf.len() * mem::size_of::<uf2::Chunk>(),
            )
        };

        // must be valid because of checks in new()
        chunk_bytes[self.chunk_buf_offset] = byte;
        self.chunk_buf_offset += 1;

        if self.chunk_buf_offset % mem::size_of::<uf2::Chunk>() == 0 {
            let start = self.chunk_buf_offset - mem::size_of::<uf2::Chunk>();
            let end = self.chunk_buf_offset;
            let chunk_data = &chunk_bytes[start..end];

            let crc = self.cur_crc;
            self.reset_crc();

            if calc_crc32(chunk_data) != crc {
                return Status::ChunkInvalid;
            }

            if uf2::Chunk::is_supported_chunk(chunk_data) {
                self.chunk_count += 1;
                if self.chunk_count == self.expected_chunk_count {
                    Status::Complete
                } else {
                    Status::ChunkDone
                }
            } else {
                // discard the bytes belonging to the last chunk size
                self.chunk_buf_offset = start;
                Status::ChunkInvalid
            }
        } else {
            Status::Progress
        }
    }

    pub unsafe fn apply_update(&self) -> ! {
		let _ = binding_save_and_disable_interrupts();
		todo!();
    }

    fn reset_crc(&mut self) {
        self.cur_crc = 0;
        self.cur_crc_offset = 0;
    }
}

fn calc_crc32(slice: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFF;

    for b in slice.iter().copied() {
        crc = crc ^ (b as u32);
        for _ in 0..8 {
            let mask = (!(crc & 1)).overflowing_add(1).0;
            crc = (crc >> 1) ^ (0xEDB88320 & mask);
        }
    }

    !crc
}
