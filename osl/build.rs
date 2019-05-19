use cmake;
use std::collections::HashMap;
use std::path::Path;

pub fn main() {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("build-settings").required(false))
        .unwrap();
    settings.merge(config::Environment::new()).ok();

    let settings_map = settings
        .try_into::<HashMap<String, String>>()
        .unwrap_or(HashMap::new());

    let osl_root = settings_map.get("osl_root").expect("OSL_ROOT is not set");
    let oiio_root = settings_map.get("oiio_root").expect("OIIO_ROOT is not set");    
    let openexr_root = settings_map.get("openexr_root").expect("OPENEXR_ROOT is not set");    

    let dst_osl_capi = cmake::Config::new("osl_capi")
        .define("OSL_ROOT", &osl_root)
        .define("OIIO_ROOT", &oiio_root)
        .define("OPENEXR_ROOT", &openexr_root)
        .always_configure(false)
        .build();

    println!("cargo:rustc-link-search=native={}", dst_osl_capi.display());
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&osl_root).join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&oiio_root).join("lib").display()
    );

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");

    println!("cargo:rustc-link-lib=static=osl_capi");
    println!("cargo:rustc-link-lib=dylib=oslexec");
    // println!("cargo:rustc-link-lib=dylib=oslcomp");
    println!("cargo:rustc-link-lib=dylib=OpenImageIO");
}