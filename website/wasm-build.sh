set -e


#rustup target add wasm32-unknown-unknown
#cargo install -f wasm-bindgen-cli
#cargo install simple-http-server

cargo build --target wasm32-unknown-unknown --release

wasm-bindgen ../target/wasm32-unknown-unknown/release/website.wasm --target web --no-typescript --out-dir dist --out-name website
cp index.html dist/index.html
simple-http-server dist -c wasm,html,js --try-file dist/index.html -i --coep --coop --ip 127.0.0.1
