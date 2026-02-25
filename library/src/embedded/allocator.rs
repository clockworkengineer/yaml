//! Embedded Custom Allocator
//!
//! Provides arena-based and bump allocation patterns for embedded and resource-constrained environments.
//! Designed for use in YAML parsing and manipulation on systems without standard allocators.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Simple bump allocator for embedded systems
/// Allocates memory from a fixed-size buffer in a linear fashion
#[derive(Debug)]
pub struct BumpAllocator {
    buffer: [u8; 4096], // 4KB fixed buffer
    offset: usize,
}

impl BumpAllocator {
    /// Creates a new bump allocator
    pub const fn new() -> Self {
        Self {
            buffer: [0; 4096],
            offset: 0,
        }
    }

    /// Allocates memory for a slice of bytes
    /// Returns None if insufficient space
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<&mut [u8]> {
        let align_offset = (self.offset + align - 1) & !(align - 1);
        let end = align_offset.checked_add(size)?;

        if end > self.buffer.len() {
            return None;
        }

        self.offset = end;
        Some(&mut self.buffer[align_offset..end])
    }

    /// Resets the allocator, allowing memory reuse
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Returns the amount of used memory
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Returns the amount of available memory
    pub fn available(&self) -> usize {
        self.buffer.len() - self.offset
    }

