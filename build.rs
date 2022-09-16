fn main() {
    tonic_build::compile_protos("./proto/msg.proto").expect("could not compile proto files");
}
