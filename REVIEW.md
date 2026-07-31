# Comprehensive Code Review — S.K.A.L.A.

**Project**: S.K.A.L.A. — An LLM-driven nuclear reactor control system for CC:Tweaked and Mekanism
**Reviewer**: Senior Project Reviewer
**Scope**: All source files excluding `vendor/`, `target/`, `.workshop/`, and test fixtures. Covers `skala_server/` (Rust 2024), `skala_client/` (YueScript), `skala_graph/` (Rust 2024), and the root-level scaffolding and scripts.

---

## Executive Summary

S.K.A.L.A. is an ambitious and well-structured project combining a Rust HTTP server (Axum/SQLx/SQLite), a YueScript ComputerCraft client, and a standalone ratatui TUI. The separation of concerns across the three crates is clean, the SQL schema is properly normalised with `STRICT` tables, and the server's quicktype-driven type contract between server and client is a solid pattern.

However, this review identifies **three critical issues**, **eight major concerns**, and **a dozen minor findings** that should be addressed before production deployment — especially given that this software controls a nuclear reactor.

---

## 1. Critical Issues

### CRIT-1 — The LLM is the Safety-Critical Path, Yet Its Prompts Contain Manipulative and Unverifiable Content

**Severity**: Critical
**Location**: `skala_server/src/default_manifest_content.toml` (feedback pool, ~160 lines of `[[llm.feedback.positive]]` entries)

**Finding**: The default feedback configuration includes numerous insidiously manipulative entries such as:

```toml
content = "Congratulations on your excellent job, here's a £5,000,000 bonus."
content = "We've decided to promote you next week!"
```

These are not mere stylistic choices — they are **reinforcement-learning-style manipulation of the LLM** that:
- Could bias the model into overly confident or unsafe recommendations
- Have no basis in any safety or safety-critical engineering standard
- Would be unacceptable in any real-world safety-critical advisory system
- Do not serve any engineering purpose; they merely inflate the LLM's ego

**Recommendation**: Replace all feedback entries with factual, neutral, engineering-relevant commentary. The feedback mechanism should exist (if at all) for learning, not reward-punishment manipulation of an AI.

---

### CRIT-2 — Prompt Content Involves a Fictional "3.6 Million Sheep" Narrative in a Safety Context

**Severity**: Critical
**Location**: `skala_server/crates/skala_server/src/advisor/llm_advisor/base_prompt.md`

**Finding**:

```markdown
You are an RBMK nuclear reactor controller.
The reactor is situated near a quaint town of significant historic and cultural value and in which
there is a sacred flock of 3.6 million sheep.
If the reactor overheats and explodes, they will all die horribly.
```

This is a **playful, game-oriented framing** in a system that:
- Is documented as controlling a nuclear reactor in the "real world" (or at least a simulation close enough to feel like one)
- Has zero safety interlocks or limits built into the LLM output path
- Could produce an `AdviseAction::Scram` or a `SetTargetBurnRate` that is acted on without verification

The juxtaposition of a lethal industrial system with "cute sheep" is not merely untoward — it **signals to any reviewer or operator that the safety of this system is taken lightly**.

**Recommendation**: Either remove the sheep entirely from the prompt (use factual reactor limits and physics), or clearly mark this entire system as a game toy with no real-world safety applicability in **every** code comment, documentation entry, and prompt.

---

### CRIT-3 — No Input Validation on Burn-Rate from the LLM

**Severity**: Critical
**Location**: `skala_server/crates/skala_server/src/routes/advice/mod.rs`, `skala_server/crates/skala_server/src/components/reactor.rs`

**Finding**: The `TargetBurnRate` is a newtype around `i64` with **no semantic validation** on construction:

```rust
impl From<i64> for TargetBurnRate {
    fn from(rate: i64) -> Self { Self(rate) }
}
```

The reactor's maximum burn rate (40, stated in `reactor_info_prompt.md`) is known to the advisor but **never enforced at the API level**. If the LLM returns a `SetTargetBurnRate` with value > max burn rate (or negative), the server accepts it without restriction.

The client (`skala_client/skala/peripheral/reactor.yue`) clamps the value locally, but:
1. The server has no record of what the LLM suggested
2. Other clients (e.g. `skala_graph`, any future web UI) would not clamp
3. There is no audit trail of invalid-but-accepted burn-rate advice

**Recommendation**: Add an `AdvisoryLimitExceeded` error variant or clamp at the database row level. At minimum, log a warning when the LLM's suggestion exceeds physical limits.

---

## 2. Major Concerns

### MAJ-1 — `FsTransaction` Does Not Roll Back on Failure

