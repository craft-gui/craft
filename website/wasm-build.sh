set -e


#rustup target add wasm32-unknown-unknown
#cargo install -f wasm-bindgen-cli
#cargo install simple-http-server

cargo build --package website --target wasm32-unknown-unknown --profile web-release

wasm-bindgen ../target/wasm32-unknown-unknown/web-release/website.wasm --target web --no-typescript --out-dir dist --out-name website

# wasm-opt is optional, but cuts several more megabytes from production builds.
if command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt -Oz dist/website_bg.wasm -o dist/website_bg.opt.wasm
    mv dist/website_bg.opt.wasm dist/website_bg.wasm
fi

cp index.html dist/index.html
simple-http-server dist -c wasm,html,js --try-file dist/index.html -i --coep --coop --nocache --ip 0.0.0.0
