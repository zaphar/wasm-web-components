build:
	cargo build;

test:
	cd wasm-web-component; wasm-pack test --headless --firefox
