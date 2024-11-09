#cargo install -f wasm-bindgen-cli
#cargo install simple-http-server

cargo build --target wasm32-unknown-unknown --example counter

wasm-bindgen target/wasm32-unknown-unknown/debug/examples/counter.wasm --target web --no-typescript --out-dir target/generated --out-name counter

simple-http-server . -c wasm,html,js -i --coep --coop --ip 127.0.0.1
