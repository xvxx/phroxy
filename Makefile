target/release/phroxy: src/*.rs
	cargo build --release
	strip $@
