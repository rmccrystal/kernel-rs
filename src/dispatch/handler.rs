use crate::dispatch::types::{Request, Response};
use crate::kernel::{KernelError, Process};

impl Request {
    pub fn handle(&self) -> Result<Response, KernelError> {
        Ok(match self {
            Request::ModuleInfo(pid) => Response::ModuleInfo(Process::by_id(*pid)?.get_modules_64()?),
            Request::Ping => Response::Pong
        })
    }
}