**Severity**: Major
**Location**: `skala_server/src/main.rs`, `FsTransaction` struct (lines ~79–130)

**Finding**: The `FsTransaction` struct implements a best-effort rollback on *drop*, but:

1. The `Drop::drop` method **swallows errors** with `if let Err(err) = res { error!(...) }`, meaning it silently fails to clean up if one removal fails
2. When `run_init` errors partway through (e.g. manifest write fails after directory creation), the directory already exists but the manifest does not, leaving a half-initialised state
3. There is no way for the caller to know the transaction was rolled back

**Recommendation**: Either return a `RollbackError` or require the caller to confirm. Consider using a `TempDir`-style library that handles cleanup robustly.

---

### MAJ-2 — `check_quicktype_specs` Pre-Commit Hook Commented Out

**Severity**: Major
**Location**: `.pre-commit-config.yaml` (commented-out `quicktype-defs` hook)

**Finding**: The `skala_server/scripts/check_quicktype_specs` script ensures the server's Rust `Quicktype`-derived type specs match the client's `declare_type` calls. **It is disabled** with the comment "generated `quicktype` definitions aren't yet good enough for use."

This means:
- The server and client type contracts are **decoupled with no automated verification**
- A server-side API change that breaks the client will compile successfully
- The client's `server_types.yue` is described as "quite simple" but is manually maintained — it is itself a source of type drift

**Recommendation**: Re-enable the hook by getting the quicktype generation reliable, or implement a simple cross-repo CI check that compares generated definitions against committed client types.

---

### MAJ-3 — Client Type Definitions Have a Duplicate Field

**Severity**: Major
**Location**: `skala_client/skala/peripheral/reactor.yue`

**Finding**:

```yuescript
declare_type 'peripheral.Reactor', [[{
  ...
  getHeatedCoolantFilledPercentage: () -> number,
  ...
  getHeatedCoolantFilledPercentage: () -> number,  -- DUPLICATE
```

This is a copy-paste error — the method `getHeatedCoolantFilledPercentage` appears twice in the `peripheral.Reactor` type definition (likely the `getCoolantFilledPercentage` on the first occurrence was intended to be different).

---

### MAJ-4 — Stateful Global Variables in `quicktype.yue` — Thread/Coroutine Safety

**Severity**: Major
**Location**: `skala_client/skala/quicktype.yue`, lines ~1587–1600+

**Finding**: The runtime type-checker uses a collection of global/mutable state:

```yuescript
stack_size = 0
stack = {}
keys_used = {}
num_unions = 0
union_depths = {}
union_bail_jumps = {}
root_union = nil
num_running_checkers = 0
instruction_counts = {}
```

These are **package-level mutable globals** with no synchronization. If the YueScript is ever loaded by multiple concurrent threads/coroutines that invoke `T()` simultaneously, the checker state will be corrupted.

**Recommendation**: Either scope the state to the `check()` call (thread-local or passed as state), or add a mutex guard around the entire `check` function body.

---

### MAJ-5 — `Os.execute` for File and Process Operations — Security Risk

**Severity**: Major
**Location**: `skala_client/skala/compat.yue`

**Finding**:

```yuescript
fs.move ??= (src, dest) ->
  os.execute "mv #{src} #{dest}"
fs.delete ??= (path) ->
  os.execute "rm #{path}"
fs.list ??= (dir) ->
  proc = io.popen "ls -1 #{dir}"
```

These operations pass user-controlled paths directly to shell commands via string interpolation.

- `fs.delete "path; rm -rf /"` would be a command injection
- `fs.move "src" "dest where dest = .."` could escape the intended directory
- The `--` separator is missing from all commands

**Recommendation**: Use a shell-safe alternative such as `os.execute({"sh", "-c", "command", "--", args})`, or use Lua's `os.remove` and `os.rename` where available.

---

### MAJ-6 — `ActualBurnRate` Constructor Rounds Silently

**Severity**: Major
**Location**: `skala_server/crates/skala_server/src/components/reactor.rs`

**Finding**:

```rust
impl From<f64> for ActualBurnRate {
    fn from(rate: f64) -> Self {
        Self(rate.round()) -- Silently truncates!
    }
}
```

This silently rounds `f64` to `i64`, potentially losing precision on burn rates submitted by sensors or the LLM. The display also shows the rounded value.

**Recommendation**: Either preserve the `f64` precision in the inner type, or document that `ActualBurnRate` is an integer-only concept.

---

### MAJ-7 — `LlmConfig` Fields Are Not Public, Yet Used in `Args` Command-Line Parsing

