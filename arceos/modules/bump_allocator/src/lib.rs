#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

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
// Helper functions
mod private {
    #[inline]
    pub const fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }

    #[inline]
    pub const fn align_down(addr: usize, align: usize) -> usize {
        addr & !(align - 1)
    }

    #[inline]
    pub const fn is_aligned(addr: usize, align: usize) -> bool {
        addr & (align - 1) == 0
    }

    #[inline]
    pub const fn is_power_of_two(x: usize) -> bool {
        x != 0 && (x & (x - 1)) == 0
    }
}

use private::*;
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    bytes_count: usize,
    used_pages: usize,
    total_pages: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    /// Creates a new empty `EarlyAllocator`.
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            bytes_count: 0,
            used_pages: 0,
            total_pages: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());
        
        // Align start and end addresses
        let start = crate::align_up(start, PAGE_SIZE);
        let end = crate::align_down(start + size, PAGE_SIZE);
        
        self.start = start;
        self.end = end;
        self.b_pos = start;
        self.p_pos = end;
        self.bytes_count = 0;
        self.used_pages = 0;
        self.total_pages = (end - start) / PAGE_SIZE;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();
        
        // Align b_pos to the required alignment
        let aligned_pos = crate::align_up(self.b_pos, align);
        let new_pos = aligned_pos + size;
        
        // Check if there is enough space
        if new_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }
        
        // Update position and count
        let result = aligned_pos;
        self.b_pos = new_pos;
        self.bytes_count += 1;
        
        Ok(unsafe { NonNull::new_unchecked(result as *mut u8) })
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        // Only decrement the count
        self.bytes_count -= 1;
        
        // If count reaches zero, reset b_pos to start
        if self.bytes_count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        // Check if the alignment is valid
        if !crate::is_power_of_two(align_pow2) || align_pow2 > PAGE_SIZE {
            return Err(AllocError::InvalidParam);
        }
        
        // Calculate required space and alignment
        let size = num_pages * PAGE_SIZE;
        let aligned_pos = crate::align_down(self.p_pos - size, align_pow2);
        
        // Check if there is enough space
        if aligned_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }
        
        // Update position and count
        self.p_pos = aligned_pos;
        self.used_pages += num_pages;
        
        Ok(aligned_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Pages are never freed in this allocator
    }
    
    fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn used_pages(&self) -> usize {
        self.used_pages
    }

    fn available_pages(&self) -> usize {
        self.total_pages - self.used_pages
    }
}
