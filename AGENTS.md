# jalak - agent instructions

## Planning and task tracking

**OpenSpec is the only planning and task-tracking workflow in this repository.**
Do not use markdown TODO lists, external issue trackers, or a second task system.

Start an OpenSpec change when the work touches product direction, the HTTP API,
the database schema, persistence, workflow, the timeline or editor UI, or
architecture. A focused bug fix that preserves the existing design does not need
one.

An OpenSpec change owns the whole lifecycle:

- `proposal.md` — why, what changes, and the affected capabilities
- `specs/<capability>/spec.md` — requirements and scenarios
- `design.md` — decisions and rejected alternatives, when the change needs them
- `tasks.md` — the implementation checklist and the proof for each task

Work the tasks in order, tick each box as it lands, and record the proof named
in the task. When implementation turns up work outside the current scope, add it
as a new task or a separate OpenSpec change — never as a comment that gets lost.

```bash
openspec list                       # active changes
openspec status --change "<name>"   # artifacts and progress
openspec validate "<name>" --strict # before proposing implementation is done
```

Use the OpenSpec skills as the default lifecycle:

1. `openspec-explore` — investigate and shape the change without editing
   implementation.
2. `openspec-propose` — create the proposal, design, delta specs, and tasks.
3. `openspec-apply-change` — implement tasks, tick them off, and record proof.
4. `openspec-archive-change` — close out a completed change.

Before using `openspec-archive-change` or running archive commands, ask the user
for explicit confirmation. During archive, assess delta specs and use
`openspec-sync-specs` when the accepted requirements need to be merged into
`openspec/specs/`; the archive flow normally asks whether to sync before moving
the change into `openspec/changes/archive/`.

## Workspace map

| Path | Name | Purpose |
|------|------|---------|
| `apps/desktop` | Jalak desktop | macOS menu-bar audio profile application |
| `openspec` | OpenSpec | Product requirements, designs, and implementation tasks |

## Running

```bash
cargo run -p jalak-desktop
./scripts/package-macos.sh
open target/Jalak.app
```

## Environment

- macOS
- Rust 1.95 or newer
- Xcode Command Line Tools
- Working default audio output device

## Verification (run before declaring work done)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
./scripts/package-macos.sh
```

## Documentation rules

- Update the owning doc in the same pass as the change, or say why not.
- `docs/decisions/` holds accepted architecture decisions only, and the bar is
  high — see `docs/decisions/README.md` before adding one.
- `docs/CODEMAPS/` describes the current shape, not aspirations. It is not a
  task log, release process, or contribution policy.
- Delete stale text instead of layering caveats on it.
- A user-facing or contributor-facing change adds a `CHANGELOG.md` entry under
  `Unreleased`. See `CONTRIBUTING.md` for the exempt categories.

## Commit and release

Lowercase conventional commits: `feat`, `fix`, `refactor`, `chore`, `docs`,
`perf`, `test`, `ci`. No emojis. For changes spanning several workspaces, prefer
one atomic commit.

One canonical version lives in root `package.json`, with one `CHANGELOG.md` and
one annotated `vX.Y.Z` tag per release. Release steps are in `CONTRIBUTING.md`.
Do not bump the version as part of a feature change; releases are cut
separately.
