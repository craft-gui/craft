set -e


#rustup target add wasm32-unknown-unknown
#cargo install -f wasm-bindgen-cli
#cargo install simple-http-server

cargo build --target wasm32-unknown-unknown --package counter_retained

wasm-bindgen target/wasm32-unknown-unknown/debug/counter_retained.wasm --target web --no-typescript --out-dir target/generated --out-name counter_retained --debug --keep-debug

simple-http-server . -c wasm,html,js -i --coep --coop --ip 0.0.0.0
