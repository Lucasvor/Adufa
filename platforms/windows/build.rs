fn main() {
    println!("cargo:rerun-if-changed=assets/adufa.rc");
    println!("cargo:rerun-if-changed=assets/adufa.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/adufa.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("Adufa's Windows icon resource should compile");
    }
}
