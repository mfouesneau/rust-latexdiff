#!/bin/bash

# Build script for rust-latexdiff

echo "Building rust-latexdiff in release mode..."

# Build the project
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary location: ./target/release/rust-latexdiff"
    echo ""
    echo "Usage examples:"
    echo "  # Expand LaTeX file:"
    echo "  ./target/release/rust-latexdiff expand main.tex -o expanded.tex"
    echo ""
    echo "  # Generate diff:"
    echo "  ./target/release/rust-latexdiff diff old.tex new.tex -o changes.tex"
    echo ""
    echo "  # Expand and diff:"
    echo "  ./target/release/rust-latexdiff diff old.tex new.tex --expand -o changes.tex"
    echo ""
    echo "To install system-wide, copy the binary to /usr/local/bin/"
    echo "  sudo cp ./target/release/rust-latexdiff /usr/local/bin/"
else
    echo "Build failed!"
    exit 1
fi
