fn main() -> Result<(), Box<dyn std::error::Error>> {
  tonic_build::compile_protos("proto/runner.proto")?;
  tonic_build::compile_protos("proto/controller.proto")?;
  // tonic_build::configure()
  //       .build_client(false)
  //       .out_dir("src/")
  //       .compile(&["proto/runner.proto"], &["path"])
  //       .expect("failed to compile protos");
  Ok(())
}