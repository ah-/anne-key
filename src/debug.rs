// TODO: install exception handler to deal with hio semihosting not being available
// and just ignore bkpts if no debugger attached
use core::fmt;

#[cfg(not(feature = "use_semihosting"))]
#[macro_export]
macro_rules! heprintln {
    ($($arg:tt)*) => {{
        let res: Result<(), ()> = Ok(());
        res
    }};
}

pub trait UnwrapLog {
    fn log_error(self);
}

impl<E: fmt::Debug> UnwrapLog for Result<(), E> {
    #[inline]
    #[cfg(feature = "use_semihosting")]
    fn log_error(self) {
        match self {
            Err(e) => crate::heprintln!("{:?}", e).unwrap(),
            _ => {}
        }
    }

    #[inline]
    #[cfg(not(feature = "use_semihosting"))]
    fn log_error(self) {}
}
