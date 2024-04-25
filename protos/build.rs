fn main() {
  println!("cargo:rerun-if-changed=proto");
  tonic_build::configure()
    .out_dir("src/pb")
    .protoc_arg("--experimental_allow_proto3_optional")
    .compile(
      &[
        "proto/sf/substreams/rpc/v2/service.proto",
        "proto/sf/substreams/v1/package.proto",
        "proto/sui.proto",
        "proto/sui_coin.proto",
      ],
      &["proto"],
    )
    .expect("Failed to compile proto(s)");
}
