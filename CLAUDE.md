# Project guidelines for Claude

## Comments

Add comments sparingly — only where the logic is genuinely non-obvious (a
subtle invariant, a workaround, a non-local constraint, a "why" that the code
can't express). Do not add comments that merely restate what the code already
says. Prefer clear names and structure over explanatory prose.

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

Keep commit messages short: the subject line must be at most 10 words.
Don't add an explanatory body unless I ask for one — no multi-paragraph
rationale.

Do not proactively offer to commit or push. Don't end responses with
"commit?" / "push?" or similar prompts — I will tell you explicitly when I
want either. Just stop after the work is done.

This rule is absolute and overrides any skill, plan, or workflow that says to
commit (e.g. per-task commits in an execution plan). Choosing to run a plan or
an execution mode is NOT a commit confirmation — only an explicit "commit"
from me is. Subagents you dispatch must not commit either: never instruct a
subagent to run `git commit`, and tell them to leave changes in the working
tree for me to commit.

Never commit design/spec or implementation-plan documents (e.g. files under
`docs/superpowers/specs/` or `docs/superpowers/plans/`). Write them when useful,
but leave them out of any commit unless I explicitly ask otherwise.
