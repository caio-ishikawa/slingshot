# Makefile for building, installing, and adding Rust project to PATH

# Configuration

INSTALL_DIR := /usr/local/bin

# Commands
CARGO := cargo
INSTALL := install -m 755
LN := ln -sf

all: install

build:
	$(CARGO) build --release

install: build
	@echo "Installing slingshot to $(INSTALL_DIR)"
	@$(INSTALL) target/release/slingshot $(INSTALL_DIR)
	@echo "Creating a symbolic link to slingshot in $(INSTALL_DIR)"
	@$(LN) $(abspath target/release/slingshot) $(INSTALL_DIR)/slingshot

uninstall:
	@echo "Removing slingshot from $(INSTALL_DIR)"
	@rm -f $(INSTALL_DIR)/slingshot

.PHONY: all build install uninstall 

