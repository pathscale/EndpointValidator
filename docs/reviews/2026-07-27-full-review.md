# EndpointValidator Review: full

**Date:** 2026-07-27
**Scope:** the whole repository. `endpoint-validator/**` (17 `.rs` files, 1 integration test, 1 fixture),
`ws-load-test/**` (2 `.rs` files, 1 Python script, 4 data files), all 6 `.md` files, both copies of
`docs/*.ron` + `docs/config.toml`, `Cargo.toml`/`Cargo.lock`, `.claude/`.
Read-only cross-checks in `/Users/revenge/code/{api.support.cafe,nofilter.io-backend,pays.online-backend,endpointgen,endpoint-libs}`.
**Commit:** `34629bd`
**Reviewer slice:** full (sole reviewer for this repo)

## Summary

- **The name is wrong, and that is the whole story.** This is not a validator. There is no static
  analysis, no rule engine, no check of any kind. It is an interactive ratatui TUI that reads
  `services.json`, lets a human arrow-key through endpoints, fill parameters, and eyeball the JSON
  response. It never parses `.ron` (`rg` for `ron::`/`from_ron` across `endpoint-validator/src` and
  `ws-load-test/src`: zero hits). Every defect class in the brief is structurally out of its reach.

- **It is also currently non-functional.** Its login is a hardcoded two-step
  `0loginstep1`/`0loginstep2` username+password handshake (`state.rs:389-431`). Grepping
  `loginstep` across endpoint-libs and all three backends returns **zero matches**. The live protocol
  is a single `Init` endpoint taking `accessToken` over `Sec-WebSocket-Protocol` as `0init, 1<token>`.
  The sibling crate in this very workspace already uses the current format
  (`ws-load-test/protocol_header_fly.txt:1`). The tool cannot authenticate to anything it exists to test.

- **Nobody runs it.** `rg -il 'endpoint.?validator'` across api.support.cafe, nofilter.io-backend and
  pays.online-backend: zero hits, including their `.github/workflows/`. This repo has **no CI at all**
  (no `.github/` directory). It appears only in `endpoint-libs/scripts/check-chain.sh:18` and
  `docs/chain.md`, which check *version alignment*, not that the tool runs. A validator nobody runs is
  indeed a finding in itself, and this one is worse: it is a manual TUI nobody can run.

- **The good news: the checks that matter are cheap and the data is already there.** `services.json`
  already carries `roles` per endpoint, resolvable `StructRef`/`StructTable`, and `endpoint-libs 2.x`
  already ships a `Field.meta: MetaMap` annotation slot (`endpoint-libs/src/model/types.rs:74-76`,
  "Empty in 2.0"). I prototyped two of the four proposed checks against
  `nofilter.io-backend/docs/services.json` (61 endpoints). Check A flagged **2 groups, both real
  defects**, including the exact `AcceptWaitRoomGuest` vs `LeaveSession`/`LeaveWaitRoom` asymmetry from
  the sibling review. Check B flagged **4 of 61 endpoints**, catching the `GetSessionInfo` WHIP-URL leak
  **plus `ListUserSessions`, which leaks the same three ingest URLs and the sibling review did not name.**
  Both are ~150 lines of Rust each.

- **Top 3 things to do:** (1) fix or delete the dead handshake, because everything else is moot while the
  tool cannot connect; (2) add Check A (role-set asymmetry) as a `--lint` subcommand, since it needs no
  new annotation, no server, and found two true positives on the first run; (3) delete
  `src/tui/ui/ratatui_legacy.rs`, a 273-line byte-identical dead copy of `src/tui/ui.rs`.

- Code quality of the parts that *are* alive is decent. `parser/services.rs` is genuinely careful
  work with load-bearing comments and a real regression test. The rot is concentrated in the TUI and
  the docs.

---

## 1. Effectiveness (headline)

### What it catches today

Honestly: three things, all only when a human is sitting in front of it with a working server.

| Caught | Where | Notes |
|---|---|---|
| `services.json` failing to deserialize against current `endpoint-libs` | `tests/parses_generated_services.rs` | The one real automated check. Guards the `EnumVariant.comment` → `description` rot. Runs in `cargo test`. |
| A `config.toml` preset that cannot be encoded for its declared type | `parser/services.rs:57-154` | Hard error naming the type/field/enum. Only fires when a human selects that endpoint in the TUI. |
| A server rejecting a hand-driven request | TUI response pane | Human eyeball, no assertion. |

That is the complete list. There is no pass/fail, no exit code other than "did the TUI start", no batch
mode, no assertion on the response.

### What it misses

All four classes from the brief, and for the same root cause: **the tool only ever looks at one endpoint
at a time, chosen interactively, and it never reasons across the endpoint set.** Every one of the four
defects is a *relational* property (this endpoint vs that one; this field vs this role), and the tool has
no place to express a relation.

| Defect class | Caught? | Why not |
|---|---|---|
| Object-level authz gaps (`LeaveSession`) | No | Requires comparing role sets across endpoints with matching signatures. Tool has no cross-endpoint pass. |
| Anon-reachable operator fields (`GetSessionInfo`) | No | `roles` is parsed into `Service`/`EndpointSchema` but **never read** (`rg 'roles' endpoint-validator/src` → 0 hits). Return types are never walked at all. |
| Caller-chosen identity (`guest_id`) | No | Parameters are treated as opaque name+type to render an input box. |
| Response struct duplication/drift | No | `structs` is deserialized into `Services.structs` (`models.rs:38`) and then **never used by any code path**. |

