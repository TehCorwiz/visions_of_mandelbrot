serve package: (build package)
    miniserve --index index.html ./target/{{package}}/

build package:
    mkdir -p ./target/{{package}}/
    cp ./static/index.html ./web_target/
    cargo build --release --package {{package}} --target wasm32-unknown-unknown --features web
    wasm-bindgen --target web --no-typescript --out-dir ./web_target/ ./target/wasm32-unknown-unknown/release/{{package}}.wasm

clean package:
    rm -rf ./target/{{package}}/
