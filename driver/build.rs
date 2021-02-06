use failure::{format_err, Error};
use std::{
    env::var,
    path::{Path, PathBuf},
};
use winreg::{enums::*, RegKey};
use bindgen::EnumVariation;

/// Returns the path to the `Windows Kits` directory. It's by default at
/// `C:\Program Files (x86)\Windows Kits\10`.
fn get_windows_kits_dir() -> Result<PathBuf, Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
    let dir: String = hklm.open_subkey(key)?.get_value("KitsRoot10")?;

    Ok(dir.into())
}

/// Returns the path to the kernel mode libraries. The path may look like this:
/// `C:\Program Files (x86)\Windows Kits\10\lib\10.0.18362.0\km`.
fn get_km_dir(windows_kits_dir: &PathBuf) -> Result<PathBuf, Error> {
    let readdir = Path::new(windows_kits_dir).join("lib").read_dir()?;

    let max_libdir = readdir
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path())
        .filter(|dir| {
            dir.components()
                .last()
                .and_then(|c| c.as_os_str().to_str())
                .map(|c| c.starts_with("10.") && dir.join("km").is_dir())
                .unwrap_or(false)
        })
        .max()
        .ok_or_else(|| format_err!("Can not find a valid km dir in `{:?}`", windows_kits_dir))?;

    Ok(max_libdir.join("km"))
}

/// Returns the path to the user mode libraries. The path may look like this:
/// `C:\Program Files (x86)\Windows Kits\10\lib\10.0.18362.0\um`.
fn get_um_dir(windows_kits_dir: &PathBuf) -> Result<PathBuf, Error> {
    let readdir = Path::new(windows_kits_dir).join("lib").read_dir()?;

    let max_libdir = readdir
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path())
        .filter(|dir| {
            dir.components()
                .last()
                .and_then(|c| c.as_os_str().to_str())
                .map(|c| c.starts_with("10.") && dir.join("um").is_dir())
                .unwrap_or(false)
        })
        .max()
        .ok_or_else(|| format_err!("Can not find a valid um dir in `{:?}`", windows_kits_dir))?;

    Ok(max_libdir.join("um"))
}

fn get_km_include_dir(windows_kits_dir: &PathBuf) -> Result<PathBuf, Error> {
    let readdir = Path::new(windows_kits_dir).join("include").read_dir()?;
    let max_libdir = readdir
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path())
        .filter(|dir| {
            dir.components()
                .last()
                .and_then(|c| c.as_os_str().to_str())
                .map(|c| c.starts_with("10.") && dir.join("km").is_dir())
                .unwrap_or(false)
        })
        .max()
        .ok_or_else(|| format_err!("Can not find a valid km dir in `{:?}`", windows_kits_dir))?;
    Ok(max_libdir.join("km"))
}

fn internal_link_search() {
    let windows_kits_dir = get_windows_kits_dir().unwrap();
    let km_dir = get_km_dir(&windows_kits_dir).unwrap();
    let um_dir = get_um_dir(&windows_kits_dir).unwrap();
    let target = var("TARGET").unwrap();

    let arch = if target.contains("x86_64") {
        "x64"
    } else if target.contains("i686") {
        "x86"
    } else {
        panic!("Only support x86_64 and i686!");
    };

    let km_lib_dir = km_dir.join(arch);
    println!("cargo:rustc-link-search=native={}", km_lib_dir.to_str().unwrap());
    let um_lib_dir = um_dir.join(arch);
    println!("cargo:rustc-link-search=native={}", um_lib_dir.to_str().unwrap());
}

fn extra_link_search() {}

fn main() {
    let km_include_dir = get_km_include_dir(&get_windows_kits_dir().unwrap()).unwrap();
    let km_include_dir = km_include_dir.to_str().unwrap();

    println!("{}", km_include_dir);

    println!("cargo:rerun-if-changed=build.rs");
    let bindings = bindgen::Builder::default()
        .header("src/include/bindings.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg(format!("-I{}", km_include_dir))

        .whitelist_function("MmCopyVirtualMemory")
        .whitelist_function("ZwQuerySystemInformation")
        .whitelist_function("RtlFindExportedRoutineByName")
        .whitelist_type("PRTL_PROCESS_MODULES")
        .whitelist_type("*SystemModuleInformation")

        .whitelist_function("RtlSecureZeroMemory")

        .whitelist_function("IoAllocateMdl")
        .whitelist_function("MmProbeAndLockPages")
        .whitelist_function("MmMapLockedPagesSpecifyCache")
        .whitelist_function("MmProtectMdlSystemAddress")
        .whitelist_function("MmUnmapLockedPages")
        .whitelist_function("MmUnlockPages")
        .whitelist_function("IoFreeMdl")

        .whitelist_function("ExAllocatePoolWithTag")
        .whitelist_function("ExFreePoolWithTag")

        .whitelist_function("PsGetProcessPeb")
        .whitelist_function("PsGetProcessWow64Process")
        .whitelist_type("PPEB32")
        .whitelist_type("PPEB_LDR_DATA32")
        .whitelist_type("PLDR_DATA_TABLE_ENTRY32")
        .whitelist_type("LDR_DATA_TABLE_ENTRY32")
        .whitelist_function("KeStackAttachProcess")
        .whitelist_function("KeUnstackDetachProcess")
        .whitelist_function("IoGetCurrentProcess")
        .whitelist_type("_LDR_DATA_TABLE_ENTRY")

        .whitelist_type("_SYSTEM_INFORMATION_CLASS")
        .whitelist_type("SYSTEM_PROCESS_INFO")
        .whitelist_type("HDC")

        .whitelist_function("NtGdiBitBlt")

        .whitelist_type("PDRIVER_OBJECT")

        .ctypes_prefix("crate::include::raw")
        .derive_debug(true)
        .default_enum_style(EnumVariation::Rust {non_exhaustive: false})
        .use_core()
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file("src/include/bindings.rs")
        .expect("Couldn't write bindings!");

    if var(format!("CARGO_FEATURE_{}", "extra_link_search".to_uppercase())).is_ok() {
        extra_link_search()
    } else {
        internal_link_search()
    }

    cc::Build::new()
        .cpp(true)
        .file("src/interop/clear.cpp")
        // .include(km_include_dir)
        .include("C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.19041.0\\km")
        .compile("interop");

    println!("cargo:rerun-if-changed=src/interop/clear.cpp");
    println!("cargo:rerun-if-changed=src/interop/clear.h");

    bindgen::Builder::default()
        .header("src/interop/include.h")

        .clang_arg("-x")
        .clang_arg("c++")

        .ctypes_prefix("crate::include::raw")
        .derive_debug(true)
        .default_enum_style(EnumVariation::Rust {non_exhaustive: false})
        .use_core()
        .layout_tests(false)
        .generate()
        .expect("Unable to generate interop bindings")

        .write_to_file("src/interop/bindings.rs")
        .expect("Could not write interop bindings");
}
