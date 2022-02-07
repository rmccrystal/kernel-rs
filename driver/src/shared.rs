pub const KEY: &str = "TestKey1";

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Dispatch {
    pub handled: bool,
    pub data: Data,
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Data {
    Request(Request),
    Response(Result<Response, KernelError>),
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Request {
    Ping,
    Unregister,
    ReadPhysical {
        address: u64,
        buf: *mut u8,
        len: usize,
    },
    WritePhysical {
        address: u64,
        buf: *const u8,
        len: usize,
    },
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Response {
    Ping,
    Unregister,
    ReadPhysical,
    WritePhysical,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum KernelError {
    NtStatus(i32),
    MmMapIoSpace { address: u64, len: usize },
    PartialCopy { address: u64, len: usize, read: usize },
    InvalidRequest,
}
