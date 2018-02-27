// TODO: install exception handler to deal with hio semihosting not being available
// and just ignore bkpts if no debugger attached

use core::fmt;
use core::fmt::Write;
use cortex_m_semihosting::hio;

#[cfg(feature = "use_semihosting")]
#[macro_export]
macro_rules! debug {
    ($($arg: tt)*) => {
        match hio::hstdout() {
            Ok(ref mut stdout) => write!(stdout, $($arg)*),
            _ => Ok(())
        };
    }
}

#[cfg(not(feature = "use_semihosting"))]
#[macro_export]
macro_rules! debug {
    ($($arg: tt)*) => {
        {
            let res: Result<(), ()> = Ok(());
            res
        }
    }
}

pub trait UnwrapLog {
    fn log_error(self);
}

impl<E: fmt::Debug> UnwrapLog for Result<(), E>
{
    #[inline]
    fn log_error(self) {
        match self {
            Err(e) => { debug!("{:?}", e).unwrap() },
            _ => {},
        }
    }
}
