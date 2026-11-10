# SonarCloud

This repository uses the same GitHub Actions lifecycle as the other vtsyryuk repositories: build/test verification, real coverage where available, then analysis and a required quality-gate result. The scanner and build runtime follow this repository's language and existing build system.

## Activate analysis

1. Import `vtsyryuk/rust-playground` from GitHub into the SonarCloud `vtsyryuk` organization so pull-request decoration is bound to this repository. Use project key `vtsyryuk_rust-playground`. The GitHub repository is public; use a public SonarCloud project.
2. In SonarCloud Project Administration → Analysis Method, disable Automatic Analysis. GitHub Actions owns analysis and coverage reporting.
3. Add the GitHub Actions secret `SONAR_TOKEN`, issued to an account/token with Execute Analysis permission on this project. Never commit or paste the token into logs.
4. Set the GitHub Actions variable `SONAR_ENABLED` to `true`. Optional variables `SONAR_ORGANIZATION` and `SONAR_PROJECT_KEY` override the checked-in defaults `vtsyryuk` and `vtsyryuk_rust-playground`.
5. Run **SonarCloud Analysis** manually on the default branch first. Confirm its analysis job and quality gate pass before relying on PR decoration. Select **Sonar way**, the organization default quality gate, in the project settings.

Until activation, the analysis job is visibly skipped and the run summary states that no analysis or quality gate ran. Build/test verification continues. Once enabled, a missing token, scanner failure, or failed quality gate fails the analysis job. A successful build alone never claims a successful scan. Fork and Dependabot PRs receive build/test verification without access to analysis credentials; the merged default-branch push is scanned.

## Build and coverage

Scanner inputs: `target/coverage/lcov.info`. The workflow uploads these for the analysis job with one-day retention. Coverage comes from the existing test suite, not synthetic or estimated values.



After activation, make `SonarCloud analysis and quality gate` a required check in GitHub branch protection if desired. Do not require that job for untrusted PRs whose analysis is intentionally skipped.
