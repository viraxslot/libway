# Project guidelines for Claude

## Git

Always ask for explicit confirmation before running any git operation that
creates or rewrites history, or that publishes to a remote:

- `git commit` (including `--amend`)
- `git push` (including `--force` / `--force-with-lease`)
- `git reset`, `git rebase`, `git cherry-pick`, `git revert`
- deleting or moving branches/tags

Read-only and staging operations are fine without asking: `git status`,
`git diff`, `git log`, `git add`, `git stash list`, `git branch` (listing).

When I do ask for a commit or push, briefly summarize what will be
committed/pushed first.

Do not proactively offer to commit or push. Don't end responses with
"commit?" / "push?" or similar prompts — I will tell you explicitly when I
want either. Just stop after the work is done.
