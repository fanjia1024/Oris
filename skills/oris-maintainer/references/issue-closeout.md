# Issue Closeout

## Closeout Policy

- Close an issue only after the code is merged or pushed to the intended branch and the crate publish succeeded.
- Use one released crate version per completed issue by default.
- Keep the issue comment short and factual: shipped change, validation run, and released version.

## Comment Template

Use this pattern when the issue should remain open briefly after release confirmation:

```bash
gh issue comment <issue_number> --body "Shipped in oris-runtime v<version>.

Summary:
- <one-line behavior change>

Validation:
- cargo fmt --all -- --check
- <targeted test commands>
- <release dry-run or publish confirmation>
"
```

## Close Template

Use this pattern when the issue is ready to close immediately:

```bash
gh issue close <issue_number> --comment "Completed and released in oris-runtime v<version>.

Validation:
- cargo fmt --all -- --check
- <targeted test commands>
- cargo publish -p oris-runtime --all-features
"
```

## Notes

- Replace placeholders with exact commands actually run. Do not claim coverage you did not execute.
- If publish was blocked, post a status comment instead of closing the issue.
- If the issue was completed without a release by explicit user instruction, say that clearly in the final comment and do not use the default release wording.
