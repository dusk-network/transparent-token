COMPILER_VERSION=v0.2.0

all: contract

contract: setup-compiler
	@RUSTFLAGS="-C link-args=-zstack-size=65536" \
	cargo +dusk build \
	  --release \
	  --manifest-path=contract/Cargo.toml \
	  --color=always \
	  -Z build-std=core,alloc \
	  --target wasm64-unknown-unknown
	@mkdir -p target/stripped
	@find target/wasm64-unknown-unknown/release -maxdepth 1 -name "*.wasm" \
	    | xargs -I % basename % \
	    | xargs -I % ./scripts/strip.sh \
	 	          target/wasm64-unknown-unknown/release/% \
	 	          target/stripped/%

setup-compiler:
	@./scripts/setup-compiler.sh $(COMPILER_VERSION)

.PHONY: all contract setup-compiler
