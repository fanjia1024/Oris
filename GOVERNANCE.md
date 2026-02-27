# Governance

This document describes how the Oris project is maintained and how project decisions are made.

## Project roles

- Maintainers:
  - Review and merge pull requests.
  - Triage issues and security reports.
  - Manage releases and roadmap priorities.
- Contributors:
  - Propose changes through issues and pull requests.
  - Follow [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Decision making

- Prefer rough consensus through public discussion in issues/PRs.
- For conflicting proposals, maintainers make the final call based on:
  - correctness and reliability,
  - API stability and migration cost,
  - long-term maintainability.

## Release policy

- `main` is the active development branch.
- Releases are tagged and published through CI workflows.
- Breaking behavior changes must include migration notes in PR descriptions and release notes.

## Change categories

- Patch changes:
  - bug fixes,
  - documentation and non-breaking internal improvements.
- Minor changes:
  - additive APIs/features with backward compatibility.
- Breaking changes:
  - contract or behavior changes requiring user action.
  - should be announced clearly before release.

## Security and responsible disclosure

- Security issues follow [SECURITY.md](SECURITY.md).
- Private reporting is required for vulnerabilities until a fix is available.

## Communication channels

- Issues: bug reports, feature requests, support questions.
- Pull requests: implementation and design review.

## Maintainer expectations

- Act consistently with project policies.
- Keep review feedback technical, respectful, and actionable.
- Prioritize reproducible reports and tests for behavior changes.

## Policy updates

Governance policies may evolve as the project grows. Changes are made by pull request in this repository.
