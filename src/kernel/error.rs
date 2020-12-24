use alloc::string::{String, ToString};

use serde::{Deserialize, Serialize};
use winapi::_core::fmt::Formatter;
use winapi::shared::ntdef::{NT_SUCCESS, NTSTATUS};

#[derive(Clone, Serialize, Deserialize)]
pub enum KernelError {
    Message(String),
    Status(NTSTATUS)
}

impl core::fmt::Debug for KernelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let _ = match &self {
            KernelError::Message(msg) => write!(f, "{}", msg),
            KernelError::Status(status) => match error_code_to_message(*status as _) {
                Some(error) => write!(f, "{} ({:X})", error, status),
                None => write!(f, "{:X}", status)
            }
        };
        Ok(())
    }
}

impl From<&str> for KernelError {
    fn from(text: &str) -> Self {
        Self::Message(text.to_string())
    }
}

impl KernelError {
    pub fn text(text: &str) -> Self {
        Self::Message(text.to_string())
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
fn error_code_to_message(_code: u32) -> Option<String> {
    None
}
