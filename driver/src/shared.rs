#[derive(Eq, PartialEq)]
pub enum Dispatch {
    Request(Request),
    Response(Result<Response, KernelError>)
}

#[derive(Eq, PartialEq)]
pub enum Request {
    Ping,
    Unregister,
    ReadPhysical {
        address: u64,
        buf: *mut [u8]
    },
    WritePhysical {
        address: u64,
        buf: *const [u8]
    }
}

#[derive(Eq, PartialEq)]
pub enum Response {
    Ping,
    Unregister,
    ReadPhysical,
    WritePhysical
}

#[derive(Eq, PartialEq)]
pub enum KernelError {
    NtStatus(i32),
    MmMapIoSpace,
    InvalidRequest
}