Note the shape of that table: the two fields that would power three of the four checks, `roles` and
`structs`, are already parsed and then thrown away. The distance from here to a useful linter is smaller
than the repo's state suggests.

---

## 2. Proposed checks

All four run on `services.json` alone, offline, no server. I suggest they land behind a new
`endpoint-validator lint --services-path <p>` subcommand that exits non-zero on findings, so it can be a
CI gate in the backends (which is where it needs to run, since that is where `services.json` is
generated).

### Check A: role-set asymmetry across identical parameter signatures

- **Rule:** group all endpoints by their parameter signature (the multiset of `(name, ty)` pairs, sorted).
  Within a group of 2+, if the `roles` sets are not all equal, report the group. The intuition: two
  endpoints that take exactly `(guest_id: UUID, session_id: NanoId(16))` are operating on the same object
  in the same way; if one is admin-only and the other is anon-callable, one of them is wrong.
- **RON annotation needed:** **none.** This is the check to build first.
- **Prototype result** on `nofilter.io-backend/docs/services.json` (61 endpoints): **2 groups flagged.**

  ```
  params ['sessionId']
     DeleteSession        [PlatformAdmin, StudioManager]
     ListWaitRoomGuests   [PlatformAdmin, StudioManager]
     GetParticipantPaths  [AnonGuest, PlatformAdmin, StudioGuest, StudioManager]
     GetSessionInfo       [AnonGuest, PlatformAdmin, StudioGuest, StudioManager]

  params ['guestId', 'sessionId']
     AcceptWaitRoomGuest  [PlatformAdmin, StudioManager]
     RejectWaitRoomGuest  [PlatformAdmin, StudioManager]
     LeaveSession         [AnonGuest, PlatformAdmin, StudioGuest, StudioManager]
     LeaveWaitRoom        [AnonGuest, PlatformAdmin, StudioGuest, StudioManager]
     Heartbeat, Join*, ListSessionMembers, WaitRoomHeartbeat  [ ...same anon set ]
  ```

  Group 2 is precisely the defect the sibling review found, surfaced with no code analysis whatsoever.
- **False-positive risk: low-to-moderate.** 2 groups out of 61 endpoints is a reviewable signal, not
  noise. The expected FP is a legitimate asymmetry (a `Join` genuinely should be anon while an `Accept`
  should not). Handle it with a suppression: `meta: {"authz_asymmetry": "intentional"}` on the endpoint,
  or a checked-in `lint-allow.toml` keyed by endpoint name. Prefer the allowlist file so suppressions are
  reviewable in one place.
- **Implementation difficulty: S.** ~120 lines. `Type` derives `Hash + Ord + Eq`
  (`endpoint-libs/src/model/types.rs:61`), so the signature key is `Vec<(String, Type)>` sorted, straight
  into a `BTreeMap`. No new dependency.

### Check B: sensitive field reachable from an anon or public role

- **Rule:** for each endpoint whose `roles` contains an anon-class role, walk `returns` transitively,
  resolving `StructRef(name)` and `StructTable{struct_ref}` against the top-level `structs` registry and
  descending through `Vec`/`Optional`/inline `Struct`. Report any reached field marked sensitive.
- **RON annotation needed:** yes, and **the slot already exists.** `Field.meta: MetaMap` is
  `#[serde(default, skip_serializing_if = "MetaMap::is_empty")]` in `endpoint-libs 2.x`
  (`src/model/types.rs:74-76`), documented as "Empty in 2.0", i.e. reserved and unused. So the RON becomes:

  ```ron
  Field(name: "castr_whip_url", ty: String, meta: {"sensitivity": "operator"}),
  ```

  with no upstream schema change at all, only an emitter that stops dropping it. Until that lands, fall
  back to a name heuristic (`whip`, `ingest`, `stream_key`, `secret`, `token`, `api_key`, `password`).
- **Prototype result** (name heuristic, since `meta` is empty today): **4 of 61 endpoints flagged.**

  ```
  ListUserSessions   41002  Session.castrWhipUrl, Session.cloudflareLiveWhipUrl, Session.restreamWhipUrl
  GetSessionInfo     41006  Session.castrWhipUrl, Session.cloudflareLiveWhipUrl, Session.restreamWhipUrl
  JoinPlainMoqSession  41003  <inline>.token
  JoinPlainMoqWaitRoom 41034  <inline>.token
  ```

  The `GetSessionInfo` hit is the sibling review's finding. **`ListUserSessions` (41002) is the same leak
  through the same `Session` struct and was not in the brief**: it is anon-reachable and returns
  `StructTable("Session")` whose fields include all three WHIP ingest URLs
  (`nofilter.io-backend/config/structs.ron:116-132`). Worth confirming independently.
- **False-positive risk: high on the name heuristic, near-zero with `meta`.** The two `token` hits are
  probably a legitimate per-join media token. This is exactly why the annotation is worth the effort: the
  heuristic is a bootstrapping tool to *find* the fields to annotate, then you switch the rule to
  `meta`-only and the FP rate goes to zero by construction.
- **Implementation difficulty: M.** ~180 lines. The struct-reference resolver is the only real work, and
  it needs a `seen: HashSet<&str>` guard because `structs.ron` permits cycles. Note `Type` is
  `#[non_exhaustive]`, so the walker needs a catch-all arm.

