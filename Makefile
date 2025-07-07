BINARY_NAME = about-this-linux
INSTALL_DIR = /usr/local/bin
CONFIG_DIR = ~/.local/share/about-this-linux

.PHONY: build release install uninstall clean run configure help

help:
	@echo "Available targets:"
	@echo "  build     - Build debug version"
	@echo "  release   - Build release version"
	@echo "  install   - Install the application to $(INSTALL_DIR)"
	@echo "  uninstall - Remove the application from $(INSTALL_DIR)"
	@echo "  clean     - Clean build artifacts"
	@echo "  run       - Run the application in debug mode"
	@echo "  configure - Run the configuration wizard"

build:
	cargo build

release:
	cargo build --release

install: release
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/
	sudo chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Installation complete. Run '$(BINARY_NAME)' to start the application."

uninstall:
	sudo rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Uninstallation complete."

clean:
	cargo clean
	rm -rf target/

run: build
	cargo run

configure: build
	cargo run -- --configure

# Development targets
fmt:
	cargo fmt

clippy:
	cargo clippy

test:
	cargo test

check: fmt clippy test
