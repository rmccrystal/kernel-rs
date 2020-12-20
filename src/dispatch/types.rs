use crate::kernel::{KernelError, ModuleInfo};
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Serialize, Deserialize};

pub type Pid = u64;

#[derive(Clone, Debug)]
pub enum Data {
    // RunRequest runs the request and returns the length
    // so the caller can create a buffer for the variable
    // length data and collect it with WriteBuffer
    RunRequest {
        req: Request,
        // number of bytes that will be returned when WriteBuffer is called
        return_bytes: *mut usize
    },
    WriteBuffer {
        buffer: Vec<u8>,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    ModuleInfo(Pid)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    ModuleInfo(Vec<ModuleInfo>)
}
