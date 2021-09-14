#[cfg(ll_alloc)]
pub mod ll_alloc;

pub trait PageAllocator {
    fn init(&self);
    fn alloc(&self, size: usize) -> Option<*mut u8>;
    fn free(&self, page: *mut u8);
}
