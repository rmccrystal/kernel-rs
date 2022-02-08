#![feature(core_intrinsics)]
use std::time::Duration;
use log::LevelFilter;
use kernel_client::driver::DriverHandle;
use kernel_client::KernelHandle;
use kernel_client::shared::Request;

fn main() {
    unsafe {
        env_logger::init();
        // let driver = kernel_client::driver::Driver::new().unwrap();
        // let mut buf = vec![0u8; 0x100_000];
        // driver.read_physical_chunked(0, &mut buf[0..0x10_000]);
        // dbg!(&buf);
        // let mut buf = vec![0u8; 0x1000000];
        // driver.read_physical_chunked(0, &mut buf);
        // dbg!(&buf);
        // let mut buf = vec!(0u8; 0x1000);
        // for i in (0..0x1000000).step_by(0x1000) {
        //     let result = driver.read_physical(i, &mut buf);
        //     if let Err(e) = result {
        //         log::error!("Could not read {:#X}-{:#X}: {:?}", i, i + 0x1000, e);
        //     }
        //     log::info!("Read {:#X}-{:#X}", i, i + 0x1000);
        // }
        // dbg!(buf);
    }

    let h = KernelHandle::new().unwrap();

    // unsafe {
    //     let dr = Driver::new().unwrap();
    //     let mut buf = [0u8; 0x10];
    //     let req = Request::ReadPhysical { buf: buf.as_mut_ptr(), len: buf.len(), address: 1761280 };
    //     dbg!(&req);
    //     let res = dr.send_request(req);
    //     dbg!(&res);
    //     // dbg!(buf);
    // }
    // return;


    // unsafe {
    //     let driver = kernel_client::driver::Driver::new().unwrap();
    //     driver.ping();
    // }
    // println!("success");
}