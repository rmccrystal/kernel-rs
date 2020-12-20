use crate::kernel::{KernelError, ModuleInfo};
use alloc::string::String;
use alloc::vec::Vec;

pub type Pid = u64;

#[derive(Clone)]
pub enum Data {
    Request(Request),
    Response(Result<Response, KernelError>)
}

#[derive(Clone)]
pub enum Request {
    ModuleInfo(Pid)
}

#[derive(Clone)]
pub enum Response {
    ModuleInfo(Vec<ModuleInfo>)
}
