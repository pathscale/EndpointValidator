# EndpointValidator

An interactive terminal UI for calling a running endpoint-libs WebSocket service's
endpoints by hand. It lives in [`endpoint-validator/`](endpoint-validator/README.md), with
how to run it, the `config.toml` of preset parameters, and how values go on the wire.

`services.json` comes from [EndpointGen](https://github.com/pathscale/EndpointGen).
`ws-load-test/` is the load benchmark, held on its own dependencies for the reason in its
`Cargo.toml`.
