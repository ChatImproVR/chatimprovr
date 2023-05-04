use log::{Level, Metadata, Record};

extern "C" {
    /// Host function for logging
    #[cfg(target_family = "wasm")]
    fn _log(ptr: *const u8, len: usize, level: u32);
}

/// Safe, convenient abstraction over raw _log()
pub fn _log_str(s: &str, level: Level) {
    #[cfg(target_family = "wasm")]
    unsafe {
        _log(s.as_ptr(), s.len(), level as usize as u32);
    }

    #[cfg(not(target_family = "wasm"))]
    println!("{} {:?}", s, level);
}

/// Set up printing for panics
/// NOTE: It's okay to call this before setup_logging(), because
/// it calls the _log() hostcall directly
pub(crate) fn setup_panic() {
    std::panic::set_hook(Box::new(|e| {
        _log_str(&format!("{e:#}\n"), Level::Error);
    }))
}

/// Logger instance
static LOGGER: LogToHost = LogToHost;

/// Setup logging to host
pub(crate) fn setup_logging() {
    log::set_logger(&LOGGER).unwrap();
}

/// Writes log messages with the _log() hostcall
struct LogToHost;

impl log::Log for LogToHost {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // It's up to the host!
        true
    }

    fn log(&self, record: &Record) {
        // TODO: Send all metadata!
        // TODO: Avoid allocating each log message
        _log_str(&format!("{}", record.args()), record.metadata().level())
    }

    // Logs are already flushed each message!
    fn flush(&self) {}
}

/// Similar to the print!() macro from the stdlib, but for plugins.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        // TODO: Yes I am aware this is slow. It is also EASY
        $crate::log::info!($($arg)*)
    }};
}

/// Similar to the println!() macro from the stdlib, but for plugins.
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        $crate::log::info!($($arg)*)
    }};
}

/// Similar to the dbg!() macro from the stdlib, but for plugins.
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::log::debug!("[{}:{}]", $crate::file!(), $crate::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::log::debug!("[{}:{}] {} = {:#?}",
                    std::file!(), std::line!(), std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