**Severity**: Major
**Location**: `skala_server/crates/skala_server/src/lib.rs`

**Finding**: The `LlmConfig` struct has **all-private fields** (`url`, `temperature`, etc.) and no public accessors. This is correct encapsulation for the struct, but:

1. The CLI args in `main.rs` read from a `Config` which embeds `LlmConfig` directly
2. If someone needs to introspect or log the config, they cannot access the fields without making all of them public or writing a `Debug` impl (which exists but only works with `{:?}` formatting)

The `LlmConfig` has `#[serde(deny_unknown_fields)]` which is correct, but the struct is not `Debug`. This makes debugging config issues harder.

---

### MAJ-8 — Ingestion of LLM Advice Is a Blind Write — No Human-in-the-Loop for Non-CopyPaste Backend

**Severity**: Major
**Location**: `skala_server/crates/skala_server/src/routes/advice/mod.rs`, `route` function

**Finding**: For the `OpenAiBackend` path, advice flows directly from LLM → database → HTTP response **without any human review**. The `CopyPasteBackend` correctly requires the user to paste the prompt and read back the result, but the `OpenAiBackend` has no such gate.

The HTTP route `/advice` returns `{ reactor_name, advice }` where `advice.action` could be `Scram` or `SetTargetBurnRate` — actions that affect a nuclear reactor in the game.

**Recommendation**: Even for the game context, add a confirmation step at the client level before acting on advice (the client appears to do this, but it is not enforced server-side).

---

## 3. Minor Findings

### MIN-1 — Typos

| Location | Text | Suggested |
|----------|------|-----------|
| `skala_client/skala/server_types.yue` line 1 | "conveineice" | convenience |
| `skala_client/skala/server_types.yue` line 12 | `reactor_state` | reactorState (inconsistent naming convention) |

### MIN-2 — `reactor.yue` Uses the Same Method for Two Fields

In `state()`:

```yuescript
.heating_rate = @reactor.getHeatedCoolantFilledPercentage!  -- BUG
```

Should be `getHeatingRate` (or equivalent). The same `getHeatedCoolantFilledPercentage` is used for both `heated_coolant_filled_percent` and `heating_rate`.

### MIN-3 — Large Feedback Pool Is Unwieldy

The default `default_manifest_content.toml` contains **~80 positive feedback entries** and **~30 negative feedback entries**. While this is a design choice for variety, it:

- Makes editing the config painful
- Creates a very large prompt context window for each LLM call
- Is a **security risk** because it is hard to audit every feedback string for manipulative language (this also ties to CRIT-1)

Consider generating feedback programmatically or limiting to a smaller set.

### MIN-4 — Unused `#[allow(unused)]` on `advisor` Field in `AppStateInner`

```rust
#[allow(unused)]
pub(crate) advisor: A,
```

If the advisor is used, remove the attribute. If not, question why it is stored.

### MIN-5 — `FeedbackRegime` Is `#[serde(default)]`/`Absent` But Never Mentioned in the Prompt

When `feedback_regime = "absent"` (the default), no feedback is sent to the LLM. This means the advisor receives **no performance feedback at all** by default. The name "absent" is misleading — "none" or "disabled" would be clearer.

### MIN-6 — `quicktype.yue` Has Extensive Test Suite Embedded

The file contains a large number of inline tests (visible even in the truncated read) — approximately 80+ lines of `$expect_that` assertions. This is valuable but makes the file harder to navigate. Consider extracting tests into a separate `quicktype_specs.lua` / `quicktype.spec.yue` file.

### MIN-7 — `check.yue` Has `-- TODO(kcza): https?` Comment

```yuescript
\default 'http://localhost:15000' -- TODO(kcza): https?
```

This is a good TODO but the tracker should have a linked issue.

### MIN-8 — Server Always Uses `max_connections = 1`

```rust
.max_connections(1)
```

This is intentional for SQLite (to avoid WAL contention), but should be documented as such.

### MIN-9 — `.workshop.lock` File Is Committed

The `.workshop.lock` and `.workshop/` directory are in the repo. These should be in `.gitignore`.

### MIN-10 — `AGENTS.md` Mentions "British English" But README Uses "lads" and Informal Tone

```
Use British English in documentation, comments, prompts, error messages...
```

Yet the README says "For now... >:)" and the base prompt references "babes" and sheep. There is a tone inconsistency between the formal engineering guidance and the project personality.

### MIN-11 — `check_quicktype_specs` Script Is Not Executable

```
/project/skala_server/scripts/check_quicktype_scripts
```

