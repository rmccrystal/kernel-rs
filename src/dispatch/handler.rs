use crate::dispatch::types::{Request, Response};
use crate::kernel::{KernelError, Process};

impl Request {
    pub fn handle(&self) -> Result<Response, KernelError> {
        Ok(match self {
            Request::ModuleInfo(pid) => Response::ModuleInfo(Process::by_id(*pid)?.get_modules()?),
            Request::Ping => Response::Pong,
            Request::GetPebAddress(pid) => Response::PebAddress(Process::by_id(*pid)?.get_peb() as _),
            Request::ReadMemory { address, size, pid } => Response::ReadMemory(Process::by_id(*pid)?.read_memory(*address, *size)?),
            Request::WriteMemory { address, buf, pid } => {
                Process::by_id(*pid)?.write_memory(*address, &buf)?;
                Response::WriteMemory
            }
        })
    }
}
