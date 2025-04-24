#![no_std]

use core::alloc::Layout;
use core::ptr::NonNull;
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
pub struct EarlyAllocator<const PAGE_SIZE:usize>{
    b_pos: usize,
    p_pos: usize,
    start: usize,
    end: usize,
    sum: usize,
}

impl<const PAGE_SIZE:usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new()->Self{
        Self{
            b_pos: 0,
            p_pos: 0,
            start: 0,
            end: 0,
            sum: 0,
        }
    }
}

impl<const PAGE_SIZE:usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.b_pos=start;
        self.p_pos=start+size;
        self.start=start;
        self.end=start+size;
        self.sum=0;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        unreachable!()
    }
}

impl<const PAGE_SIZE:usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align=layout.align();
        let start=self.b_pos.next_multiple_of(align);
        self.b_pos=start+layout.size();
        if self.b_pos>self.p_pos {
            return Err(AllocError::NoMemory);
        }
        unsafe {
            Ok(NonNull::new_unchecked(start as *mut u8))
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout) {
        self.sum-=1;
        if self.sum==0 {
            self.b_pos=self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.p_pos-self.b_pos
    }

    fn used_bytes(&self) -> usize {
        self.p_pos-self.b_pos
    }

    fn available_bytes(&self) -> usize {
        self.p_pos-self.b_pos
    }
}

impl<const PAGE_SIZE:usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = 0;

    fn alloc_pages(&mut self, num_pages: usize, _align_pow2: usize) -> AllocResult<usize> {
        if self.sum==0{
            self.p_pos-=num_pages*PAGE_SIZE;
            self.sum=num_pages;
        }
        self.sum-=1;
        Ok(self.p_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, num_pages: usize) {
        self.sum+=1;
        if self.sum==0 {
            self.p_pos+=num_pages*PAGE_SIZE;
        }
    }

    fn total_pages(&self) -> usize {
        self.p_pos-self.b_pos
    }

    fn used_pages(&self) -> usize {
        self.p_pos-self.b_pos
    }

    fn available_pages(&self) -> usize {
        self.p_pos-self.b_pos
    }
}
