# Triage Labels

This file defines the canonical triage roles for CoSave and maps them to the GitHub issue tracker label strings.

| Canonical Role | Issue Tracker Label | Meaning |
| --- | --- | --- |
| `needs-triage` | `needs-triage` | Maintainer needs to evaluate this issue |
| `needs-info` | `needs-info` | Waiting on reporter for more information |
| `ready-for-agent` | `ready-for-agent` | Fully specified, ready for an AFK agent |
| `ready-for-human` | `ready-for-human` | Requires human implementation |
| `wontfix` | `wontfix` | Will not be actioned |

## Conventions

- Apply exactly one category role (`bug` or `enhancement`) and one state role.
- When a skill mentions a role (e.g. "apply the AFK-ready triage label"), apply the matching label string from this table.
