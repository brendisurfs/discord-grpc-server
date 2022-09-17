fn main() {
    let proto_file = "./proto/msg.proto";

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src")
        .compile(&[proto_file], &["."])
        .unwrap_or_else(|err| panic!("protobuf compile error: {err}"));

    println!("cargo:rerun-if-changed={}", proto_file);
}
