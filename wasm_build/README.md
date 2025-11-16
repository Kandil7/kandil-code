# WASM Build Configuration for Kandil Code

This directory contains configuration files for building Kandil Code for WebAssembly.

## Building for WASM

To build for WebAssembly:

```bash
# Install the WASM target
rustup target add wasm32-unknown-unknown

# Build for WASM
cargo build --target wasm32-unknown-unknown --features wasm
```

## Browser Integration

The WASM build provides a JavaScript-compatible interface to core Kandil functionality:

- Command execution
- AI interaction
- Code analysis tools
- Project management functions