fn main() {
    println!("cargo:rerun-if-changed=partitions.csv");
    let output_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR is not set");
    std::fs::copy(
        "partitions.csv",
        std::path::Path::new(&output_dir).join("partitions.csv"),
    )
    .expect("copy partitions.csv to ESP-IDF output directory");
    embuild::espidf::sysenv::output();
}
