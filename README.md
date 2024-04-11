## About
An example of a clean wasm project
- Build with [wasm-pack](https://github.com/rustwasm/wasm-pack) without a bundler
- Serve it locally with [miniserve](https://github.com/svenstaro/miniserve)
- All necessary tools are installed as needed within the project
- Reuse the `target` directory for crates and tools
- No third-party build utilities, only Cargo

## Build
To build the wasm crate run:
```sh
cargo xtask build
```

Then start a local server with:
```sh
cargo xtask serve
```

By default, the build script will install the necessary tools if they are not already installed. To prevent this, pass the `--no-install` flag:
```sh
cargo xtask --no-install build
```