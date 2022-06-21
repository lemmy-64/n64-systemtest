use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::cmp::max;
use core::mem::size_of;
use crate::MemoryMap;

/// A dynamically allocated chunk of memory that is accessed as uncached memory. This is
/// useful to talk to other devices (e.g. DMA, RDP etc)
pub struct UncachedHeapMemory<T: Copy + Clone> {
    layout: Layout,
    count: usize,
    original_data: *mut u8,
    uncached_data: *mut T,
}

impl<T: Copy + Clone> UncachedHeapMemory<T> {
    /// Allocates the memory with the given number of elements. The actual byte size will be count * size_of::<T>().
    /// The memory will be aligned to the size of T and uninitialized
    pub fn new(count: usize) -> Self {
        Self::new_with_align(count, 0)
    }

    /// Allocates the memory with the given number of elements. The actual byte size will be count * size_of::<T>().
    /// The memory will be aligned to the size of T or align (whichever is larger) and uninitialized
    pub fn new_with_align(count: usize, align: usize) -> Self {
        let element_size = size_of::<T>();
        let byte_size = count * element_size;
        let layout = Layout::from_size_align(byte_size, max(align, element_size)).unwrap();
        let original_data = unsafe { alloc(layout) };
        let uncached_data = MemoryMap::uncached_mut(original_data) as *mut T;

        Self {
            layout,
            count,
            original_data,
            uncached_data,
        }
    }

    pub fn new_with_init_value(count: usize, init_value: T) -> Self {
        let result = Self::new(count);

        // To initialize we can't use alloc_zeroes, as we need to init uncached. On the plus side,
        // we can allow arbitrary init values
        for i in 0..count {
            unsafe { result.uncached_data.add(i).write_volatile(init_value) };
        }

        result
    }

    pub const fn count(&self) -> usize { self.count }

    pub fn write(&mut self, index: usize, value: T) {
        assert!(index < self.count);
        unsafe {
            self.uncached_data.add(index).write_volatile(value);
        }
    }

    pub fn read(&mut self, index: usize) -> T {
        assert!(index < self.count);
        unsafe {
            self.uncached_data.add(index).read_volatile()
        }
    }

    /// Pointer to physical start of memory
    pub fn start_phyiscal(&mut self) -> usize {
        MemoryMap::uncached_to_physical_mut(self.uncached_data)
    }
}

impl<T: Copy + Clone> Drop for UncachedHeapMemory<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.original_data, self.layout); }
    }
}