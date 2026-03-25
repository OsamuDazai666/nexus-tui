# Publishing

Maintainer-facing release notes for publishing `ani-nexus-tui` to crates.io.

## One-Time Setup

1. Create a crates.io API token with publish permissions.
2. Add it to GitHub repository secrets as `CARGO_REGISTRY_TOKEN`.

## Release Flow

1. Bump `version` in `Cargo.toml`.
2. Commit the version change.
3. Push a matching Git tag such as `v0.1.1`.

The workflow in `.github/workflows/publish-crate.yml` will:
- verify the tag matches `Cargo.toml`
- skip publishing if that version already exists on crates.io
- run formatting, clippy, tests, and `cargo package`
- publish the crate automatically

## Example

```bash
git add Cargo.toml Cargo.lock
git commit -m "release: v0.1.1"
git tag v0.1.1
git push origin main --follow-tags
```
