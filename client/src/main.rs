use log::LevelFilter;

fn main() {
    env_logger::init();
    log::set_max_level(LevelFilter::Trace);
    unsafe {
        let driver = kernel_client::driver::Driver::new().unwrap();
        driver.ping();
    }
    println!("success");
}