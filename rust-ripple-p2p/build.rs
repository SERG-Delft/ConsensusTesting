use protobuf_codegen_pure;

fn main() {
    protobuf_codegen_pure::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["protos/ripple.proto"])
        .include("protos")
        .run()
        .expect("protoc");
}