    /// Returns the total capacity
    pub const fn capacity(&self) -> usize {
        self.buffer.len()
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory pool for fixed-size allocations
/// Useful for frequently allocated/deallocated objects of the same size
#[derive(Debug)]
pub struct FixedSizePool<const SIZE: usize, const COUNT: usize> {
    blocks: [[u8; SIZE]; COUNT],
    free_list: [bool; COUNT],
}

impl<const SIZE: usize, const COUNT: usize> FixedSizePool<SIZE, COUNT> {
    /// Creates a new fixed-size memory pool
    pub const fn new() -> Self {
        Self {
            blocks: [[0; SIZE]; COUNT],
            free_list: [true; COUNT],
        }
    }

    /// Allocates a block from the pool
    /// Returns None if no blocks available
    pub fn alloc(&mut self) -> Option<&mut [u8; SIZE]> {
        for i in 0..COUNT {
            if self.free_list[i] {
                self.free_list[i] = false;
                return Some(&mut self.blocks[i]);
            }
        }
        None
    }

    /// Deallocates a block back to the pool
    /// # Safety
    /// The caller must ensure the block was allocated from this pool
    pub unsafe fn dealloc(&mut self, block: *mut [u8; SIZE]) {
        let pool_start = self.blocks.as_ptr() as usize;
        let pool_end = pool_start + (SIZE * COUNT);
        let block_addr = block as usize;

        if block_addr >= pool_start && block_addr < pool_end {
            let index = (block_addr - pool_start) / SIZE;
            if index < COUNT {
                self.free_list[index] = true;
            }
        }
    }

    /// Returns the number of free blocks
    pub fn free_count(&self) -> usize {
        self.free_list.iter().filter(|&&free| free).count()
    }

    /// Returns the number of allocated blocks
    pub fn allocated_count(&self) -> usize {
        COUNT - self.free_count()
    }
}

impl<const SIZE: usize, const COUNT: usize> Default for FixedSizePool<SIZE, COUNT> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_allocator_creation() {
        let allocator = BumpAllocator::new();
        assert_eq!(allocator.used(), 0);
        assert_eq!(allocator.available(), 4096);
        assert_eq!(allocator.capacity(), 4096);
    }

    #[test]
    fn test_bump_allocator_alloc() {
        let mut allocator = BumpAllocator::new();
        let slice1 = allocator.alloc(100, 1);
        assert!(slice1.is_some());
        assert_eq!(slice1.unwrap().len(), 100);
        assert_eq!(allocator.used(), 100);
        let slice2 = allocator.alloc(200, 1);
        assert!(slice2.is_some());
        assert_eq!(slice2.unwrap().len(), 200);
        assert_eq!(allocator.used(), 300);
    }

    #[test]
    fn test_bump_allocator_alignment() {
        let mut allocator = BumpAllocator::new();
        let slice1 = allocator.alloc(5, 8);
        assert!(slice1.is_some());
        let slice2 = allocator.alloc(10, 8);
        assert!(slice2.is_some());
    }

    #[test]
    fn test_bump_allocator_out_of_memory() {
        let mut allocator = BumpAllocator::new();
        let slice = allocator.alloc(5000, 1);
        assert!(slice.is_none());
    }

    #[test]
    fn test_bump_allocator_reset() {
        let mut allocator = BumpAllocator::new();
        allocator.alloc(100, 1);
        assert_eq!(allocator.used(), 100);
        allocator.reset();
        assert_eq!(allocator.used(), 0);
        assert_eq!(allocator.available(), 4096);
    }

    #[test]
    fn test_bump_allocator_full_reset_and_reuse() {
        let mut allocator = BumpAllocator::new();
        let slice1 = allocator.alloc(4096, 1);
        assert!(slice1.is_some());
        allocator.reset();
        let slice2 = allocator.alloc(4096, 1);
        assert!(slice2.is_some());
    }

    #[test]
    fn test_fixed_size_pool_creation() {
        let pool: FixedSizePool<64, 10> = FixedSizePool::new();
        assert_eq!(pool.free_count(), 10);
        assert_eq!(pool.allocated_count(), 0);
    }

    #[test]
    fn test_fixed_size_pool_alloc() {
        let mut pool: FixedSizePool<64, 10> = FixedSizePool::new();

        let block1 = pool.alloc();
        assert!(block1.is_some());
        assert_eq!(pool.free_count(), 9);
        assert_eq!(pool.allocated_count(), 1);

        let block2 = pool.alloc();
        assert!(block2.is_some());
        assert_eq!(pool.free_count(), 8);
        assert_eq!(pool.allocated_count(), 2);
    }

    #[test]
    fn test_fixed_size_pool_exhaustion() {
        let mut pool: FixedSizePool<64, 3> = FixedSizePool::new();

        assert!(pool.alloc().is_some());
        assert!(pool.alloc().is_some());
        assert!(pool.alloc().is_some());
        assert!(pool.alloc().is_none()); // Pool exhausted

        assert_eq!(pool.free_count(), 0);
        assert_eq!(pool.allocated_count(), 3);
    }

    #[test]
    fn test_fixed_size_pool_alloc_dealloc() {
        let mut pool: FixedSizePool<32, 2> = FixedSizePool::new();
        let block1_ptr;
        let block2_ptr;
        {
            let block1 = pool.alloc().unwrap();
            block1_ptr = block1 as *mut [u8; 32];
            let block2 = pool.alloc().unwrap();
            block2_ptr = block2 as *mut [u8; 32];
        }
        assert_eq!(pool.free_count(), 0);
        unsafe {
            pool.dealloc(block1_ptr);
            pool.dealloc(block2_ptr);
        }
        assert_eq!(pool.free_count(), 2);
    }

    #[test]
    fn test_fixed_size_pool_alloc_dealloc_reuse() {
        let mut pool: FixedSizePool<16, 1> = FixedSizePool::new();
        let block_ptr;
        {
            let block = pool.alloc().unwrap();
            block_ptr = block as *mut [u8; 16];
        }
        unsafe {
            pool.dealloc(block_ptr);
        }
        let block2 = pool.alloc();
        assert!(block2.is_some());
    }

    #[test]
    fn test_bump_allocator_multiple_allocs() {
        let mut allocator = BumpAllocator::new();

        for i in 1..=10 {
            let slice = allocator.alloc(100, 1);
            assert!(slice.is_some());
            assert_eq!(allocator.used(), i * 100);
        }
    }

    #[test]
    fn test_bump_allocator_capacity_boundary() {
        let mut allocator = BumpAllocator::new();

        // Allocate almost full capacity
        let slice1 = allocator.alloc(4000, 1);
        assert!(slice1.is_some());

        // Should fit
        let slice2 = allocator.alloc(96, 1);
        assert!(slice2.is_some());

        // Should not fit
        let slice3 = allocator.alloc(1, 1);
        assert!(slice3.is_none());
    }
}
