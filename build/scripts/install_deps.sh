#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#

# bin/bash
set -o errexit
set -o nounset
set -o pipefail

# "---------------------------------------------------------"
# "-                                                       -"
# "-  Tests all dependencies required by make           -"
# "-                                                       -"
# "---------------------------------------------------------"

command bash --version >/dev/null 2>&1 || {
  # command sudo apt-get -qqq -y install curl
   echo "Please install bash"
   exit
}
echo "* Bash installed"


command curl --version >/dev/null 2>&1 || {
  # command sudo apt-get -qqq -y install curl
   echo "curl is used to download Rust & Cargo"
   echo "Please install curl"
   exit
}
echo "* Curl installed"


command rustc --version >/dev/null 2>&1 || {
  echo "Trying to install Rust & Cargo"
  command curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
}

command rustc --version >/dev/null 2>&1 || {
   echo "Please install Rust & Cargo"
   echo "https://www.rust-lang.org/tools/install"
   echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
   exit
}
echo "* Rust & Cargo installed"


# Try to install nightly if its missing
command rustc +nightly --version >/dev/null 2>&1 || {
   echo "Trying to install nightly Rust "
   command rustup toolchain install nightly
}

# Check again and if previous install failed, print guide for manual install
command rustc +nightly --version >/dev/null 2>&1 || {
   echo "Please install nightly Rust "
   echo "https://doc.rust-lang.org/book/appendix-07-nightly-rust.html"
   echo "rustup toolchain install nightly"
   exit
}
echo "* Rust nightly installed"


command cargo fmt --version >/dev/null 2>&1 || {
  echo "Trying to install cargo fmt"
  command rustup component add rustfmt
}

command cargo fmt --version >/dev/null 2>&1 || {
      echo "Cargo fmt formats rust code"
      echo "Please install cargo fmt"
      echo "rustup component add rustfmt"
      echo "https://github.com/rust-lang/rustfmt"
      exit
}
echo "* cargo fmt installed"


command cargo nextest --version >/dev/null 2>&1 || {
  echo "Trying to install cargo nextest"
  command cargo install cargo-nextest --locked
}

command cargo nextest --version >/dev/null 2>&1 || {
      echo "Cargo nextest runs tests im parallel"
      echo "Please install Cargo nextest"
      echo "cargo install cargo-nextest --locked"
      echo "https://nexte.st/book/installing-from-source.html"
      exit
}
echo "* cargo nextest installed"


command cargo outdated --version >/dev/null 2>&1 || {
      echo "Trying to install Cargo outdated"
      command cargo install --locked cargo-outdated
}

command cargo outdated --version >/dev/null 2>&1 || {
      echo "Cargo outdated tests for outdated dependencies"
      echo "Please install Cargo outdated"
      echo "cargo install --locked cargo-outdated"
      echo "https://github.com/kbknapp/cargo-outdated"
      exit
}
echo "* cargo outdated installed"


command cargo +nightly udeps --version >/dev/null 2>&1 || {
  echo "Trying to install Cargo udeps"
  command cargo install cargo-udeps --locked
}

command cargo +nightly udeps --version >/dev/null 2>&1 || {
      echo "Cargo udeps tests for unused dependencies"
      echo "Please install Cargo udeps"
      echo "cargo install cargo-udeps --locked"
      echo "https://crates.io/crates/cargo-udeps"
      exit
}
echo "* cargo udeps installed"


command cargo audit --version >/dev/null 2>&1 || {
      echo "Trying to install Cargo audit"
      command cargo install cargo-audit
}

command cargo audit --version >/dev/null 2>&1 || {
      echo "Cargo audit tests and reports security vulnerabilities"
      echo "Please install Cargo audit"
      echo "cargo install cargo-audit"
      echo "https://crates.io/crates/cargo-audit"
      exit
}
echo "* cargo audit installed"


command cargo clippy --version >/dev/null 2>&1 || {
    echo "Trying to install Cargo clippy"
    command rustup component add clippy
}

command cargo clippy --version >/dev/null 2>&1 || {
      echo "Cargo clippy checks for linting errors"
      echo "Please install Cargo clippy"
      echo "rustup component add clippy"
      echo "https://github.com/rust-lang/rust-clippy"
      exit
}
echo "* cargo clippy installed"


echo ""
echo "==============================="
echo "All DEV dependencies installed."
echo "==============================="
echo ""
