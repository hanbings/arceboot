use alloc::vec::Vec;
use axhal::{
    mem::{MemoryAddr, PhysAddr, VirtAddr},
    paging::MappingFlags,
};
use axsync::Mutex;
use uefi_raw::table::boot::MemoryType;

static ALLOCATED_PAGES: Mutex<Vec<(VirtAddr, usize)>> = Mutex::new(Vec::new());
static ALLOCATED_POOLS: Mutex<Vec<(usize, core::alloc::Layout)>> = Mutex::new(Vec::new());

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AllocateType {
    AnyPages = 0,   // AllocateAnyPages
    MaxAddress = 1, // AllocateMaxAddress
    Address = 2,    // AllocateAddress
}

impl TryFrom<u32> for AllocateType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AllocateType::AnyPages),
            1 => Ok(AllocateType::MaxAddress),
            2 => Ok(AllocateType::Address),
            _ => Err(()),
        }
    }
}

impl From<AllocateType> for u32 {
    fn from(v: AllocateType) -> u32 {
        v as u32
    }
}

pub fn alloc_pages(_alloc_type: AllocateType, _memory_type: MemoryType, count: usize) -> *mut u8 {
    let layout = core::alloc::Layout::from_size_align(count * 4096, 4096)
        .expect("Invalid layout for allocate_pages");
    let ptr = axalloc::global_allocator()
        .alloc(layout)
        .expect("Failed to allocate pages for EFI")
        .as_ptr();

    let page_count = (layout.size() + 4095) / 4096;

    axmm::kernel_aspace()
        .lock()
        .protect(
            VirtAddr::from_ptr_of(ptr).align_down(4096usize),
            page_count * 4096,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE,
        )
        .expect("Failed to protect EFI memory");

    // Record allocation for later deallocation
    let vaddr = VirtAddr::from_ptr_of(ptr);
    ALLOCATED_PAGES.lock().push((vaddr, count));

    ptr
}

pub fn free_pages(addr: PhysAddr, pages: usize) {
    use axhal::mem::phys_to_virt;

    let vaddr = phys_to_virt(addr);
    let mut allocated = ALLOCATED_PAGES.lock();

    // Find the allocation record matching this address
    if let Some(idx) = allocated.iter().position(|(v, _)| *v == vaddr) {
        let (_, recorded_pages) = allocated.swap_remove(idx);

        // Use the recorded page count if available, otherwise use the provided count
        let page_count = if recorded_pages > 0 { recorded_pages } else { pages };

        let layout = core::alloc::Layout::from_size_align(page_count * 4096, 4096)
            .expect("Invalid layout for free_pages");

        // Safety: pointer and layout came from our allocator in alloc_pages
        unsafe {
            axalloc::global_allocator().dealloc(
                core::ptr::NonNull::new_unchecked(vaddr.as_mut_ptr()),
                layout,
            );
        }
    } else {
        // Address not found in our records - this could be:
        // 1. A double-free attempt
        // 2. Memory allocated by other means
        // Log a warning but don't panic to maintain UEFI compatibility
        axlog::warn!(
            "free_pages: address {:#x} not found in allocation records",
            addr.as_usize()
        );
    }
}

pub fn allocate_pool(_memory_type: MemoryType, size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }
    // UEFI requires at least 8-byte alignment for pool allocations.
    let layout = match core::alloc::Layout::from_size_align(size, 8) {
        Ok(l) => l,
        Err(_) => return core::ptr::null_mut(),
    };
    let ptr = match axalloc::global_allocator().alloc(layout) {
        Ok(nn) => nn.as_ptr(),
        Err(_) => return core::ptr::null_mut(),
    };
    ALLOCATED_POOLS.lock().push((ptr as usize, layout));
    ptr
}

pub fn free_pool(buffer: *mut u8) {
    if buffer.is_null() {
        return;
    }
    let addr = buffer as usize;
    let mut pools = ALLOCATED_POOLS.lock();
    if let Some(idx) = pools.iter().position(|(p, _)| *p == addr) {
        let (_, layout) = pools.swap_remove(idx);
        // Safety: pointer/layout came from our allocator.
        unsafe {
            axalloc::global_allocator().dealloc(
                core::ptr::NonNull::new_unchecked(buffer),
                layout,
            )
        };
    }
}
