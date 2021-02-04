use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::kernel::{KernelError, ModuleInfo};

pub type Pid = u64;

#[derive(Debug)]
pub enum Data<'a> {
    // RunRequest runs the request and returns the length
    // so the caller can create a buffer for the variable
    // length data and collect it with WriteBuffer
    RunRequest {
        req: Request<'a>,
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Request<'a> {
    Ping,
    ModuleInfo(Pid),
    GetPebAddress(Pid),
    GetProcessBitness(Pid),
    ReadMemory {
        pid: Pid,
        address: u64,
        #[serde(skip_serializing, skip_deserializing)]
        buf: &'a mut [u8],
    },
    WriteMemory {
        pid: Pid,
        address: u64,
        // A pointer to a slice
        #[serde(skip_serializing, skip_deserializing)]
        buf: &'a [u8]
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Pong,
    ModuleInfo(Vec<ModuleInfo>),
    PebAddress(u64),
    ProcessBitness(u16),
    ReadMemory,
    WriteMemory,
}
