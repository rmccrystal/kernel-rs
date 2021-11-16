use log::LevelFilter;

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

    kernel_client::KernelHandle::new().unwrap();
    // unsafe {
    //     let driver = kernel_client::driver::Driver::new().unwrap();
    //     driver.ping();
    // }
    // println!("success");
}