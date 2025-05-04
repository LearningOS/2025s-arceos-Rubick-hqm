#![no_std]

use core::{alloc::Layout, ptr::NonNull, usize};

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    byte_start: usize,
    byte_pos: usize,
    page_pos: usize,
    end: usize,
    byte_count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            byte_start: 0,
            byte_pos: 0,
            page_pos: 0,
            end: 0,
            byte_count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.byte_start = start;
        self.byte_pos = start;
        self.page_pos = start + size;
        self.end = start + size;
        self.byte_count = size;
    }
    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        unimplemented!()
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();

        let aligned_pos = (self.byte_pos + align - 1) & !(align - 1);
        let next_pos = aligned_pos.checked_add(size).ok_or(AllocError::NoMemory)?;

        if next_pos > self.page_pos {
            return Err(AllocError::NoMemory);
        }

        self.byte_pos = next_pos;
        self.byte_count += 1;

        unsafe { Ok(NonNull::new_unchecked(aligned_pos as *mut u8)) }
    }
    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        if self.byte_count > 0 {
            self.byte_count -= 1;
            if self.byte_count == 0 {
                self.byte_pos = self.byte_start;
            }
        }
    }
    fn total_bytes(&self) -> usize {
        self.page_pos - self.byte_start
    }
    fn used_bytes(&self) -> usize {
        self.byte_pos - self.byte_start
    }
    fn available_bytes(&self) -> usize {
        self.page_pos - self.byte_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }

        let align_pages = align_pow2 / PAGE_SIZE;
        if !align_pages.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        let align_mask = !(align_pages - 1);
        let total_size = num_pages * PAGE_SIZE;

        let mut new_page_pos = self
            .page_pos
            .checked_sub(total_size)
            .ok_or(AllocError::NoMemory)?;
        new_page_pos &= align_mask * PAGE_SIZE; // align page address

        if new_page_pos < self.byte_pos {
            return Err(AllocError::NoMemory);
        }

        self.page_pos = new_page_pos;
        Ok(new_page_pos)
    }
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        unimplemented!()
    }
    fn total_pages(&self) -> usize {
        (self.end - self.byte_pos) / PAGE_SIZE
    }
    fn used_pages(&self) -> usize {
        (self.end - self.page_pos) / PAGE_SIZE
    }
    fn available_pages(&self) -> usize {
        (self.page_pos - self.byte_pos) / PAGE_SIZE
    }
}
