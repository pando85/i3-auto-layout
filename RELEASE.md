# Release workflow

- Bump version in `Cargo.toml`.
- Test:
  ```bash
  make release
  pkill i3-auto-layout
  sudo cp ${CARGO_TARGET_DIR}/x86_64-unknown-linux-gnu/release/i3-auto-layout /usr/bin/i3-auto-layout
  i3-msg restart
  ```
- Update `CHANGELOG.md` with `make update-changelog`.
- Merge PR.
- Tag version in main branch: `make tag`

## Upgrade dependencies manually

Requirements:

- `cargo-edit`: `cargo install cargo-edit`

Upgrade dependencies:

- `cargo upgrade` or `cargo upgrade --incompatible`

Update cargo lock dependencies:

- `cargo update`
