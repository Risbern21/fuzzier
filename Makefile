.PHONY: all run build clean

all: build

build:
	@echo "Building the binary"
	cargo build

run:
	@echo "Running the application"
	cargo run

clean:
	@echo "Cleaning up build artifacts"
	rm -rf target
