PROTO_DIR = protos
RUST_DIR = order-match
RUST_OUT = $(RUST_DIR)/src/generated/rpc

PROTO_FILES := $(wildcard $(PROTO_DIR)/*.proto)

all: gen-rust

gen-rust:
	@echo "Generating Rust code..."
	@mkdir -p $(RUST_OUT)
	@cd $(RUST_DIR) && RUST_BACKTRACE=1 cargo build

clean:
	rm -rf $(RUST_DIR)/src/generated/rpc/*.rs

