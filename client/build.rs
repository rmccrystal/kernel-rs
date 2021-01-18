use failure::{format_err, Error};
use std::{
    env::var,
    path::{Path, PathBuf},
};
use winreg::{enums::*, RegKey};

/// Returns the path to the `Windows Kits` directory. It's by default at
/// `C:\Program Files (x86)\Windows Kits\10`.
fn get_windows_kits_dir() -> Result<PathBuf, Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
    let dir: String = hklm.open_subkey(key)?.get_value("KitsRoot10")?;

    Ok(dir.into())
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

fn internal_link_search() {
    let windows_kits_dir = get_windows_kits_dir().unwrap();
    let um_dir = get_um_dir(&windows_kits_dir).unwrap();
    let target = var("TARGET").unwrap();

    let arch = if target.contains("x86_64") {
        "x64"
    } else if target.contains("i686") {
        "x86"
    } else {
        panic!("Only support x86_64 and i686!");
    };

    let um_lib_dir = um_dir.join(arch);
    println!("cargo:rustc-link-search=native={}", um_lib_dir.to_str().unwrap());
}

fn main() {
    internal_link_search();

    let windows_kits_path = get_windows_kits_dir().unwrap();
    let um_dir = get_um_dir(&windows_kits_path).unwrap().join("x64");
    let deps_dir = PathBuf::new()
        .join(var("CARGO_MANIFEST_DIR").unwrap())
        .join("deps");

    println!("cargo:rustc-link-search=native={}", deps_dir.to_str().unwrap());

    cc::Build::new()
    // cxx_build::bridge("src/kdmapper.rs")
        .file("kdmapper/intel_driver.cpp")
        .file("kdmapper/kdmapper.cpp")
        // .file("kdmapper/main.cpp")
        .file("kdmapper/portable_executable.cpp")
        .file("kdmapper/service.cpp")
        .file("kdmapper/utils.cpp")
        .file("kdmapper/exports.cpp")

        .include("C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\MSVC\\14.28.29333\\atlmfc\\include")

        .object(um_dir.join("Version.lib"))

        .debug(false)

        .flag("/std:c++17")

        .include("kdmapper")
        .compile("kdmapper");

    println!("cargo:rerun-if-changed=src/kdmapper.rs");
    println!("cargo:rerun-if-changed=kdmapper");

    println!("cargo:rerun-if-changed=build.rs");
}