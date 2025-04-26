## Web

```
# one time
cargo install --locked trunk

# debug (http://127.0.0.1:8080)
trunk serve 

# release (/dist folder)
trunk build --release
```

## Windows

```
# debug
cargo run

# release
cargo build --release
```

## Android

```
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