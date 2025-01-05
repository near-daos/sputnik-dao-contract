#!/bin/bash
sudo apt update
sudo apt install -y pkg-config
sudo apt install -y clang
rustup target add wasm32-unknown-unknown

# Install Binaryen
wget https://github.com/WebAssembly/binaryen/releases/download/version_121/binaryen-version_121-x86_64-linux.tar.gz
tar -xvzf binaryen-version_121-x86_64-linux.tar.gz 
echo 'export PATH="$(pwd)/binaryen-version_121/bin:$PATH"' >> ~/.bashrc
