// kernel/src/allocator/bump.rs
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub unsafe fn init_heap(
    mapper: &mut impl memory::Mapper,
    frame_allocator: &mut impl memory::FrameAllocator<Size4KiB>,
) -> Result<(), memory::HeapError> {
    let heap_start = 0x_4444_4444_0000;
    let heap_size = 100 * 1024; // 100 KiB
    let heap_end = heap_start + heap_size;

    memory::create_example_mapping(heap_start, mapper, frame_allocator);

    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }

    Ok(())
}