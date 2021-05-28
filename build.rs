use std::{env, path::PathBuf};


fn main() {
    println!("cargo:rerun-if-changed=pico-binding,pico-sdk");

    let bindings = bindgen::Builder::default()
        .header("pico-binding/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .prepend_enum_name(false)
        .layout_tests(false)
        .disable_untagged_union()
        .use_core()
        .detect_include_paths(true)
        .ctypes_prefix("crate::ctypes")
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("binding.rs"))
        .unwrap();
}
