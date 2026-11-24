## Summary

<!-- What does this change do, and why? -->

## Linked issue

<!-- One parent issue, plus any of its sub-issues this PR finishes. One "Closes" line per issue: closing the parent does not close its sub-issues. Unrelated work goes in a separate PR. -->

Closes #

## Test plan

<!-- How did you check this works? List commands run and what you saw. -->

- [ ]

## Checklist

- [ ] Tests added or updated
- [ ] `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean
- [ ] Public items are documented
- [ ] No outbound network code, and any listener binds to `127.0.0.1` only
- [ ] No real captures, personal data or data files committed
- [ ] ADR added if `cc-core` contracts or the architecture changed
- [ ] Title is `type(scope): description` with no number, and each commit is under 200 changed lines (or has a `Large-Commit` trailer)
- [ ] If an AI tool drafted any of this, the description ends with a one-line disclosure
