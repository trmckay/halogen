use alloc::{boxed::Box, string::String};

/// Generate an enum type that has the name `$ty` and each of the repeating
/// variants. Also, generate a `Debug` implementation that prints these out in a
/// human-readable format so that `expect!` and `unwrap!` prints the chain of
/// errors as if it were a stack-trace. This macro saves the need to edit
/// multiple locations when adding/changing an error type.
macro_rules! make_errors {
    { $ty:ident { $($variant:ident,)* }} => {
        /// `KernelError` is an all encompassing, nestable enum for tracing errors.
        #[derive(Clone)]
        pub enum $ty {
            $($variant(String, u32, Option<Box<KernelError>>),)*
        }

        impl core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(
                        $ty::$variant(file, line, Some(inner)) => {
                            write!(
                                f, "\n  {:20} -- {}:{} {:?}",
                                stringify!($variant), file, line, *inner
                            )
                        },
                        $ty::$variant(file, line, None) => {
                            write!(
                                f, "\n  {:20} -- {}:{}",
                                stringify!($variant), file, line
                            )
                        },
                    )*
                }
            }
        }

        impl<T> From<$ty> for Result<T, $ty> {
            fn from(err: $ty) -> Result<T, $ty> {
                Err(err)
            }
        }
    };
}

make_errors! {
    KernelError {
        ProcessCreate,
        ThreadCreate,
        NoSuchThread,
        ExecutableFormat,
        OutOfVirtualAddresses,
        OutOfPhysicalFrames,
        HeapAllocationOutOfSpace,
        HeapInvalidFree,
        StackAllocation,
        PageTableAllocation,
        InvalidMapping,
        PageTableCorruption,
        Sbi,
    }
}

pub type KernelResult<T> = Result<T, KernelError>;

/// Construct a new kernel error, optionally with a causing error. This can be
/// chained to build psuedo-stack-traces.
///
/// # Example
///
/// ```
/// fn foo<T>(will_fail: bool) -> Result<T, KernelError> {
///     if will_fail {
///         kerror!().into()
///     }
/// }
///
/// fn bar<T>() -> Result<T, KernelError> {
///     match foo(true) {
///       Ok(t) => Ok(t),
///       Err(why) => kerror!(KernelError::Undefined, why).into()
///     }
/// }
/// ```
#[macro_export]
macro_rules! kerror {
    ($variant:path, $cause:expr) => {
        $variant(
            alloc::string::String::from(file!()),
            line!(),
            Some(alloc::boxed::Box::new($cause)),
        )
    };
    ($variant:path) => {
        $variant(alloc::string::String::from(file!()), line!(), None)
    };
    () => {
        $crate::error::KernelError::Undefined(None)
    };
}
