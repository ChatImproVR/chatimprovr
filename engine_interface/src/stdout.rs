extern "C" {
    fn _print(ptr: *const u8, len: usize);
}

pub fn _print_str(s: &str) {
    unsafe {
        _print(s.as_ptr(), s.len());
    }
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {{
        // TODO: Yes I am aware this is slow. It is also EASY
        _print_str(&format_args!($($arg)*).to_string());
    }};
}

#[macro_export]
macro_rules! printlnk {
    ($($arg:tt)*) => {{
        // TODO: Yes I am aware this is slow. It is also EASY
        let s = format_args!($($arg)*).to_string();
        _print_str(&(s + "\n"));
    }};
}

#[macro_export]
macro_rules! dbgk {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::printlnk!("[{}:{}]", $crate::file!(), $crate::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::printlnk!("[{}:{}] {} = {:#?}",
                    std::file!(), std::line!(), std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}