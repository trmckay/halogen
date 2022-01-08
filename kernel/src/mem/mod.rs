/// Tools for interacting with physical memory
pub mod phys;

/// Implementation of the RISC-V Sv39 memory-paging system
pub mod sv39;

/// Global symbols imported from the linker
extern "C" {
    pub static TEXT_BEGIN: usize;
    pub static K_STACK_BEGIN: usize;
    pub static K_STACK_END: usize;
    pub static K_HEAP_BEGIN: usize;
    pub static K_HEAP_SIZE: usize;
    pub static MEM_END: usize;
}

/// Macro to get the beginning address of the text section
#[macro_export]
macro_rules! text_begin {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::TEXT_BEGIN as *const u8 })
    }};
}

/// Macro to get the end address of the text section
#[macro_export]
macro_rules! text_end {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_BEGIN as *const u8 })
    }};
}

/// Macro to get the size of the text section
#[macro_export]
macro_rules! text_size {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_BEGIN - crate::mem::TEXT_BEGIN })
    }};
}

/// Macro to get the beginning address of the kernel stack
#[macro_export]
macro_rules! k_stack_begin {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_BEGIN as *const u8 })
    }};
}

/// Macro to get the end address of the kernel stack
#[macro_export]
macro_rules! k_stack_end {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_END as *const u8 })
    }};
}

/// Macro to get the size of the kernel stack
#[macro_export]
macro_rules! k_stack_size {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_END - crate::mem::K_STACK_BEGIN })
    }};
}

/// Macro to get the beginning address of the kernel heap
#[macro_export]
macro_rules! k_heap_begin {
    () => {{
        #[allow(unused_unsafe)]
        unsafe {
            crate::mem::K_HEAP_BEGIN as *mut u8
        }
    }};
}

/// Macro to get the end address of the kernel heap
#[macro_export]
macro_rules! k_heap_end {
    () => {{
        #[allow(unused_unsafe)]
        unsafe {
            (crate::mem::K_HEAP_BEGIN + crate::mem::K_HEAP_SIZE) as *mut u8
        }
    }};
}

/// Macro to get the end size the kernel heap
#[macro_export]
macro_rules! k_heap_size {
    () => {{
        #[allow(unused_unsafe)]
        unsafe {
            crate::mem::K_HEAP_SIZE
        }
    }};
}
