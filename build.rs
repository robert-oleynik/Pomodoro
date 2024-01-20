fn main() {
    println!("cargo:rerun-if-changed=resources.gresource.xml");
    println!("cargo:rerun-if-changed=src");
    glib_build_tools::compile_resources(
        &["./src"],
        "./resources.gresource.xml",
        "resources.gresource",
    )
}