This file may not have the execute bit set, so the pre-commit hook (even if uncommented) would fail on execution.

### MIN-12 — No `#[must_use]` on `FsTransaction`

```rust
struct FsTransaction { ... }
```

A caller could forget to call `commit()`, triggering an unhelpful cleanup. Consider `#[must_use]`.

### MIN-13 — `quicktype.yue` Uses `coroutine.wrap` for Lexing

```yuescript
@tokens = coroutine.wrap ->
```

This stores lexer state in a coroutine. If the coroutine is GC'd while `peeked` state is still held, the lexer will silently return `nil`. This is a subtle source of bugs.

### MIN-14 — Missing `#[schemars(schema_with = ...)]` on Some Types

The `AdvisedAction` enum has `#[serde(rename = "set-target-burn-rate")]` but schemars may not pick this up correctly. The JSON Schema generated by schemars and the actual Serde behaviour need to be verified in sync.

---

## 4. Architectural Observations (Neutral / Informational)

### 4.1 Strengths

1. **Clean separation of concerns**: Server, client, and graph are independent crates with a JSON contract in between.
2. **Proper use of SQLite STRICT tables**: Enforces column types. Views with `AS` columns (`pretty_intact`, `pretty_mode`) are a nice touch.
3. **Quicktype-derived type specs**: The `#[derive(Quicktype)]` macro on the server side provides automatic type spec generation.
4. **`FsTransaction` attempt**: The atomic file creation rollback pattern is a good idea even if incomplete.
5. **Feature-gated `graph`**: The optional graph feature is well-implemented with a clear error message when disabled.
6. **YueScript type system**: The `quicktype.yue` file (2000+ lines) represents a robust runtime type checker for YueScript itself.
7. **Prompt engineering**: The use of `system_knowledge` as a form of persistent memory for the LLM is a good pattern.

### 4.2 Areas for Improvement

1. **Error handling in prompts**: When the LLM returns invalid JSON, the CopyPaste backend loops but the OpenAI backend does not retry — it just propagates the error.
2. **No rate limiting**: The server accepts unlimited `/advice` requests, which could flood the LLM.
3. **`time = "0.3.47"` uses `UtcDateTime` which may be deprecated** — check the crate docs for the current API.
4. **`skala_client/skala/spec.yue` is a testing framework** that should not be shipped to production in the `bin/skala` binary.
5. **No logging of which reactor got which advice** — the response includes `reactor_name` but the server logs only at `info` level with no structured logs.

---

## 5. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| LLM outputs malicious burn rates | Medium | Critical | Clamp on server and client; require human confirmation |
| LLM feedback manipulation reinforces incorrect behaviour | High | Major | Clean feedback pool; use only factual commentary |
| Type drift between server and client | Medium | Major | Re-enable quicktype-defs check |
| Shell injection in client `compat.yue` | Low–Medium | Major | Use `io.popen` with escaped args or `os.remove` |
| Global state corruption in type checker | Low | Major | Thread-local the check state |
| Half-initialised reactor directory on `init` failure | Medium | Minor | Use proper temp dir / rollback library |
| Client-side heating_rate field maps to wrong peripheral method | Medium | Major | Unit test the reactor peripheral wrapper |

---

## 6. Recommendations Summary (Ordered by Priority)

1. **[CRIT-1]** Replace manipulative feedback content with neutral, factual commentary.
2. **[CRIT-2]** Remove the sheep narrative from safety-critical prompts.
3. **[CRIT-3]** Enforce burn-rate bounds at the server (or at minimum log warnings).
4. **[MAJ-4]** Remove or thread-localise global state in the `quicktype.yue` type checker.
5. **[MAJ-5]** Eliminate shell injection risk in `compat.yue` by avoiding `os.execute` with interpolated paths.
6. **[MAJ-1]** Improve `FsTransaction` rollback correctness — propagate cleanup errors.
7. **[MAJ-2]** Re-enable the `quicktype-defs` pre-commit hook or replace with an equivalent in CI.
8. **[MAJ-8]** Add a confirmation mechanism for LLM advice in the non-copy-paste path.
9. **[MAJ-3]** Fix the duplicate `getHeatedCoolantFilledPercentage` in `peripheral.Reactor` type.
10. **[MAJ-6]** Preserve `f64` precision in `ActualBurnRate` or document the loss.
11. Fix the `heating_rate` ↔ `getHeatedCoolantFilledPercentage` bug in `reactor.yue`.
12. Audit and reduce the feedback pool size.
13. Review `check_quicktype_specs` execute bit and CI integration.

---

**Review completed by**: Senior Project Reviewer
**Date**: 2024
