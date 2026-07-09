# CI/CD Operations

This repo uses GitHub Actions for build, validation, container publishing, and optional Kubernetes deployment.

## CI

The `CI` workflow runs on pushes to `main` or `master` and on pull requests:

- checks Rust formatting
- runs Clippy with warnings denied
- runs all workspace tests
- builds the workspace in release mode
- verifies the Docker image can be built

## CD

The `CD` workflow runs on pushes to `main` or `master`, version tags matching `v*`, and manual dispatch.

It publishes the app image to GitHub Container Registry:

```text
ghcr.io/<owner>/<repo>/liquidity-hub:<git-sha>
```

## Required Repository Settings

For image publishing:

- GitHub Actions must have package write permission.
- The workflow uses the built-in `GITHUB_TOKEN`.

For Kubernetes deployment:

- Add repository secret `KUBE_CONFIG`.
- Trigger the `CD` workflow manually with `deploy=true`.
- The workflow applies `deploy/k8s/liquidity-hub.yaml`, updates the image tag, and waits for rollout.

## Local Parity

Run the same Rust gate locally before pushing:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
```

Smoke test the app:

```bash
RUST_LOG=trace cargo run -p liquidity-hub-app -- --ticks 1
```

The app writes compact logs to stdout and JSON logs to `logs/liquidity-hub.jsonl`.