### Check C: caller-supplied identity parameter on an anon-reachable endpoint

- **Rule:** report any endpoint whose `roles` include an anon-class role **and** which takes a parameter
  naming a principal (`guest_id`/`guestId`, `user_id`, `account_id`, `owner_id`, or anything annotated
  `meta: {"identity": true}`), unless the endpoint is annotated as binding it.
- **RON annotation needed:** yes, for the *suppression* side. The detection can be name-based, but the
  "this one is fine, it is bound at handshake" exemption must be explicit or the check is unusable.
  Suggest `meta: {"identity": "connection_bound"}` on the parameter.
- **Expected volume:** **~20 of the 32 anon-reachable endpoints** in nofilter take a `guestId`. That is a
  lot of output for one systemic design decision.
- **False-positive risk: this one is different in kind.** Unlike A and B, this check does not find a bug
  per endpoint; it finds *one* architectural decision reported 20 times. Recommend it emits a **single
  aggregated finding** ("20 anon-reachable endpoints accept a caller-supplied identity parameter; see
  list") rather than 20 findings, or it will be suppressed wholesale on day one and never re-read.
- **Implementation difficulty: S** to implement, **L** to make actionable. The rule is trivial; the value
  depends entirely on getting the reporting granularity right. Build it after A and B.

### Check D: response struct duplication and drift

- **Rule:** canonicalise every struct shape to a sorted `Vec<(field_name, Type)>`. Compare (i) inline
  `Type::Struct` shapes appearing in `returns` against the named entries in the top-level `structs`
  registry, and (ii) registered structs against each other. Report exact structural matches under
  different names (duplication) and near-matches, defined as Jaccard similarity of the field-name sets
  above ~0.8 with any type mismatch on a shared name (drift).
- **RON annotation needed:** none for detection. A `lint-allow.toml` entry for deliberate look-alikes.
- **False-positive risk: moderate,** and it is tunable via the similarity threshold. Small structs are the
  problem: `{id, name}` and `{id, name}` will match constantly and mean nothing. Mitigate with a minimum
  field count of ~4 before near-match reporting; keep exact-match reporting at any size but rank it lower.
- **Implementation difficulty: M.** ~150 lines, no new dependency. Note this check only sees the *schema*
  side of the drift. The hand-written-vs-generated Rust type divergence the sibling review found is not
  visible in `services.json` at all, so this check catches the schema half of that problem and no more.
  Be explicit about that limitation when reporting, or it will create false confidence.

### Ordering recommendation

**A → B → D → C.** A is free (no annotation, immediate true positives), B is the highest-severity class
and its annotation slot already exists, D is self-contained, C needs a design conversation about
aggregation before it is worth writing.

---

## 3. How a new check is added today

There is nothing to critique, because **there are zero checks and zero scaffolding.** `rg` for a check
registry, a rule trait, a lint module, a diagnostic type across `endpoint-validator/src`: nothing. The
entire crate is 17 files totalling ~1,600 lines, of which ~1,100 are TUI.

So section 3 is a greenfield proposal rather than a refactor. The single-implementor-trait warning in the
brief applies here in advance: `ConvertValue` (`parser/services.rs:12-14`) is already a trait with exactly
one impl, and it is justified (orphan rule: `Type` is foreign, so an inherent impl is impossible, and the
comment says so). A `Check` trait would have four implementors, which clears the bar.

Sketch:

```rust
// src/lint/mod.rs
pub struct Finding {
    pub rule: &'static str,          // "authz-asymmetry"
    pub severity: Severity,
    pub endpoints: Vec<String>,      // for group findings
    pub message: String,             // one actionable sentence
}

pub trait Check {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &Ctx<'_>) -> Vec<Finding>;
}

/// Resolved once, shared by all checks: the struct registry as a lookup,
/// the enum registry, and a flat endpoint list with its owning service.
pub struct Ctx<'a> {
    pub services: &'a Services,
    pub structs: HashMap<&'a str, &'a [Field]>,
    pub endpoints: Vec<(&'a Service, &'a EndpointSchema)>,
    pub allow: &'a AllowList,
}

pub fn all() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(AuthzAsymmetry),
        Box::new(SensitiveFieldExposure),
        Box::new(CallerSuppliedIdentity),
        Box::new(StructDrift),
    ]
}
```

The `Ctx` is the part that actually saves work: **all four checks need the same struct-reference
resolver**, and building it once in `Ctx` is the difference between one 60-line resolver and four
copy-pasted ones. Build `Ctx` first, even if you only ship Check A.

Wire-up in `main.rs` as a `lint` subcommand: run all checks, print findings grouped by rule, exit `1` if
any finding at or above a `--fail-on` threshold (default: the sensitive-field and authz rules). That gives
the backends something to put in `.github/workflows/ci.yml` next to their existing codegen step.

---

## 4. Performance

Not a concern, and worth stating so nobody optimises it. The largest `services.json` in the family is
nofilter's at 118 KB / 61 endpoints; api.support.cafe is 35 KB / 26 endpoints. Check A is `O(n log n)`,
Check D is `O(n²)` on ~30 structs. All four checks together will run in single-digit milliseconds. Parsing
dominates and it is one `serde_json::from_reader`. **Do not** design for streaming or parallelism here.

One live-code note, unrelated to any CI gate: the TUI redraw task
(`tui/app.rs:41-52`) ticks every 500 ms and unconditionally acquires both the `AppState` and `Terminal`
mutexes to redraw whether or not anything changed, while `handle_event` (`app.rs:158-164`) *also* redraws
on every keypress. It is a manual tool, so this is cosmetic, but it is why keystrokes can feel laggy.

---

## Findings

### [SEV-1] Login handshake is dead protocol; the tool cannot connect to any current backend
- **ID:** `endpointvalidator-full-01`
- **Severity:** High
- **Category:** Correctness
- **Confidence:** High
- **Location:** `endpoint-validator/src/tui/state.rs:389-431`
- **What:** `handle_connect` performs a hardcoded two-step handshake, sending
  `Sec-WebSocket-Protocol: 0loginstep1, 1{username}`, reading an `accessToken` out of
  `params.accessToken`, reconnecting with `0loginstep2, 1{token}, 2{password}`. Grepping `loginstep`
  across `endpoint-libs`, `endpointgen`, `api.support.cafe`, `nofilter.io-backend` and
  `pays.online-backend` returns zero matches. The live protocol is a single `Init` endpoint taking
  `accessToken` (`api.support.cafe/docs/services.json` code 10000, `nofilter.io-backend` code 21001, both
  `roles: ["UserRole::Public"]`), passed at handshake as `0init, 1<token>` and read by
  `endpoint-libs/src/libs/ws/tungstenite/upgrader.rs:283-287`.
- **Why it matters:** the tool is the only interactive way to exercise these services and it is 100%
  non-functional against all three. The username/password fields in the TUI collect credentials for an
  auth flow that no longer exists. This also explains the "press enter a couple of times again" advice in
  `README.md:93`: that is a workaround for a broken connect path, documented rather than fixed.
- **Fix:** replace the two-step flow with a single connect using `0init, 1{token}`, and replace the
  Username/Password settings fields with a single Access Token field. The repo already contains a working
  reference: `ws-load-test/protocol_header_fly.txt:1` is `0init, 1d407c4ff-4cce-408b-8665-72342b1ec5bd`
  and `ws_simple.rs` uses it successfully. Mechanical once the decision is made; the only design question
  is whether to also support nofilter's `AuthGuest` (21002) path.
- **Effort:** M
- **Blast radius:** `state.rs` (`handle_connect`, `SettingsField`, `username`/`password`), `ui.rs:86-95`,
  `README.md:84-95`, `endpoint-validator/README.md:48-50`. No API break (this is a binary).

### [SEV-2] No static validation exists, and nothing downstream runs the tool
- **ID:** `endpointvalidator-full-02`
- **Severity:** High
- **Category:** Design
- **Confidence:** High
- **Location:** whole crate; `endpoint-validator/src/parser/models.rs:32-39`,
  `endpoint-validator/src/parser/services.rs:16-49`
- **What:** despite the name, there is no rule engine, no check, no pass/fail. `extract_endpoints`
  (`services.rs:17-48`) reads only `name`, `code`, `parameters` and `stream_response` off each endpoint,
  discarding `roles`, `returns` and `errors`. `Services.structs` (`models.rs:38`) is deserialized and
  never read by any code path. Meanwhile the three backends contain zero references to this tool
  (`rg -il 'endpoint.?validator'` across all three, including `.github/workflows/`), and **this repo has
  no `.github/` directory and therefore no CI whatsoever**. `AGENTS.md:52-68` documents CI-green gating as
  DORMANT because "CI here does not reliably attach checks to pull requests", which is accurate but
  understates it: there is no workflow to attach.
- **Why it matters:** four real security defects shipped in nofilter.io-backend that a schema-level linter
  would have caught at build time. The data needed to catch two of them is already parsed by this crate
  and discarded three lines later.
- **Fix:** add the `lint` subcommand and `Check` trait from section 3, starting with Check A. Then add a
  CI job in each backend that runs it against the freshly generated `docs/services.json`. Add a minimal
  `rust.yml` here too, matching `endpointgen`'s.
- **Effort:** L for the first two checks plus wiring; S for the CI file.
- **Blast radius:** new `src/lint/` module, `main.rs`, `cli/args.rs`. Additive, no break. CI changes in
  three backend repos are separate PRs.

### [SEV-3] Non-scalar `config.toml` defaults are Debug-formatted into garbage
- **ID:** `endpointvalidator-full-03`
- **Severity:** Medium
- **Category:** Correctness
- **Confidence:** High
- **Location:** `endpoint-validator/src/parser/services.rs:186-193`
- **What:** `extract_param_defaults` flattens each `ParamValue` to a `String`, and for the two composite
  variants it uses Rust's `Debug`:
  ```rust
  ParamValue::Array(arr) => format!("{:?}", arr),
  ParamValue::Object(obj) => format!("{:?}", obj),
  ```
  A config entry `params.tags = ["a", "b"]` becomes the literal string `[String("a"), String("b")]`. That
  string is then handed to `Type::convert_value`, which for a `Vec(String)` will happily produce
  `["String(\"a\")", "String(\"b\")"]`, and for an `Object`/`StructRef` will fail JSON parsing.
- **Why it matters:** silently wrong request payloads for exactly the composite parameters a human least
  wants to retype, which is the entire reason `config.toml` exists. It fails as a bad request from the
  server rather than as a config error naming the line, so it reads as a server bug.
- **Fix:** convert `ParamValue` to `serde_json::Value` rather than to `String`, and give
  `ConvertValue` a `convert_json(&self, v: &Value)` path alongside the string path. Mechanical but touches
  the `param_values: Vec<String>` assumption in `AppState` (`state.rs:51`), so it needs a small design
  decision about whether the TUI edits strings or JSON.
- **Effort:** M
- **Blast radius:** `parser/services.rs`, `parser/models.rs`, `tui/state.rs`, `tui/ui.rs:223-232`.

### [SEV-4] 273-line byte-identical dead duplicate of the UI module, plus two more dead binaries
- **ID:** `endpointvalidator-full-04`
- **Severity:** Medium
- **Category:** AI-smell
- **Confidence:** High
- **Location:** `endpoint-validator/src/tui/ui/ratatui_legacy.rs` (273 lines),
  `endpoint-validator/src/tui/ui/tui_realm.rs` (0 bytes),
  `ws-load-test/src/bin/ws_load_tester.rs` (6 lines)
- **What:** `src/tui/ui.rs` and `src/tui/ui/` coexist. `diff src/tui/ui.rs src/tui/ui/ratatui_legacy.rs`
  reports **a single hunk: the position of the `Frame` import.** Otherwise byte-identical. Neither file in
  `ui/` is declared as a module anywhere (`tui/mod.rs:3` is just `pub mod ui;`, and `ui.rs` declares no
  submodules), so both are dead weight that `cargo` never compiles: clippy reports no warnings from them
  while reporting 17 from the rest of the lib. Separately, `ws_load_tester.rs` is a `// TODO` comment plus
  an empty `main`, and because it lives in `src/bin/` cargo auto-discovers it: `cargo run` at the
  workspace root lists three binaries including `ws_load_tester`, and `target/debug/ws_load_tester` is a
  real, empty, shipped executable.
- **Why it matters:** the next agent asked to change the request pane has a coin-flip chance of editing
  the dead copy, and nothing, not the compiler, not clippy, not the tests, will say a word. The
  `tui_realm.rs` name also implies a migration to `tui-realm` that never happened and is not in any doc.
- **Fix:** `git rm endpoint-validator/src/tui/ui/ratatui_legacy.rs endpoint-validator/src/tui/ui/tui_realm.rs ws-load-test/src/bin/ws_load_tester.rs`.
  Purely mechanical. If the tui-realm migration is still wanted, it belongs in an issue, not a 0-byte file.
- **Effort:** S
- **Blast radius:** none. Nothing references any of the three.

### [SEV-5] Reachable panics on an empty or unselected endpoint set
- **ID:** `endpointvalidator-full-05`
- **Severity:** Medium
- **Category:** Correctness
- **Confidence:** Medium (needs a human to confirm the TUI can reach both states)
- **Location:** `endpoint-validator/src/tui/state.rs:271`, `endpoint-validator/src/tui/state.rs:459`
- **What:** two spots.
  1. `select_next_endpoint` guards with `if self.selected_endpoint < self.endpoints.len() - 1`. With an
     empty `endpoints` vec, `0usize - 1` panics in debug and wraps to `usize::MAX` in release, making the
     comparison true and letting `selected_endpoint` increment unbounded. A `services.json` with no
     `frontend_facing` endpoints reaches this, since `connected` is set by `handle_connect` independently
     of the endpoint list.
  2. `handle_endpoint_connect` calls `self.method_id.unwrap()` when sending. `method_id` is `None` until
     `update_selected_endpoint_data` runs, which only happens inside `switch_block`
     (`state.rs:251-266`) and the two selection helpers, all gated on `connected`.
  Also note `widgets/list.rs:8` slices `&items[selected..]` with no bound check; it is currently safe only
  because of the guard in (1), so fixing (1) carelessly could turn this into a slice panic.
- **Why it matters:** a crash in raw terminal mode leaves the terminal in a broken state, because the
  cleanup at `app.rs:69-78` only runs on the normal Esc path.
- **Fix:** `if self.selected_endpoint + 1 < self.endpoints.len()`, and replace the `unwrap` with the same
  `.context("no endpoint selected")?` pattern used three lines above it at `state.rs:444-447`. Mechanical.
- **Effort:** S
- **Blast radius:** `state.rs` only.

### [SEV-6] Docs make claims the code contradicts, across all six markdown files
- **ID:** `endpointvalidator-full-06`
- **Severity:** Medium
- **Category:** Docs
- **Confidence:** High
- **Location:** see table
- **What:** `AGENTS.md:26` states "Docs describe what is true now." Verified against the code, they do not.

| Doc | Line | Claim | Reality |
|---|---|---|---|
| `README.md` | 78 | `cargo run -- --services-path ...` from the Validator root | Fails: `cargo run` could not determine which binary to run. Three bins exist (`endpoint-validator`, `ws-simple`, `ws_load_tester`). Needs `-p endpoint-validator` or a `default-run` key. |
| `README.md` | 28 | "`services.json` contains a JSON list of serialized `EndpointData` structs, as defined in models.rs" | Wrong twice. It is an *object* with `enums`/`services`/`structs` (`models.rs:32-39`), and `EndpointData` (`models.rs:63-68`) is the **`config.toml`** type, unrelated. |
| `README.md` | 49, 52 | `--output-dir gen/`, "services.json should be generated in the `gen/` directory" | All three backends have it at `docs/services.json`; `endpoint-validator/README.md:32` says `--output-dir .`. The root README is stale. |
| `README.md` | 84-95 | Username → Password → Connect walkthrough | The auth flow it describes does not exist server-side. See SEV-1. |
| `README.md` | 38 | "TODO: The instructions below should be moved to..." | A TODO shipped in user-facing docs. |
| `endpoint-validator/README.md` | 19 | "Both paths are prompted for interactively if omitted" | `main.rs:9-15` matches `(Some, Some)`; supplying **one** path discards it and re-prompts for **both**. See Nits. |
| `AGENTS.md` | 25 | "Keep `cargo fmt` and `cargo clippy --all-targets` clean. Lint failures are part of the build here, not advisory." | `cargo fmt --check` is clean. `cargo clippy --all-targets` emits **31 warnings** (17 lib, 14 ws-load-test), including 8 collapsible-if, 4 redundant-field-names, 2 too-many-arguments, and 6 unused-assignment. |
| `ws-load-test/WS_SIMPLE_README.md` | 3 | "hardcoded defaults in `src/bin/ws-simple.rs`" | File is `src/bin/ws_simple.rs` (underscore). Only the *binary* is hyphenated. |
| `ws-load-test/COMPARE_RESULTS.md` | 27 | `python ../../endpoint-optimizer/ws-load-test/compare_results.py` | No `endpoint-optimizer` repo. Path is a leftover from a rename. |
| `docs/auth.ron` | 30 | `ty: BigInt` | `BigInt` does not exist in `endpoint_libs::model::Type`. `endpoint-validator/README.md:62-63` says so explicitly. |
| `docs/enums.ron` | 12, 17, 22 | `comment: ""` on `EnumVariant` | Renamed to `description` upstream. This is the *exact* rot that `endpoint-validator/README.md:58-61` describes as having broken the tool. The example files were never updated. |

- **Why it matters:** the two `.ron` examples are the ones that actually bite. They are the only worked
  examples a newcomer has, they are linked from `README.md:36`, and they would fail to parse with the
  current `endpoint-gen`. A reader who trusts them writes a broken RON and blames the toolchain. That the
  crate README documents this exact class of rot while shipping two instances of it is the sharpest
  version of the problem.
- **Fix:** regenerate `docs/auth.ron` and `docs/enums.ron` against current `endpoint-libs` types
  (`BigInt` → `Int64`, `comment:` → `description:`), or replace both with a symlink/excerpt of a real
  backend's RON. Correct the ten table rows above. Note `docs/` and `endpoint-validator/docs/` are
  **byte-identical duplicates** (`diff -r` reports no differences), so every fix must be applied twice or
  one copy deleted; delete the root copy, since only the crate copy is packaged.
- **Effort:** M
- **Blast radius:** docs only.

### [SEV-7] Credentials handled in cleartext and live tokens committed
- **ID:** `endpointvalidator-full-07`
- **Severity:** Low
- **Category:** Security
- **Confidence:** High
- **Location:** `endpoint-validator/src/tui/ui.rs:91-95`,
  `ws-load-test/protocol_header_bunny.txt:1`, `ws-load-test/protocol_header_fly.txt:1`
- **What:** the Password field is rendered with the same `create_input_widget` as URL and Username, so it
  echoes in cleartext on screen and stays in the alternate-screen scrollback. Separately, two committed
  files contain what look like live honey.id access tokens
  (`0init, 1b002e6b1-0fc9-4313-9dc5-9cf9b8b94876` and `0init, 1d407c4ff-4cce-408b-8665-72342b1ec5bd`);
  `example_protocol_header.txt` correctly contains a placeholder, which suggests the other two were meant
  to be local-only.
- **Why it matters:** low, because these are load-test tokens for benchmark environments, not production
  user credentials, and the password field is moot until SEV-1 is fixed. But committed bearer tokens do
  not expire on their own and `.gitignore` does not cover these files.
- **Fix:** mask the password field (render `"•".repeat(n)`); confirm with the owner whether those two
  tokens are live and revoke if so, then gitignore `protocol_header_*.txt` except the example. If SEV-1 is
  fixed by replacing the field with an access token, mask that instead.
- **Effort:** S
- **Blast radius:** `ui.rs`, `.gitignore`, two data files.

### [SEV-8] `split_top_level` mis-parses escaped quotes and unbalanced brackets
- **ID:** `endpointvalidator-full-08`
- **Severity:** Low
- **Category:** Correctness
- **Confidence:** High
- **Location:** `endpoint-validator/src/parser/services.rs:161-177`
- **What:** the depth/quote scanner toggles `quoted` on every `"` with no escape handling, so a value
  containing `\"` inverts quote state for the rest of the string. `depth` is an `i32` that can go negative
  on an unbalanced `}`, after which a `depth == 0` split point is never found again and the whole
  remainder is returned as one part. The trailing filter `.filter(|p| !p.is_empty())` also means a
  deliberately empty element in a `Vec` is dropped rather than becoming `null`.
- **Why it matters:** minor, and only for hand-written composite config values, which SEV-3 shows are
  already broken by a different bug. But it fails silently: you get a wrong-shaped request, not an error.
- **Fix:** track a `escaped` flag alongside `quoted`, and `bail!` if `depth` goes negative or is non-zero
  at the end. The function is well-commented and self-contained; this is a 10-line change plus two unit
  tests. Note there are currently **no unit tests on this function at all** despite it being the trickiest
  parsing in the crate; the four tests in `tests/parses_generated_services.rs` cover scalars, enums and
  the fixture, but never a nested struct or a `Vec`.
- **Effort:** S
- **Blast radius:** `parser/services.rs` only.

---

## AI-smell inventory

Mostly clean, with the notable exception of SEV-4. Specifics:

- **Duplication:** the 273-line dead `ratatui_legacy.rs` (SEV-4) is the big one. Beyond it, the four
  widget helpers (`button.rs`, `input.rs`, `list.rs`, `json_viewer.rs`) each open with an identical
  9-line focused/unfocused border-colour block; clippy flags `input.rs:17-21` as
  "this `if` has identical blocks" because both branches return `Color::Gray`, which is a copy-paste
  artefact of that block. A `fn border(is_focused: bool) -> Style` helper removes ~40 lines.
- **Dead code:** `ratatui_legacy.rs`, `tui_realm.rs` (0 bytes), `ws_load_tester.rs` (stub bin),
  `Services.structs` (parsed, never read), `EndpointSchema.roles`/`.returns`/`.errors` (parsed, never
  read), `List::highlight_style` in `widgets/list.rs:42-46` (dead: the list is drawn with
  `render_widget`, not `render_stateful_widget`, so no `ListState` and no highlight ever applies).
- **Single-implementor traits:** one, `ConvertValue` (`services.rs:12-14`), and it is **justified** by the
  orphan rule with a comment saying exactly that. Not a smell.
- **Comments restating code:** a handful of the low-value kind, e.g. `main.rs:5` "Parse command-line
  arguments" above `cli::parse_args()`, `state.rs:93` "Scroll logic for the response section",
  `app.rs:24` "Set up terminal in raw mode". Minor.
- **Defensive scaffolding for impossible states:** `state.rs:354-363` sets `self.connected = false` on a
  failed connect and `self.connected = true` on a failed disconnect, restoring state that a failing
  operation should not have changed. Harmless but reads as guessing.
- **TODO inventory:** exactly 3, all real and all noted above: `README.md:38`,
  `ws_load_tester.rs:1`, `ws_simple.rs:461` ("TODO: Classify this separately").
- **Do the tests exercise the checks or assert on mocks?** They exercise real behaviour, and this is the
  best thing in the repo. `tests/parses_generated_services.rs` parses a **real generated file** (the
  fixture is byte-identical to `api.support.cafe/docs/services.json`, both 34,710 bytes, so it is current)
  and derives its expectations from the fixture rather than hardcoding them
  (`enum_variants_convert_to_their_wire_integer` at lines 47-73 pulls the first enum out of the fixture
  and asserts the round-trip). No mocks, no `assert!(true)`. The gap is coverage, not quality: 4 tests,
  zero on `split_top_level`, zero on `Type::Struct`/`Vec` conversion, zero on `extract_param_defaults`
  (which is where SEV-3 lives), and zero on any TUI state transition.
- **Inconsistent style:** `state.rs:354` and following use `_err` as a bound name and then *use* it
  (`self.json_data = Some(_err.to_string())`), which is backwards from the leading-underscore convention.
  Cosmetic.

---

## Cross-cutting recommendations

1. **Decide what this repo is, and rename accordingly.** Right now one crate is a broken manual TUI and
   the other is a working benchmark harness, sharing a workspace and a name that describes neither. The
   highest-value version is a *schema linter* (section 2) with the TUI as a secondary mode. If the TUI is
   not going to be fixed, say so in the README and stop calling the repo a validator. **Breaks:** nothing
   technical; `endpoint-libs/scripts/check-chain.sh:18` and `docs/chain.md:16` reference the name and
   would need updating.

2. **Build `Ctx` + `Check` + Check A, and put it in the backends' CI.** This is the concrete next task and
   it is scoped to about a day. Order: `src/lint/mod.rs` with the struct-resolver `Ctx`, then
   `AuthzAsymmetry`, then a `lint` subcommand with a non-zero exit, then a CI step in each backend that
   runs it on the generated `docs/services.json`. It found two true positives in nofilter on the first
   prototype run. **Breaks:** nothing, purely additive; the backends' CI will go red on first run, which
   is the point, so land the allowlist mechanism in the same change.

3. **Fix or remove the handshake (SEV-1) before anything else touches the TUI.** Every TUI improvement is
   unverifiable while the tool cannot connect. If the decision is to keep the TUI, this is a prerequisite;
   if it is to drop it, deleting ~1,100 lines of `tui/` removes the ratatui 0.20 dependency, which is the
   reason `crossterm` is compiled twice (0.26.1 pulled by ratatui, 0.28.1 direct: see `Cargo.lock:842`
   and `:858`). **Breaks:** the README walkthrough either way.

4. **Push the sensitivity annotation upstream into `endpoint-libs`' emitter.** `Field.meta` exists and is
   documented as empty; getting `endpoint-gen` to round-trip `meta` from RON into `services.json` is a
   small upstream change that unlocks Check B at near-zero false positives and is reusable by every future
   check. This is the one item with a cross-repo dependency, so start the conversation early.
   **Breaks:** requires an `endpoint-libs` + `endpoint-gen` release, and per `chain.md:85-93` this repo is
   already known-red on version alignment (two `endpoint-libs` majors, 1.9.1 and 2.1.1, because
   `ws-load-test` is pinned to 1.x). Sequence it against that.

5. **Delete the dead files and get clippy to zero.** SEV-4 plus the 31 clippy warnings, most of which are
   auto-fixable (`cargo clippy --fix` offers 22 of them). This makes `AGENTS.md:25` true again and means
   the next real warning is visible. Do this first; it is an hour and it de-risks everything else.
   **Breaks:** nothing.

---

## What I did not cover

- **I did not run the TUI.** No terminal capture available, and per SEV-1 there is no reachable server to
  connect it to anyway. All TUI findings are from reading, and SEV-5 is marked Medium confidence for that
  reason.
- **`ws-load-test/src/bin/ws_simple.rs` (770 lines) got a structural skim, not a line-by-line read.** It
  is a benchmark harness, out of the brief's focus, and its manifest warns its threading behaviour backs
  published measurements. I confirmed it compiles, noted its 14 clippy warnings and its one TODO, and
  confirmed it uses the *current* handshake format, which is what made SEV-1 provable. Its concurrency
  model, histogram accounting and the correctness of its latency numbers are unreviewed.
- **`compare_results.py` was skimmed for doc-accuracy only**, not reviewed for correctness.
- **I did not verify the sibling reviews' findings in the backend source.** I confirmed the *schema-level*
  shape of three of them in `nofilter.io-backend/config/*.ron` and `docs/services.json`, which is all the
  proposed checks operate on. Whether `LeaveSession`'s handler really lacks an ownership check is the
  sibling reviewer's finding, taken as given. My `ListUserSessions` (41002) observation is new and
  schema-derived; **it needs a human to confirm the handler actually returns those fields** before being
  treated as a second confirmed leak.
- **Check C and Check D were designed but not prototyped.** Only A and B have measured false-positive
  numbers. C's "~20 endpoints" figure is a count of anon-reachable endpoints taking `guestId`, not a
  prototype run.
- **No review of `.claude/hooks/ask-before-risky-commands.sh`** beyond noting it exists and that
  `CLAUDE.md:12` asks for it to stay in sync with `permissions.ask`. I did not verify that sync.

---

## Quick-start for the follow-up agent

**Read in this order:**

1. `endpoint-validator/src/parser/models.rs`: 78 lines, the data model, and the doc comment at the top
   explains the one historical bug that shaped this crate. Note lines 32-39: `structs` and the endpoints'
   `roles` are right there, unused.
2. `endpoint-validator/src/parser/services.rs`: the only substantial non-TUI logic, and the best-written
   file here. `extract_endpoints` (17-48) is where the new lint pass hangs off.
3. `endpoint-libs/src/model/{endpoint.rs,types.rs}` (read-only, sibling repo): `EndpointSchema.roles` at
   `endpoint.rs:38` and `Field.meta`/`MetaMap` at `types.rs:16-77`. This is the schema every proposed
   check reads.
4. `endpoint-validator/tests/parses_generated_services.rs`: the testing pattern to copy: real fixture,
   expectations derived from it.
5. `endpoint-validator/src/tui/state.rs:389-431`: the dead handshake (SEV-1), if you are fixing that.
6. `nofilter.io-backend/config/schema_lists/03_studio.ron:398-465` (read-only): `LeaveWaitRoom` at 398
   and `AcceptWaitRoomGuest` at 447, side by side: identical parameters, different `roles`. This is what
   Check A detects, and reading it makes the rule obvious.

**Commands:**

```bash
# Build and test. Note -p: `cargo run` at the root is ambiguous across 3 bins.
cargo build
cargo test -p endpoint-validator          # 4 tests, all should pass, <1s
cargo run -p endpoint-validator -- --help

# Lint. fmt is clean; clippy is NOT (31 warnings) despite AGENTS.md:25.
cargo fmt --all -- --check
cargo clippy --all-targets

# Real inputs to develop the lint against (read-only, do not modify):
#   nofilter.io-backend/docs/services.json    118 KB, 61 endpoints, 32 anon-reachable  <- richest
#   pays.online-backend/docs/services.json     69 KB
#   api.support.cafe/docs/services.json        35 KB, 26 endpoints (== the test fixture)

# Cross-repo version check (from the endpoint-libs checkout):
../endpoint-libs/scripts/check-chain.sh   # expect RED on check 1, see chain.md:85
```

**Surprising things about the layout:**

- `src/tui/ui.rs` **and** `src/tui/ui/` both exist, and everything in the directory is dead. Confirm which
  file you are editing (`ui.rs`, always).
- `docs/` and `endpoint-validator/docs/` are byte-identical duplicates. Edits must be doubled or one copy
  deleted.
- The test fixture is byte-identical to `api.support.cafe/docs/services.json` (34,710 bytes both), so it
  is current, not a snapshot that drifted. Refresh it from there.
- `endpoint-libs` appears **twice** in `Cargo.lock` (1.9.1 and 2.1.1) on purpose:
  `ws-load-test/Cargo.toml:8-11` explains the pin and `endpoint-libs/docs/chain.md:85-93` records it as
  known-red. Do not "fix" it casually.
- The workspace has three binaries, one of which (`ws_load_tester`) is an auto-discovered empty stub not
  declared in any `[[bin]]`. Bare `cargo run` therefore fails.
- `AGENTS.md` is the working agreement and `CLAUDE.md` just imports it. Read `AGENTS.md`, not `CLAUDE.md`.
