# Release workflow

```bash
.ci/release.sh
```

## Upgrade dependencies manually

Requirements:

- `cargo-edit`: `cargo install cargo-edit`

Upgrade dependencies:

- `cargo upgrade` or `cargo upgrade --incompatible`

Update cargo lock dependencies:

- `cargo update`
