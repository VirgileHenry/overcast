# build the overcast client
cd ../overcast-client
cargo build --release --target wasm32-unknown-unknown
# go back and copy the wasm
cd ../overcast-web
cp ../target/wasm32-unknown-unknown/release/overcast-client.wasm ./wasm/
# generate the js to bind wasm
wasm-bindgen --no-typescript --target web --out-dir ./js/ --out-name "overcast" ./wasm/overcast-client.wasm