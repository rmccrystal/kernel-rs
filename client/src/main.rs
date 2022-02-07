use log::LevelFilter;
use kernel_client::driver::Driver;
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

    let handle = kernel_client::KernelHandle::new().unwrap();
    let pid = 21992;
    let modules = handle.get_modules(pid).unwrap();
    dbg!(modules.len());
    let mut buf = [0u8; 0x100];
    let m = modules.iter().find(|m| m.name.to_lowercase() == "notepad.exe").unwrap();
    dbg!(m);
    let mut buf = [0u8; 0x100];
    unsafe {
        let dr = Driver::new().unwrap();
        let req = Request::ReadPhysical { address: 1761280, buf: buf.as_mut_ptr(), len: buf.len() };
        dbg!(&req);
        let res = dr.send_request(req).unwrap();
        dbg!(&res);
        // dbg!(buf);
    }
    handle.read_memory(pid, m.base_address, &mut buf);
    dbg!(buf);

    // unsafe {
    //     let driver = kernel_client::driver::Driver::new().unwrap();
    //     driver.ping();
    // }
    // println!("success");
}