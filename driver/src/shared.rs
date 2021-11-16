pub const KEY: &str = "TestKey1";

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Dispatch {
    pub handled: bool,
    pub data: Data,
}

#[derive(Eq, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Data {
    Request(Request),
    Response(Result<Response, KernelError>),
}

#[derive(Eq, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Request {
    Ping,
    Unregister,
    ReadPhysical {
        address: u64,
        buf: *mut [u8],
    },
    WritePhysical {
        address: u64,
        buf: *const [u8],
    },
}

#[derive(Eq, PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Response {
    Ping,
    Unregister,
    ReadPhysical,
    WritePhysical,
}

#[derive(Eq, PartialEq, Debug)]
pub enum KernelError {
    NtStatus(i32),
    MmMapIoSpace { address: u64, len: usize },
    PartialCopy { address: u64, len: usize, read: usize },
    InvalidRequest,
}
