/// Macros for physical memory.
pub mod phys;

pub mod pages;

pub const NULL_PTR: *const u8 = 0 as *const u8;
pub const NULL_PTR_MUT: *mut u8 = 0 as *mut u8;

extern "C" {
    pub static TEXT_BEGIN: usize;
    pub static K_STACK_BEGIN: usize;
    pub static K_STACK_END: usize;
    pub static K_HEAP_BEGIN: usize;
    pub static K_HEAP_SIZE: usize;
    pub static MEM_END: usize;
}

#[macro_export]
macro_rules! text_begin {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::TEXT_BEGIN as *const u8 })
    }};
}

#[macro_export]
macro_rules! text_end {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_BEGIN as *const u8 })
    }};
}

#[macro_export]
macro_rules! text_size {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_BEGIN - crate::mem::TEXT_BEGIN })
    }};
}

#[macro_export]
macro_rules! k_stack_begin {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_BEGIN as *const u8 })
    }};
}

#[macro_export]
macro_rules! k_stack_end {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_END as *const u8 })
    }};
}

#[macro_export]
macro_rules! k_stack_size {
    () => {{
        #[allow(unused_unsafe)]
        (unsafe { crate::mem::K_STACK_END - crate::mem::K_STACK_BEGIN })
    }};
}

#[macro_export]
macro_rules! k_heap_begin {
    () => {{
        #[allow(unused_unsafe)]
        unsafe {
            crate::mem::K_HEAP_BEGIN as *mut u8
        }
    }};
}

#[macro_export]
macro_rules! k_heap_end {
    () => {{
        #[allow(unused_unsafe)]
        unsafe {
            (crate::mem::K_HEAP_BEGIN + crate::mem::K_HEAP_SIZE) as *mut u8
        }
    }};
}

#[macro_export]
macro_rules! k_heap_size {
    () => {{
        #[allow(unused_unsafe)]
        unsafe {
            crate::mem::K_HEAP_SIZE
        }
    }};
}
