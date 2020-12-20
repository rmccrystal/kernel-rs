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
        response: *mut RunRequestResponse,
    },
    WriteBuffer {
        buffer: Vec<u8>,
    },
}

// Returned when RunRequest is returned
pub enum RunRequestResponse {
    Null,
    // the caller should allocate a buffer and call again
    AllocBuffer(usize),
    // there is no need to allocate and a response can be immediately sent
    Response(Result<Response, KernelError>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Ping,
    ModuleInfo(Pid),
    GetPebAddress(Pid),
    ReadMemory {
        pid: Pid,
        address: u64,
        size: u64,
    },
    WriteMemory {
        pid: Pid,
        address: u64,
        buf: Vec<u8>
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Pong,
    ModuleInfo(Vec<ModuleInfo>),
    PebAddress(u64),
    ReadMemory(Vec<u8>),
    WriteMemory,
}
