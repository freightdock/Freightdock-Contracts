fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = ["proto/freightdock/cluster/v1/worker.proto", "proto/freightdock/cluster/v1/manager.proto"];
    for p in protos {
        println!("cargo:rerun-if-changed={p}");
    }
    let fds = protox::compile(protos, ["proto"])?;
    tonic_prost_build::configure().build_client(true).build_server(true).compile_fds(fds)?;
    Ok(())
}
