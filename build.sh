#!/bin/bash
# Build script för Genlib Desktop

set -e

# Färger för output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}==>${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

print_error() {
    echo -e "${RED}Error:${NC} $1"
}

# Visa hjälp
show_help() {
    echo "Genlib Desktop Build Script"
    echo ""
    echo "Användning: ./build.sh [kommando]"
    echo ""
    echo "Kommandon:"
    echo "  dev       Bygg debug-version (standard)"
    echo "  release   Bygg optimerad release-version"
    echo "  check     Snabb kompileringskontroll"
    echo "  test      Kör alla tester"
    echo "  run       Bygg och kör debug-version"
    echo "  run-rel   Bygg och kör release-version"
    echo "  clean     Rensa build-artefakter"
    echo "  fmt       Formatera kod med rustfmt"
    echo "  clippy    Kör clippy linter"
    echo "  all       Kör fmt, clippy, test och release build"
    echo "  help      Visa denna hjälp"
    echo ""
}

# Kontrollera att cargo finns
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "cargo hittades inte. Installera Rust från https://rustup.rs"
        exit 1
    fi
}

# Debug build
build_dev() {
    print_status "Bygger debug-version..."
    cargo build
    print_status "Debug-build klar: target/debug/genlib-desktop"
}

# Release build
build_release() {
    print_status "Bygger release-version (optimerad)..."
    cargo build --release

    # Visa filstorlek
    if [ -f "target/release/genlib-desktop" ]; then
        SIZE=$(du -h target/release/genlib-desktop | cut -f1)
        print_status "Release-build klar: target/release/genlib-desktop ($SIZE)"
    fi
}

# Snabb check
check() {
    print_status "Kontrollerar kompilering..."
    cargo check
    print_status "Kompileringskontroll klar"
}

# Kör tester
run_tests() {
    print_status "Kör tester..."
    cargo test
    print_status "Alla tester passerade"
}

# Kör debug-version
run_dev() {
    print_status "Bygger och kör debug-version..."
    cargo run
}

# Kör release-version
run_release() {
    print_status "Bygger och kör release-version..."
    cargo run --release
}

# Rensa
clean() {
    print_status "Rensar build-artefakter..."
    cargo clean
    print_status "Rensning klar"
}

# Formatera
format() {
    print_status "Formaterar kod..."
    if command -v rustfmt &> /dev/null; then
        cargo fmt
        print_status "Formatering klar"
    else
        print_warning "rustfmt hittades inte. Kör: rustup component add rustfmt"
    fi
}

# Clippy
run_clippy() {
    print_status "Kör clippy..."
    if command -v cargo-clippy &> /dev/null || cargo clippy --version &> /dev/null; then
        cargo clippy -- -W clippy::all
        print_status "Clippy klar"
    else
        print_warning "clippy hittades inte. Kör: rustup component add clippy"
    fi
}

# Kör allt
run_all() {
    format
    run_clippy
    run_tests
    build_release
    print_status "Allt klart!"
}

# Main
check_cargo

case "${1:-dev}" in
    dev)
        build_dev
        ;;
    release)
        build_release
        ;;
    check)
        check
        ;;
    test)
        run_tests
        ;;
    run)
        run_dev
        ;;
    run-rel)
        run_release
        ;;
    clean)
        clean
        ;;
    fmt)
        format
        ;;
    clippy)
        run_clippy
        ;;
    all)
        run_all
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        print_error "Okänt kommando: $1"
        show_help
        exit 1
        ;;
esac
