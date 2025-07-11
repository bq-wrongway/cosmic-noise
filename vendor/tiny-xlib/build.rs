// MIT/Apache2/Zlib License

//! Finds the path of Xlib and Xlib-XCB.
//!
//! Normally, we should be able to link directly to these packages. However, on some atypical
//! Linux configurations (like NixOS), they might be in other directories. As far as I know
//! `pkg-config` is the only blessed way to find and link to these libraries. So unfortunately,
//! we have to have a heavy build script and build-time dependency.

use std::io::{self, prelude::*};
use std::path::PathBuf;
use std::{env, fs};

fn find_link_deps(deps: &[(&str, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    for (_name, pkg_config_name) in deps {
        pkg_config::Config::new().probe(pkg_config_name)?;
    }

    Ok(())
}

fn find_dlopen_dirs(deps: &[(&str, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    let mut libdir_file = {
        let path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("libdir.rs");
        let libdir_file = fs::File::create(path)?;
        io::BufWriter::new(libdir_file)
    };

    for (name, pkg_config_name) in deps {
        let dir = match pkg_config::get_variable(pkg_config_name, "libdir") {
            Ok(libdir) => format!("Some(\"{}\")", libdir),
            Err(err) => {
                println!(
                    "cargo:warning=failed to get libdir for library {}: {}",
                    pkg_config_name, err
                );
                "None".to_string()
            }
        };

        writeln!(
            libdir_file,
            "const {}_LIBDIR: Option<&str> = {};",
            name, dir
        )?;
    }

    libdir_file.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const DEPENDENCIES: &[(&str, &str)] = &[("XLIB", "x11"), ("XLIB_XCB", "x11-xcb")];

    if env::var_os("CARGO_FEATURE_DLOPEN").is_some() {
        find_dlopen_dirs(DEPENDENCIES)?;
    } else {
        find_link_deps(DEPENDENCIES)?;
    }

    Ok(())
}
