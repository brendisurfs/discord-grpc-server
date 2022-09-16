use std::{env, path::PathBuf};

fn main() {
	let proto_file = "./proto/msg.proto";
	let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

	tonic_build::configure()
		.build_server(true)
		.out_dir("./src")
		.compile(&[proto_file], &["."])
		.unwrap_or_else(|err| panic!("protobuf compile error: {err}"));

	println!("cargo:rerun-if-changed={}", proto_file);
}
