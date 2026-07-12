# CI/CD Operations

This repo uses GitHub Actions for build, validation, coverage, security, container publishing, and optional Kubernetes deployment.

## CI

The `CI` workflow runs on pushes to `main` and on pull requests:

- checks Rust formatting
- runs Clippy with warnings denied
- runs all workspace tests
- builds the workspace in release mode with `Cargo.lock`
- verifies the Docker image can be built

Additional validation workflows:

- `GitHub Actions`: lints workflow YAML with actionlint
- `Coverage`: generates LCOV coverage with `cargo-llvm-cov`
- `CodeQL`: scans Rust code and uploads SARIF results as workflow artifacts
- `Rust Security`: runs RustSec audit and cargo-deny policy checks
- `Dependency Review`: reviews dependency changes on pull requests
- `Dependency Submission`: submits Cargo dependency snapshots to GitHub's dependency graph
- `SonarCloud Analysis`: uploads LCOV and source metrics when SonarCloud variables/secrets are configured
- `Cloud E2E`: checks `/health` and `/ready` on the deployed cloud service

## CD

The `CD` workflow runs on pushes to `main`, version tags matching `v*`, and manual dispatch.

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
- Trigger the `CD` workflow manually with `deploy_kubernetes=true`.
- The workflow applies `deploy/k8s/liquidity-hub.yaml`, updates the image tag, and waits for rollout.

For Render deployment:

- Connect the repository as a Render Blueprint.
- Use `render.yaml`.
- The service starts the app with `LH_SERVE_ADMIN=true` and exposes `/health`.

For cloud verification:

- Add repository variable `CLOUD_BASE_URL`.
- Run the `Cloud E2E` workflow manually or let the daily schedule run.

For SonarCloud:

- Import the repository in SonarCloud.
- Add repository secret `SONAR_TOKEN`.
- Add repository variables `SONAR_ORGANIZATION` and `SONAR_PROJECT_KEY`.

For CodeQL:

- The workflow always runs Rust CodeQL analysis and stores SARIF output in the `codeql-rust-sarif` artifact.
- Keep `CODEQL_UPLOAD` unset while GitHub CodeQL default setup is enabled for the repository.
- Set repository variable `CODEQL_UPLOAD=always` only if default setup is disabled and SARIF uploads from advanced setup are supported. GitHub rejects uploads from advanced configurations while default setup is active.

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
