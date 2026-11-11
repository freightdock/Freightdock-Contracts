# freightdock-proto

The contracts of a Freightdock cluster: the protobuf messages and gRPC services the manager-agent, the worker-agent,
the network-agent and the UI's server speak, plus what they must agree on that is not a message (the cluster join
token's format, the name on a manager's certificate, the default port).

```
proto/freightdock/cluster/v1/worker.proto   enrolment and the worker session: hello, heartbeats, commands, reports
proto/freightdock/cluster/v1/manager.proto  manager to manager: Raft RPCs (opaque JSON) and forwarded proposals
src/token.rs                                FDCT1-<CA sha256>-<secret>, the cluster join token
src/lib.rs                                  the generated code (module v1) and the shared names
```

Rules:

- A field is only ever added, under a new number. A number is never reused or renumbered.
- A breaking change is a new package (`freightdock.cluster.v2`), served beside v1 for as long as old components run.
- Components depend on this crate and never on each other's internals.

Building needs no `protoc`: `build.rs` compiles the `.proto` files with protox.

```sh
cargo test
```

GPL-3.0-only, as Freightdock.
