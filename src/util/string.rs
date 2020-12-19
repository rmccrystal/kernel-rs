use alloc::string::{String, ToString};

use winapi::shared::ntdef::UNICODE_STRING;

use crate::include::UCHAR;
use cstr_core::CString;

pub fn create_unicode_string(s: &[u16]) -> UNICODE_STRING {
    let len = s.len();

    let n = if len > 0 && s[len - 1] == 0 { len - 1 } else { len };

    UNICODE_STRING {
        Length: (n * 2) as u16,
        MaximumLength: (len * 2) as u16,
        Buffer: s.as_ptr() as _,
    }
}


// impl<const S: usize> ToString for [UCHAR; S] {
//     fn to_string(&self) -> String {
//         unimplemented!()
//     }
// }