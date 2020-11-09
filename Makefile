all: build

build:
	cargo build -- --cfg=embedded_shaders

run:
	cargo run -- --cfg=embedded_shaders

c:
	cargo check
