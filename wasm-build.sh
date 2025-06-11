set -e


#rustup target add wasm32-unknown-unknown
#cargo install -f wasm-bindgen-cli
#cargo install simple-http-server

cargo build --target wasm32-unknown-unknown --package request

wasm-bindgen target/wasm32-unknown-unknown/debug/ani_list.wasm --target web --no-typescript --out-dir target/generated --out-name request --debug --keep-debug

simple-http-server . -c wasm,html,js -i --coep --coop --ip 0.0.0.0
