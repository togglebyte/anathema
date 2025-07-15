fn main() {
    // Set the environment variable during build
    println!("cargo:rustc-env=AWS_LC_SYS_PREBUILT_NASM=1");

    // Alternative: You can also use this method if you need to set it for the build process itself
    // unsafe {
    //     std::env::set_var("AWS_LC_SYS_PREBUILT_NASM", "1");
    // }
}
