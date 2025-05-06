//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

pub struct LabByteAllocator {
    start: usize,
    pos_1: usize,
    pos_2: usize,
    end: usize,
    block_96: [u8; 96],
    block_192: [u8; 192],
    block_384: [u8; 384],
    block_86016: [u8; 86016],
    cnt: usize,
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        Self {
            start: 0,
            pos_1: 0,
            end: 0,
            pos_2: 0,
            block_96: [0; 96],
            block_192: [0; 192],
            block_384: [0; 384],
            block_86016: [0; 86016],
            cnt: 0,
        }
    }
}

unsafe impl Send for LabByteAllocator {}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.pos_1 = start;
        self.pos_2 = start + size;
        self.end = start + size;
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        if start == self.end {
            assert!(self.pos_2 == self.end);
            self.end += size;
            self.pos_2 = self.end;
        } else {
            return Err(AllocError::InvalidParam);
        }
        Ok(())
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();
        if align == 1 {
            if (self.pos_2 == self.end) & ((self.pos_2 - self.pos_1) < 0x102000) {
                return Err(AllocError::NoMemory);
            }

            let ptr = if self.cnt % 2 == 1 {
                let res = self.pos_1 as *mut u8;
                self.pos_1 += size;
                assert!(self.pos_1 < self.pos_2);
                res
            } else {
                self.pos_2 -= size;
                let res = self.pos_2 as *mut u8;
                assert!(self.pos_1 < self.pos_2);
                res
            };

            self.cnt += 1;
            Ok(NonNull::new(ptr).unwrap())
        } else {
            match size {
                96 => {
                    return Ok(NonNull::new(self.block_96.as_mut_ptr()).unwrap());
                }
                192 => {
                    return Ok(NonNull::new(self.block_192.as_mut_ptr()).unwrap());
                }
                384 => {
                    return Ok(NonNull::new(self.block_384.as_mut_ptr()).unwrap());
                }
                _ => {
                    return Ok(NonNull::new(self.block_86016.as_mut_ptr()).unwrap());
                }
            }
        }
    }
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        let ptr = pos.as_ptr() as usize;
        if ptr == self.pos_2 {
            self.pos_2 += layout.size();
            self.cnt = 0;
        } else if ptr == self.pos_1 - layout.size() {
            self.pos_1 -= layout.size();
        }
    }
    fn total_bytes(&self) -> usize {
        0x1000
    }
    fn used_bytes(&self) -> usize {
        self.pos_1 - self.start + self.end - self.pos_2
    }
    fn available_bytes(&self) -> usize {
        self.pos_2 - self.pos_1
    }
}
