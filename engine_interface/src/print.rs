extern "C" {
    fn _print(ptr: *const u8, len: usize);
}

pub fn _print_str(s: &str) {
    unsafe {
        _print(s.as_ptr(), s.len());
    }
}

#[macro_export]
macro_rules! printkkk {
    ($($arg:tt)*) => {{
        _print_str(&format_args!($($arg)*).to_string());
    }};
}