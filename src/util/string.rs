use alloc::string::{String};
use alloc::vec::Vec;
use log::*;

use crate::include::{UNICODE_STRING, UNICODE_STRING32, MmIsAddressValid};

impl alloc::string::ToString for UNICODE_STRING {
    fn to_string(&self) -> String {
        if unsafe { !MmIsAddressValid(self.Buffer as _) } {
            error!("Attempted to convert an invalid UNICODE_STRING (buffer: {:p})", self.Buffer);
            return "".to_string()
        }
        let mut buf = Vec::new();

        for i in 0..(self.Length/2) {
            unsafe { buf.push(self.Buffer.add(i as _).read()) };
        }

        String::from_utf16_lossy(&buf)
    }
}

impl alloc::string::ToString for UNICODE_STRING32 {
    fn to_string(&self) -> String {
        if unsafe { !MmIsAddressValid(self.Buffer as _) } {
            panic!("Attempted to convert an invalid UNICODE_STRING")
        }
        let mut buf = Vec::new();

        for i in 0..(self.Length/2) {
            unsafe { buf.push((self.Buffer as *mut u16).add(i as _).read()) };
        }

        String::from_utf16_lossy(&buf)
    }
}


pub fn to_unicode_string(string: &str) -> UNICODE_STRING {
    create_unicode_string(&string.encode_utf16().collect::<Vec<_>>())
}

fn create_unicode_string(s: &[u16]) -> UNICODE_STRING {
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


