#[macro_export]
macro_rules! guise_log {
    (target: $target:expr, $($arg:tt)+) => (
        #[cfg(feature = "log")] {
            log::log!(target: $target, log::Level::Debug, $($arg)+)
        }
    );
    ($($arg:tt)+) => (
        #[cfg(feature = "log")] {
            log::log!(log::Level::Debug, $($arg)+)
        }
    )
}

#[macro_export]
macro_rules! guise_dbg {
    (target: $target:expr, $($arg:tt)+) => (
        #[cfg(feature = "log")] {
            log::log!(target: $target, log::Level::Error, $($arg)+)
        }
    );
    ($($arg:tt)+) => (
        #[cfg(feature = "log")] {
            log::log!(log::Level::Error, $($arg)+)
        }
    )
}
