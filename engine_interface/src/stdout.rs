extern "C" {
    #[cfg(target_family = "wasm")]
    fn _print(ptr: *const u8, len: usize);
}

pub fn _print_str(s: &str) {
    #[cfg(target_family = "wasm")]
    unsafe {
        _print(s.as_ptr(), s.len());
    }

    #[cfg(not(target_family = "wasm"))]
    println!("{}", s);
}

/// Set up printing for panics
pub(crate) fn setup_panic() {
    std::panic::set_hook(Box::new(|e| {
        _print_str(&(e.to_string() + "\n"));
    }))
}

/// Similar to the print!() macro from the stdlib, but for plugins.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        // TODO: Yes I am aware this is slow. It is also EASY
        _print_str(&format_args!($($arg)*).to_string());
    }};
}

/// Similar to the println!() macro from the stdlib, but for plugins.
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        // TODO: Yes I am aware this is slow. It is also EASY
        let s = format_args!($($arg)*).to_string();
        _print_str(&(s + "\n"));
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
        $crate::println!("[{}:{}]", $crate::file!(), $crate::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    std::file!(), std::line!(), std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
