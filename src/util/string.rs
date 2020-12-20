use alloc::string::{String, ToString};

use crate::include::{UNICODE_STRING, UNICODE_STRING32};

use crate::include::UCHAR;
use cstr_core::CString;
use core::convert::TryInto;
use alloc::vec::Vec;

pub fn create_unicode_string(s: &[u16]) -> UNICODE_STRING {
    let len = s.len();

    let n = if len > 0 && s[len - 1] == 0 { len - 1 } else { len };

    UNICODE_STRING {
        Length: (n * 2) as u16,
        MaximumLength: (len * 2) as u16,
        Buffer: s.as_ptr() as _,
    }
}

pub fn unicode_string_to_string(string: &UNICODE_STRING) -> String {
    let mut buf = Vec::new();

    for i in 0..(string.Length/2) {
        unsafe { buf.push(string.Buffer.add(i as _).read()) };
    }

    String::from_utf16_lossy(&buf)
}

pub fn unicode32_string_to_string(string: &UNICODE_STRING32) -> String {
    let mut buf = Vec::new();

    for i in 0..(string.Length/2) {
        unsafe { buf.push((string.Buffer as *mut u16).add(i as _).read()) };
    }

    String::from_utf16_lossy(&buf)
}

// impl<const S: usize> ToString for [UCHAR; S] {
//     fn to_string(&self) -> String {
//         unimplemented!()
//     }
// }