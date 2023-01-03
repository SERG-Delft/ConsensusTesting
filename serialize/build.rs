use std::{env, path::Path};

use protobuf_codegen_pure::Customize;

fn main() {
    println!("cargo:rerun-if-changed=protos/ripple.proto");
    protobuf_codegen_pure::Codegen::new()
        .out_dir(Path::new(&env::var_os("OUT_DIR").unwrap()))
        .include("protos")
        .inputs(["protos/ripple.proto"])
        .customize(Customize {
            serde_derive: Some(true),
            gen_mod_rs: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc");
}
