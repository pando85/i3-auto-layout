#!/bin/bash
set -e

# bump version
vim Cargo.toml

# test
make release
pkill i3-auto-layout
sudo cp ${CARGO_TARGET_DIR:-target}/x86_64-unknown-linux-gnu/release/i3-auto-layout /usr/bin/i3-auto-layout
i3-msg restart

make update-changelog

git add .
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
git commit -m "release: Version $VERSION"

echo "After merging the PR, tag and release are automatically done"
