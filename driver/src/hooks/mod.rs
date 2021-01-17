#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::kernel::{get_process_list, Process, find_kernel_module, get_kernel_module_export, Result, hook_function, get_kernel_modules};
use log::*;
use crate::include::{BOOLEAN, HDC};

pub unsafe fn init_hooks() -> Result<()> {
    let processes = get_process_list()?;
    let modules = get_kernel_modules()?;

    // hook NtGdiBitBlt
    {
        // we have to attach to winlogon.exe to put win32k in our address space.
        let _attach = Process::by_id(processes.iter()
            .find(|p| p.name == "winlogon.exe")
            .ok_or("could not find winlogon.exe")?
            .pid)?
            .attach();

        debug!("Finding win32kfull.sys address");
        let win32k = find_kernel_module(&modules, "win32kfull.sys").ok_or("could not find win32k.sys")?;
        debug!("Found win32kfull.sys: {:p}", win32k);
        let nt_gdi_stretch_blt = get_kernel_module_export(win32k, "NtGdiStretchBlt").ok_or("could not find NtGdiStretchBlt")?;
        dbg!(nt_gdi_stretch_blt);

        ORIGINAL_NT_GDI_STRETCH_BLT = Some(core::mem::transmute(
            hook_function(nt_gdi_stretch_blt, core::mem::transmute(hooked_nt_gdi_stretch_blt as NtGdiStretchBlt_t))
        ));
        dbg!(ORIGINAL_NT_GDI_STRETCH_BLT);
    }

    Ok(())
}

type NtGdiStretchBlt_t = unsafe fn(
    hdcDst: HDC,
    xDst: i32,
    yDst: i32,
    cxDst: i32,
    cyDst: i32,
    hdcSrc: HDC,
    xSrc: i32,
    ySrc: i32,
    cxSrc: i32,
    cySrc: i32,
    dwRop: u32,
    dwBackColor: u32,
) -> BOOLEAN;

static mut ORIGINAL_NT_GDI_STRETCH_BLT: Option<NtGdiStretchBlt_t> = None;

unsafe fn hooked_nt_gdi_stretch_blt(
    hdcDst: HDC,
    xDst: i32,
    yDst: i32,
    cxDst: i32,
    cyDst: i32,
    hdcSrc: HDC,
    xSrc: i32,
    ySrc: i32,
    cxSrc: i32,
    cySrc: i32,
    dwRop: u32,
    dwBackColor: u32,
) -> BOOLEAN {
    dbg!(hdcDst,
         xDst,
         yDst,
         cxDst,
         cyDst,
         hdcSrc,
         xSrc,
         ySrc,
         cxSrc,
         cySrc,
         dwRop,
         dwBackColor);

    let original = ORIGINAL_NT_GDI_STRETCH_BLT.unwrap();
    original(hdcDst,
             xDst,
             yDst,
             cxDst,
             cyDst,
             hdcSrc,
             xSrc,
             ySrc,
             cxSrc,
             cySrc,
             dwRop,
             dwBackColor)
}

