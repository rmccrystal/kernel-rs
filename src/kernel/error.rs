use alloc::string::String;

use cstr_core::{CStr, CString};
use winapi::_core::fmt::Formatter;
use winapi::shared::ntdef::{LANG_NEUTRAL, MAKELANGID, NT_SUCCESS, NTSTATUS, NULL, SUBLANG_DEFAULT};
use winapi::um::winbase::{FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS, FormatMessageA};

#[derive(Copy, Clone)]
pub enum KernelError {
    Message(&'static str),
    Status(NTSTATUS)
}

impl core::fmt::Debug for KernelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self {
            KernelError::Message(msg) => write!(f, "{}", msg),
            KernelError::Status(status) => match error_code_to_message(*status as _) {
                Some(error) => write!(f, "{} ({:X})", error, status),
                None => write!(f, "{:X}", status)
            }
        };
        Ok(())
    }
}

pub trait ToKernelResult {
    fn to_kernel_result(self) -> Result<(), KernelError>;
}

impl ToKernelResult for NTSTATUS {
    fn to_kernel_result(self) -> Result<(), KernelError> {
        if !NT_SUCCESS(self) {
            Err(KernelError::Status(self))
        } else {
            Ok(())
        }
    }
}

/// Converts a Windows error code to its corresponding message.
/// If there is no message associated with the code, this will return None
fn error_code_to_message(code: u32) -> Option<String> {
    return None;

    let mut message_buf: [i8; 512] = [0; 512];

    // Get the error string by the code
    let buf_len = unsafe {
        FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            code,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
            message_buf.as_mut_ptr(),
            512,
            NULL as *mut *mut i8,
        )
    };

    // there is no message for the error
    if buf_len == 0 {
        return None;
    }

    let mut error_string = unsafe { CString::from_raw(message_buf.clone().as_mut_ptr()).into_string().unwrap() };

    // Remove \n from end of string
    error_string.pop();
    // Remove \r from end of string
    error_string.pop();

    Some(error_string)
}
