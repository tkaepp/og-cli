# The OG CLI

The powertool for DG developers.

## Releasing

1. Update version in `Cargo.toml` and run `cargo check` to ensure version is
   propagated to `Cargo.lock`
2. Create tag with `git tag -am "Version 0.10.0" v0.10.0`
3. Push with `git push --follow-tag`

The release workflow will start on the tagged commit.
