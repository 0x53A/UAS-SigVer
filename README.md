## Web

```sh
# one time
cargo install wasm-pack

# build debug
wasm-pack build --target web --dev

# serve locally (or just open the html file directly)
python -m http.server 8080

# build release
wasm-pack build --target web --release
```

## Windows

```sh
# debug
cargo run

# release
cargo build --release
```

## Android

```sh
# one time
cargo install --git https://github.com/tauri-apps/cargo-mobile2
# once after clone (or clean) to create the /gen/ folder
cargo mobile init

# debug
cargo android run

# release (one of these)
cargo android apk build --release # universal apk for armv7, aarch64, i686, x86-64
cargo android apk build --release aarch64 # only arm64 (newer devices)
cargo android apk build --release --split-per-abi # one apk per platform
```