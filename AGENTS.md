# Repository Guidelines

## Scope

These instructions apply to the whole repository, except vendored code under
`skala_server/vendor/**`. Do not modify vendored code unless the task explicitly
requires it.

Ignore any `AGENTS.md` files found inside vendored dependencies; they belong to
upstream packages and do not override these instructions. Scoped `AGENTS.md`
files under `skala_client/` and `skala_server/` add local guidance for those
parts of the project.

## Language And Tone

- Use British English in documentation, comments, prompts, error messages, and
  user-facing copy.
- Keep technical documentation formal and operational. The README has a playful
  project tone, but do not introduce humour into agent guidance or engineering
  instructions unless you are editing existing themed copy.

## Coding Standards

- New functions must include explicit type annotations where the language
  supports them.
- Keep changes close to the surrounding style. Avoid broad refactors while
  making feature or bug-fix changes.
- Prefer small, behaviour-focused changes with matching tests when code changes
  affect server routes, advisor behaviour, generated type contracts, or client
  control flow.

## Verification

- Run checks that are relevant to the area changed. Use scoped `AGENTS.md` files
  for client and server commands.
- For documentation-only changes, a focused Markdown review and
  `git diff --check` are enough.

## Workflow Safety

- Preserve user changes in a dirty worktree. Do not revert or overwrite work you
  did not make unless explicitly instructed.
