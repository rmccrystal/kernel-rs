use crate::include::{HDC};
use winapi::shared::ntdef::HRESULT;
use crate::kernel::hook_function;
use core::mem;
use log::*;

type BitBlt_t = fn(HDC, i32, i32, i32, i32, HDC, i32, i32, u32) -> HRESULT;

pub fn init_hooks() {
    unsafe {

        // ORIGINAL_BIT_BLT = Some(mem::transmute(hook_function(0 as _, mem::transmute(hooked_bit_blt as fn(_, _, _, _, _, _, _, _, _) -> _))));
    }
}

static mut ORIGINAL_BIT_BLT: Option<BitBlt_t> = None;

pub fn hooked_bit_blt(
    hdc: HDC,     // A handle to the destination device context.
    x: i32,       // The x-coordinate, in logical units, of the upper-left corner of the destination rectangle.
    y: i32,       // The y-coordinate, in logical units, of the upper-left corner of the destination rectangle.
    cx: i32,      // The width, in logical units, of the source and destination rectangles.
    cy: i32,      // The height, in logical units, of the source and the destination rectangles.
    hdc_src: HDC, // A handle to the source device context.
    x1: i32,      // The x-coordinate, in logical units, of the upper-left corner of the source rectangle.
    y1: i32,      // The y-coordinate, in logical units, of the upper-left corner of the source rectangle.
    rop: u32      // A raster-operation code. These codes define how the color data for the source rectangle is to be combined with the color data for the destination rectangle to achieve the final color.
) -> HRESULT {
    info!("BitBlt called");

    let original = unsafe { ORIGINAL_BIT_BLT.unwrap() };
    original(hdc, x, y, cx, cy, hdc_src, x1, y1, rop)
}
