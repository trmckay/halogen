#[macro_export]
macro_rules! bit {
    ($n:expr) => {
        (0b1 << $n)
    };
}

#[macro_export]
macro_rules! exit_failure {
    () => {
        use qemu_exit::QEMUExit;
        crate::QEMU_EXIT.exit_failure();
    };
}

#[macro_export]
macro_rules! exit_success {
    () => {
        use qemu_exit::QEMUExit;
        crate::QEMU_EXIT.exit_success();
    };
}
