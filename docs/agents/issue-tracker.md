# Issue tracker: GitHub & Multica

Issues, epics, and specs for CoSave are tracked via GitHub issues and orchestrated through Multica. Use the `gh` CLI for all command-line operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."` (or create directly via Multica / Mika)
- **Read an issue**: `gh issue view <number> --comments`
- **List issues**: `gh issue list --state open`
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply / remove labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close an issue**: `gh issue close <number> --comment "..."`

The repository is inferred automatically from `git remote -v` (`https://github.com/akashgurava/cosave.git`).

## Pull Requests as a Triage Surface

**PRs as a request surface: no.**
PRs are used strictly for feature delivery and review:
- Every feature epic integrates through an intermediate branch: `feature/<issue-id>-<slug>`.
- Individual tracer-bullet tickets are implemented on isolated branches: `feat/<ticket-id>-<slug>`.
- IC agents open PRs targeting `feature/<issue-id>-<slug>`.
- Lead Developer reviews PRs via `/code-review` and merges into `feature/<issue-id>-<slug>`.
- Once all tickets in the epic are merged, a final PR into `main` is opened for human approval.

## When a skill says "publish to the issue tracker"
Create a GitHub issue or post a sub-issue in Multica.

## When a skill says "fetch the relevant ticket"
Run `gh issue view <number> --comments` or fetch the ticket body from Multica.
