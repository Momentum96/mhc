# Dori sync backlog: senpi + oh-my-openagent past the mhc pins

Status: analysis only, measured 2026-11-22 from full (non-partial) clones in
`/Volumes/Work/mac_work/upstream/{senpi,oh-my-openagent}`. Nothing in mhc was built, changed or committed.
Every count below was measured with `git log --no-merges --numstat --no-renames` over the stated range unless it
is marked *estimate*.

How the numbers are counted:

- "Commits" are non-merge commits that touch the group. One commit that touches several groups is counted in
  each, so group counts do not sum to the range total.
- "TS src lines" are added/deleted lines in non-test `.ts/.tsx/.mts/.cts` files. Test lines are files matching
  `*.test.*`, `*.spec.*`, `test/`, `tests/`, `__tests__/`, `fixtures/`, `bench/`, `*.snap`. Docs/CI lines are
  `*.md`, `docs/`, `.github/`. These are counted, but they are not port work.
- Groups were mapped to crates with `SOURCE_ROOTS` in `tools/parity-audit.mjs` (senpi side), the builtin and omo
  component lists in the same file, and the TS paths recorded in `crates/**/parity.d/*.md` (omo side).

## 1. Upstream ranges

| Source | Range | Commits (all) | Merges | Non-merge |
|---|---|---:|---:|---:|
| senpi | `59d58cee5` (v2026.10.27-2, PINS.md) .. `99a26322f` (tag `v2026.11.19-5`, 2026-11-19 19:01 UTC) | 6751 | 2924 | 3827 |
| senpi, past the tag | `v2026.11.19-5` .. `6d91e370c` (origin/main, 2026-11-22) | 191 | 59 | 132 |
| oh-my-openagent | `354bcf9fc4` (v5.1.52, PINS.md) .. `5835a98e8` (tag `v5.1.81`, 2026-11-21) | 8438 | 3109 | 5329 |
| oh-my-openagent, past the tag | `v5.1.81` .. `origin/dev` | 32 | - | - |

- The senpi target is not a guess: `packages/omo-senpi/package.json`, `packages/omo-native/package.json` and the
  root `package.json` at `v5.1.81` all depend on `@code-yeongyu/senpi` `2026.11.19-5`. At `v5.1.52` the same
  fields said `2026.10.27-2`, which is the mhc senpi pin, so the two pins in PINS.md are a matched pair today.
- The pin is an ancestor of the tag (`git merge-base --is-ancestor` passes).
- senpi tags after `v2026.11.19-5`: `v2026.11.20`, `v2026.11.21`, `v2026.11.21-2`, `v2026.11.22`, `v2026.11.22-2`.
  Those 132 non-merge commits put +2201/-593 TS lines into S01 (rpc), +763/-625 into S17 (other builtins),
  +253/-104 into S03 (core), and almost nothing anywhere else. They belong to the next omo release, not to 5.1.81.
- One omo commit is pure formatting: `d3b07955b chore(fmt): adopt bun fmt for JS/TS (Fixes #12225)`, 184,258 TS
  lines (596,495 lines in all files). It is excluded from every omo line count below and kept in the commit
  counts. Its merge, PR#12227, shows up as the largest PR in O13. In mhc it maps to nothing.
- Range totals without that commit: senpi +146,534/-35,988 TS src lines; omo +144,474/-40,830 TS src lines.
  1759 of the 3827 senpi non-merge commits and 1757 of the 5329 omo non-merge commits touch only tests, docs,
  CI or `changes/` files.
- File counts: senpi non-test TS files went from 3100 (pin) to 3914 (829 added, 15 deleted). omo non-test TS
  files went from 5119 to 5780 (745 added, 84 deleted, mostly in `web` and `omo-opencode`).

## 2 and 3. Changes by area, with Dori relevance

Relevance: HIGH means the group touches something Dori uses (host session verbs, `omo -p`, `--session`,
task/subagents/categories, eval kernels, monitors and bash background sessions, memory tool and projection,
ulw/ultrawork, Claude CLI accounts and ChatGPT subscription, MCP, websearch/webfetch, LSP, rpc host daemon and
session lifecycle). LOW means desktop/computer-use, TUI cosmetics, Windows-only, telemetry or docs. MED is my
label for groups that touch a HIGH surface only indirectly, or only partly.

### senpi (`59d58cee5..v2026.11.19-5`)

| Group | Commits | TS src lines (+/-) | Test lines | Docs/CI lines | mhc crate(s) | Dori |
|---|---:|---:|---:|---:|---|---|
| S01 rpc host daemon, session worker, connection protocol | 584 | +36239 / -9220 | 319 | 5065 | maho-rpc | HIGH |
| S02 rpc compact-recovery / compaction admissions | 130 | +12894 / -2293 | 0 | 0 | maho-rpc | HIGH |
| S03 core agent-session, session store/move/import, prompts, retry | 414 | +11325 / -2983 | 568 | 4842 | maho-core | HIGH |
| S04 core credentials, auth pool, egress, trust | 178 | +10586 / -2666 | 0 | 364 | maho-core | MED (pool/accounts HIGH, custody/egress LOW) |
| S05 core execution plane, kill-guard, kernels, permissions | 76 | +5880 / -1694 | 0 | 0 | maho-core | HIGH (kill-guard, prepared pipeline) |
| S06 agent package (durable store, remote execution, wake inbox/outbox) | 246 | +17629 / -1632 | 8637 | 1846 | maho-agent | HIGH (durable store, agent-loop); hand/brain split MED |
| S07 ai package (providers, OAuth, auth pool) | 262 | +3825 / -2057 | 8067 | 2663 | maho-ai | HIGH |
| S08 builtin anthropic-subscription (Claude CLI accounts) | 137 | +4176 / -1496 | 0 | 1183 | maho-ext-anthropic-subscription | HIGH |
| S09 core tools (bash/read/edit/write) + new machine-ops package | 65 | +6061 / -2465 | 1136 | 376 | maho-tools (machine-ops: no crate) | HIGH |
| S10 senpi-codemode (eval py/js kernels) | 252 | +3166 / -497 | 22719 | 3634 | maho-codemode | HIGH |
| S11 extension host/api + builtin loose files | 154 | +2887 / -579 | 0 | 1962 | maho-ext-host, maho-ext-api, maho-ext-builtin-loose | HIGH |
| S12 builtin terminal (bash sessions, monitors) | 40 | +1726 / -325 | 49 | 350 | maho-ext-terminal | HIGH |
| S13 builtin mcp | 34 | +1467 / -487 | 0 | 244 | maho-ext-mcp | HIGH |
| S14 builtin webfetch/websearch | 35 | +2209 / -423 | 0 | 360 | maho-ext-webfetch, maho-ext-websearch | HIGH |
| S15 builtin permission-system | 43 | +3710 / -1186 | 0 | 192 | maho-ext-permission-system | MED |
| S16 new builtins without a crate (job-widgets, kill-guard, credential-proxy) | 31 | +6193 / -1250 | 0 | 0 | none | kill-guard HIGH; job-widgets LOW; credential-proxy MED |
| S17 other builtins (prompt-preset, goal, compaction, todotools, html-render, imagegen, hooks, ...) | 173 | +5446 / -2513 | 0 | 1346 | maho-ext-<name> | MED (compaction, goal, prompt-preset) |
| S18 interactive mode (attach, session-control commands) | 118 | +2918 / -662 | 488 | 982 | maho-interactive | LOW |
| S19 tui package | 94 | +1550 / -134 | 1488 | 756 | maho-tui | LOW |
| S20 cli/main, private-fds, engine-trust launch, utils | 139 | +4130 / -1070 | 0 | 1578 | maho-cli | HIGH (main, session opening, private fds) |
| S21 server/client/protocol/session-backends | 98 | +1211 / -60 | 891 | 607 | maho-server | LOW |
| S22 pty, telemetry, chord, realm-registry, evals | 47 | +397 / -83 | 882 | 27 | vendor/maho-pty, maho-telemetry, maho-chord | MED (pty only) |
| S99 repo infra, tests, docs, changes/, scripts, CI | 3383 | +909 / -213 | 226857 | 81862 | n/a | counted only |

Notable changes per group (PR merge titles first, ranked by TS lines; `#` numbers are senpi PRs):

- **S01 rpc host daemon** (129 PR merges). #4668 host payload permits and fragmented worker transfers for
  engine-socket workers; #5200/#4602 bounded startup policy (presenter before publication, owner-token fence,
  bounded startup controller wired into `startHost`); #6350 park a built session and hand it out through
  `claim_session`; #6755/#6752 `machine_op` engine command (relay-host device ops run in the daemon); #6126
  interactive role over the relay; #4530 rebuild legacy display state for a late peer; #6532 edit project MCP
  servers and skill files over RPC; #4957 one bounded schema per inbound RPC record; #6767 durable wake outbox
  for a hand's late results; #6815 quarantine rules for a worker that keeps answering progress. Largest new files:
  `runtime-factory-loopless.ts`, `session-event-writer.ts`, `loopless-tool-handlers.ts`,
  `delegated-connection.ts`, `endpoint-reservation-store.ts`, `socket-event-fanout.ts`.
  `cli/host-session-command.ts` (the `omo host session ...` verbs) did not change in the range and
  `cli/host-command.ts` changed by 4 lines. The CLI surface Dori calls is stable; the daemon behind it is not.
- **S02 compact recovery** (#4272 stack, 22 PR merges): compact store, message/call/identity adapters with a
  catalog completeness gate, recovery manifest and pages, admissions routed through compact frames,
  outstanding-call phase machine. This is a new subsystem with no port yet.
- **S03 core session**: #5907 the S20 durable session store lands on main (SessionManager store-first writes,
  JSONL projection); #5020 import JSONL tails with durable provenance; #6456/#6435/#6421 `move_session` (refuse a
  move that cannot land, lock-loss rules); #5240 nested tool calls carry their root issuer; #5682 a
  `session_lifecycle` event sits right before the agent event that caused it; #5394 `runtimeItemKey` persisted
  on toolCall/toolResult; #5613 retry-fallback weekly limit probes another family; #5665 Opus/Fable presets
  render full shared sections. `agent-session.ts` alone has 2893 changed lines across 134 commits.
- **S04 credentials/egress**: custody CK3-CK11 (sealed custody format, roster, DeviceProxyEgress and the
  `egress_call` ledger); #6828 credential injection proxy for child processes; #5027 credential-pool leases;
  `credential-accounts.ts` and `auth-storage.ts` changes. Custody and egress serve hosted or relay execution.
  The pool and account changes reach Claude/ChatGPT account failover.
- **S05 execution/kill-guard**: #5358 and `f79fe4b36`: kill-guard judges `kill/pkill/killall` at run time through
  PATH shims and a `BASH_ENV` wrapper, and refuses agent signals that hit protected processes. The prepared
  pipeline becomes an opt-in session ToolHost (#5657, later the default when the legacy scheduler is removed in
  S36/#5738); #6204 `set_permission_preset`.
- **S06 agent package**: durable execution ledger (#5004), DurableStore contract plus memory and Bun SQLite
  backends, wake inbox/outbox (#5679), scope lifecycle (#5683), batch coordinator, "hand" remote execution and
  conformance suites (S08-S40), #5738 legacy scheduler removed in favor of the prepared pipeline, #5741 loop
  hooks run on the user's machine.
- **S07 ai**: #5136 removes the Claude Code OAuth impersonation path and refuses `sk-ant-oat` tokens on the
  `anthropic` provider; `b0f46d620` removes the hand-rolled Anthropic OAuth (PKCE) flow (#4436); Sign in with
  ChatGPT (SIWC): login choice (#5081), ID-token validation (#5012), revoke on logout (#5025), per-account model
  catalog (#5078), plan-usage error classification (#5077), re-login replaces instead of appending (#5009);
  #5589 recover Anthropic sessions whose replayed thinking is bound to another conversation.
- **S08 anthropic-subscription**: #4437/#4522 Claude CLI login stack (`claude auth login/status/logout` driven
  through the bundled CLI, CLI-store account slots with per-account config dirs, per-slot claude-cli lane);
  #5353 signed-out login heals itself; #6412 `set_model` no longer waits for Claude Code's confirmation; #6730
  recheck stale limits on person turns; #6333 prompt-cache lifetime from settings.
- **S09 tools**: #6740 worker-safe cores for bash/read/write/edit (`*-core.ts`, `output-accumulator*.ts`);
  #5603 new `packages/machine-ops` engine-free file/process layer; #6713 slice 1a, an executor seam for bash on
  the session's device (`OMO_TOOLS_VIA_DAEMON=bash`, default off); #7056 tool children get
  `oom_score_adj=1000` on Linux.
- **S10 codemode**: #4387/#4392/#4391 permission checks for JS and Python eval kernels; #6168 nested tool partial
  results stream into the cell; #6422 realm registries and `pi.engine` (omo#12965); #6095 JS kernel defaults to
  process isolation on Bun hosts; kernels per execution scope (S16b).
- **S11 extension host/api**: two-sided extensions forward tool events from hand to loop (S29); optional `runs`
  declaration (S28); #7069 a tool hook handler cannot hold the tool loop (deadline for `tool_result`); #4438
  `provider_login_start` event; `pi.getMcpServers` / `unregisterMcpServer` / `mcp_servers_change`.
- **S12 terminal**: #6212 report long-running background sessions, opt-in auto-reap; #6151 monitor rpc targets
  on another device (`<device>/<session>`); #6675 interrupt can stop the interrupted turn's background jobs;
  #4718 stop the monitor notifier when its context is retired.
- **S13 mcp**: #6366 OAuth grant continuity across token refresh; #6328 dynamic registration reuse rules; #4850
  one `loaded_surfaces_changed` after startup connects; #5247 MCP without a session (RT5b); scope routing (S16c).
- **S14 webfetch/websearch**: #5957 render app-shell pages through the user's connected browser, incognito;
  websearch adds a `public` fan-out provider over keyless engines, TinyFish, Parallel and OpenRouter grounding.
- **S15 permission-system**: path identity canonicalizer and off-thread resolver worker; eval-cell operation
  decisions; unanswered prompts end when the turn aborts (#6676).
- **S16 new builtins**: `job-widgets` (P1-P4: spec, validator, live widgets, `job_widget_state` RPC event, TUI
  lines), `kill-guard` (the builtin half of S05), `credential-proxy`.
- **S17 other builtins**: prompt presets for GPT-6.2 Astra/Sol (#6962/#6979), Claude Fable 5.2 (#6314), DeepSeek
  V4.1 Flash; `imagegen` GPT Image 2.6 with a 2.5 fallback (#5821); goal resumes after a limit (#6403);
  compaction keeps speculative warm summaries (#5481); `html-render` moves out of the builtin tree (see section 5).
- **S18/S19 interactive and tui**: #4271 attach the interactive TUI to an RPC endpoint on every platform; #5695
  copy original Markdown from fullscreen selections; footer and retry-status cosmetics.
- **S20 cli**: #4771/#4864 private descriptors forwarded to one engine generation, Windows pipe pairs; #5045
  `show_html_page` attached through the host SDK; #5970 startup refuses or takes over a session another live
  process holds; #6914 an attaching start applies its command line; per-generation bundled bun shim and the
  engine trust catalog (omo#10762).
- **S21/S22**: client materializer kit, a Bun SQLite DurableStore backend, the new `realm-registry` package;
  pty pipe fallback settles on child exit (#6203).

### oh-my-openagent (`v5.1.52..v5.1.81`, bun-fmt commit excluded from line counts)

| Group | Commits | TS src lines (+/-) | Test lines | Docs/CI lines | mhc crate(s) | Dori |
|---|---:|---:|---:|---:|---|---|
| O01 thread component (thread_create/send/list, wake, host-ensure, cross-device) | 812 | +29181 / -10252 | 36095 | 432 | maho-omo-gateway (`src/thread`) | HIGH for local thread tools, wake, host-ensure; cross-device LOW |
| O02 remote/device-service (omo remote, relay local-api, SDK), new | 392 | +25958 / -5728 | 28401 | 625 | none | LOW (relay and device service) |
| O03 omo-native (omo CLI launcher: remote/sync/trust verbs) | 925 | +11062 / -4091 | 21070 | 134 | maho-cli / maho-omo launcher pieces | MED (launcher trust, update); remote/sync verbs LOW |
| O04 memory (memory-core + memory component) | 434 | +23631 / -6142 | 29081 | 29 | memory-core, maho-omo-memory, maho-memory | HIGH (memory sync parts LOW) |
| O05 task/subagents/categories/team | 256 | +6453 / -2282 | 11871 | 75 | senpi-task, maho-omo-task, team-core, maho-task | HIGH |
| O06 ultrawork / ulw-loop / start-work | 24 | +1096 / -988 | 503 | 0 | maho-ultrawork, maho-omo-ulw-loop, maho-omo-start-work-continuation | HIGH |
| O07 LSP | 6 | +21 / -4 | 104 | 0 | lsp-core, lsp-daemon, maho-omo-lsp, maho-lsp | HIGH, but practically unchanged |
| O08 MCP packages | 9 | +83 / -33 | 284 | 26 | mcp-stdio-core, git-bash-mcp, ast-grep-mcp, maho-omo-builtin-mcps (mcp-client-core: none) | HIGH |
| O09 config/model | 53 | +742 / -370 | 1936 | 2 | omo-config-core, model-core, maho-omo-config-* | HIGH |
| O10 skills and prompts | 113 | +2660 / -464 | 1928 | 23178 | maho-omo-bundled-skills, maho-skills, prompts-core | MED |
| O11 omo-senpi extension core, vendored bunshin-ts, plugin build | 1555 | +5582 / -215 | 4957 | 951 | maho-omo | MED (eval-cell fleet SDK is relay; LOW for Dori) |
| O12 desktop / computer-use (TS plus upstream Rust `crates/`) | 415 | +15474 / -2421 | 62268 | 831 | senpi-desktop-*, crates/desktop/*, maho-omo-computer-use, maho-browser | LOW |
| O13 other hosts (omo-opencode, omo-codex, openclaw) | 389 | +8991 / -2910 | 17326 | 128 | none (not ported) | LOW |
| O14 telemetry/feedback | 34 | +376 / -83 | 927 | 0 | telemetry-core, maho-omo-telemetry, maho-omo-feedback | LOW |
| O15 web (omo.dev site) | 14 | +2920 / -3826 | 952 | 290 | n/a | LOW |
| O16 other omo-senpi components (onboarding, claim-receipt, side-panel, call-budget, js-kernel-guard, ...) | 27 | +255 / -65 | 788 | 0 | maho-omo-<name> (call-budget, js-kernel-guard: none) | js-kernel-guard HIGH (eval); rest LOW |
| O17 small shared packages (utils, tmux-core, isolation-core, rules-engine, ...) | 17 | +74 / -30 | 130 | 3 | same-name crates under crates/omo | LOW |
| O99 repo infra, scripts, tests, docs, CI, changes/ | 1870 | +9915 / -926 | 20149 | 28271 | n/a | counted only |

Notable changes per group (`#` numbers are oh-my-openagent PRs):

- **O01 thread**: the biggest omo area, shipped through release trains (#13563, #14356, #14196, #14537,
  #14332). Features: #13599 cold-wake train (`wake` verb, a woken-session record, carrier and close; #13037);
  `host-ensure` in `thread/wake/`; #12890 one tool body per control tool over a `SessionControl` port;
  `thread_create` places a thread on any device, folder or new git worktree (`cb35acf79`); `thread_read wait_ms`
  (#13743, `504b1e20f`); #12335 thread tools reach sessions on the account's other devices; the address module
  API (#14305, also the new `thread-address-core` package). New component `call-budget` (omo#14130) bounds one
  thread tool call to 50 s so it settles inside the eval host-tool bridge's 60 s.
- **O02 remote/device-service** (new packages `device-service-core`, `thread-address-core`, new components
  `remote`, `device-service`, `omo-sdk`): #11857 device-service SDK and the `omo remote` command group; #13475
  vendored relay-daemon local-api client; #13753 attach with pending approvals, follow and wait; #13741 `/remote`
  in the TUI. This is omo's cloud and relay path.
- **O03 omo-native**: launcher trust stage with an exec bound (#11956, #10762); #13087 install and auto-update
  standalone binaries; #12627 `omo update` replaces the launcher and prunes old runtimes; `omo remote`
  open/send/interrupt/close verbs (#12709, #13746); `omo sync` status and recovery phrase (#14533); #12257 the
  device-service bootstrap runs at every interactive `omo` start.
- **O04 memory**: #12517 one stale reflection-lock reaper (session start, each turn, `omo doctor --fix`);
  #12532/#12538 per-identity runtime retention and size cap with oldest-first eviction; #11671 shutdown waits for
  in-flight cache reads; #12645 dream runs repoint links to removed notes; memory-sync (#12597, #12302 key
  rotation, #11774 genesis, recovery phrase, sync RPC coordinator).
- **O05 task**: #12010 a child runs only on its designated models (the parent-model fallback rung is removed);
  #12761 a child whose engine cannot hold its chain never starts; #12839 child dialogs on the parent's UI,
  children follow its approval mode; #14025 `omo.modelRouting.resolve` answered by the engine's routing; #12268
  Resume reaches children a Stop paused; #7662 stop replaying unchanged `task_output` snapshots; `team-core`
  lock owners fenced by worker lifetime.
- **O06 ultrawork**: #14389 forbidding advice no longer arms ultrawork and `keyword_detector` in `omo.jsonc`
  turns it off; #14460 default and GPT directives are two whole prompts; DeepSeek V4.1 Flash gets the GPT prompt.
- **O07 LSP**: after excluding the formatting commit, 21 added and 4 deleted TS lines (knip fragments, test
  budget helper, one description fix). Nothing to port.
- **O08 MCP**: `mcp-client-core` propagates the configured request timeout to skill MCP calls (#11869) and uses
  the legacy HTTP+SSE transport for `type: "sse"` (#11591).
- **O09 config/model**: Claude Fable 5.2 becomes the current Fable and the 5.5 launch gate is retired (#12953);
  GPT-6.2 routing (#14149, #14427); reasoning defaults raised and then reverted (#12094/#12272); model-profile
  Daily steps changed.
- **O10 skills**: programming scaffolders and `check-test-junk` moved to Bun/TS (#11970, #11972); new `omo-relay`
  native skill; skill-pointer fleet hint.
- **O11 omo-senpi core**: #12258 embeds the bunshin-ts fleet SDK for eval cells (vendored, 3896 lines in
  `3b264abcf`). The SDK is the relay path.
- **O16**: `js-kernel-guard` (#14023), an eval-only tool whose prelude keeps a JS cell's `process.exit` from
  killing the kernel.
- **O12/O13/O15**: computer-use v3 contract and live-tier CI, desktop SDK and QuickJS run cell (O12); omo-opencode
  per-model prompt parity and `service-auth` keychain login (O13); the omo.dev site renewal (O15).

## 4. Porting effort for the HIGH groups, and an order

The sizes are *estimates*: S = under a day, M = days, L = one to two weeks, XL = more than two weeks of focused
porting, ledger and golden work. Current Rust size of the target crate (`src/**/*.rs` lines, measured) is in
brackets for scale.

| Group | Size | Reason |
|---|---|---|
| O07 LSP | S | Effectively no upstream change; move the ledger pin only. |
| O08 MCP packages | S | Two `mcp-client-core` fixes; mhc has no `mcp-client-core` crate, so locate where skill-MCP timeouts live first. |
| O06 ultrawork | S | About 1 k lines; keyword-detector off switch and the directive split [maho-ultrawork 1.4 k]. |
| O16 js-kernel-guard | S | One small prelude component; needs a new component crate or a home in maho-omo. |
| O09 config/model | S | Model ids and routing tables (Fable 5.2, GPT-6.2) [model-core, omo-config-core]. |
| S12 terminal | M | Background-session reporting and auto-reap, interrupt stops background jobs, notifier retirement; cross-device monitor targets can wait [maho-ext-terminal 36 k]. |
| S13 mcp | M | OAuth continuity and registration rules, session-less MCP, scope routing [maho-ext-mcp]. |
| S14 webfetch/websearch | M | websearch may already be partly ported (section 5); the connected-browser webfetch path is the new work. |
| S10 codemode | M | Kernel permission checks, process isolation default and nested partial streaming; 22.7 k test lines to mirror [maho-codemode 25 k]. AGENTS.md describes maho-codemode as "bun/js only", so the Python-kernel half (#4392) has no target; a py kernel for Dori is a separate, larger item (L, *estimate*). |
| S11 extension host/api | M | Tool-hook deadline and MCP registry API are needed; the two-sided hand forwarding can be stubbed [maho-ext-host, maho-ext-api]. |
| S08 anthropic-subscription | M | The Claude CLI login stack, account slots and per-slot lanes; this is what Dori's Claude accounts run on [20 k]. |
| S07 ai | M | Delete the removed Anthropic OAuth paths, add SIWC hardening and per-account catalog [maho-ai 161 k]. |
| O05 task | M | Designated-chain-only children, child dialogs, routing resolve; 11.9 k test lines [senpi-task 164 k, maho-omo-task 21 k]. |
| S09 tools + machine-ops | L | The worker-safe core split touches bash/read/edit/write together, and machine-ops is a new package with no crate. |
| S05 + S16 kill-guard | L | New runtime (PATH shims, `BASH_ENV` wrapper, bash_input line modeling); safety-sensitive, needs its own goldens. |
| S20 cli | M | Session-opening takeover rule, private-fd tables; most of the rest is distribution plumbing. |
| O04 memory | L | 23.6 k TS lines; take retention, reaper and lock fixes; leave memory-sync out [memory-core 99 k, maho-omo-memory 100 k]. |
| S03 core session | L | Durable SessionManager binding, JSONL import, move rules, lifecycle event order; touches `agent-session` everywhere [maho-core 170 k]. |
| S06 agent package | XL | New durable store, execution ledger and wake inbox/outbox contracts with conformance suites; prerequisite for S03's durable store [maho-agent 74 k]. |
| O01 thread (local + wake) | XL | 29 k TS lines on top of a partial port [maho-omo-gateway 17 k]; the cross-device half can be skipped, which roughly halves it (*estimate*). |
| S01 + S02 rpc host daemon | XL | 49 k TS lines with new subsystems (compact recovery, loopless runtime, claim_session, bounded startup) [maho-rpc 104 k]; Dori's host verbs depend on it. |

Ordered plan for the HIGH groups (each step ends green on the parity audit and goldens before the next):

1. **Re-pin hygiene** (no behavior): move PINS.md, `tools/golden/pin.mjs`, `ci.yml` and `mhc-binary.yml`
   `SENPI_PIN` together; add an omo pin check (see section 6). Port O07 (ledger only), O09, O06, O08, O16.
   These are small, independent and Dori-visible (model routing, ultrawork arming).
2. **Provider and accounts**: S07, then S08. Dori's Claude CLI accounts and ChatGPT subscription live here, and
   S07 deletes code that S08 no longer calls.
3. **Tools and bash safety**: S09 core split, then S05+S16 kill-guard, then S12 terminal. Monitors and
   background sessions depend on the terminal changes; kill-guard follows the standing "do not kill processes
   you did not start" agreement.
4. **Eval and extensions**: S10 codemode, S11 extension host/api, S13 mcp, S14 webfetch/websearch.
5. **Agent core**: S06 durable store and execution contracts, then S03 session (durable SessionManager, move,
   lifecycle event order), then S20 cli session opening.
6. **Host daemon**: S01, then S02. This is last because it builds on S03/S06, and it is the riskiest for
   `omo host session open/read/prompt/steer/abort/state/unload/close/list/wait` and `ensure/status/stop`.
   Freeze golden traces of those verbs from the current mhc first, so any behavior change is deliberate.
7. **omo side, after senpi step 5**: O05 task (designated chains, dialogs), O04 memory (without sync), then
   O01 thread local tools, wake and host-ensure.

MED groups (S04 pool/accounts part, S15, S17 compaction/goal/prompt-preset, O03 launcher, O10, O11) can ride
along with the step that touches the same crate.

## 5. Obsolete areas and ports already past the pin

Upstream changes that retire mhc code:

- **Anthropic OAuth in maho-ai.** senpi deleted `ai/src/auth/oauth/anthropic.ts`,
  `anthropic-callback-listener.ts`, `authorization-input.ts`, `error-details.ts`, `auth/pool/classify.ts`,
  `utils/claude-code-version.ts`, `claude-code-version-cache.ts` and `fable55-launch.ts` (#5136, #4436). It now
  refuses `sk-ant-oat` tokens on `anthropic`. The Rust ports of those files become dead code, and Claude access
  goes only through `anthropic-subscription` (the Claude CLI). Check that Dori does not depend on the OAuth path.
- **`maho-ext-html-render`.** The `builtin/html-render` directory is gone at the tag (bootstrap, index, tool and
  images files deleted). `show_html_page` now attaches through the host SDK in `coding-agent/src/html-render`
  (#5045). The crate maps to a deleted root and has to be re-rooted under maho-cli.
- **`mcp/expose/native-search.ts` deleted** (#4782 removed the MCP-era copy of native tool-search). Its port in
  maho-ext-mcp is dead.
- **Legacy tool scheduler.** #5738 lands S36 and removes the legacy scheduler in favor of the prepared pipeline,
  so the scheduler path in maho-agent/maho-core is superseded and not just extended.
- **omo `lsp-core`, `tmux-core`, `ast-grep-mcp`, `rules-engine`**: the large line counts in these packages are
  the bun-fmt reformat. They are not obsolete, but they need no port.
- No mhc crate maps to a deleted omo package; omo removed no packages in the range, it only added 11
  (`device-service-core`, `thread-address-core`, `senpi-desktop-sdk`, `gnome-window-rects-extension` and seven
  `omo-computer-*`). omo also added Rust crates of its own under `crates/` (`omo-computer-*`,
  `omo-git-job-helper`, `senpi-desktop-xpc` existed already). Those are desktop/computer-use and LOW for Dori.

Ports already past the pin (checked in `git log` of mhc `main` at `b6414898f`):

- No mhc commit names an upstream change past the pins. Every `senpi#NNNN` reference in mhc commit messages (24
  distinct numbers, highest #4194) resolves to a PR merged at or before `59d58cee5`; no commit names an omo
  version after 5.1.52. `4a9260245 feat(omo): port the task host-plumbing primitives from omo v5.1.52` and
  `7b3ee0c01 feat(handoff): ... align surfaces with upstream` (senpi#3323) fill gaps *at* the pins, not beyond.
- The last pin move is `0181bea52 chore(sync): pin senpi v2026.10.27-2 and omo v5.1.52` (2026-11-01), merged
  as `53dfcebac` (2026-11-03). Since then mhc has 469 commits (418 non-merge). By scope they are mostly
  `memory` (141), `cli` (20), `tools` (16), `ultrawork` (12) and `task` (12): fork-local fixes, kibitzer/memory
  work and gap fills.
- **Likely functional overlap, to verify before porting S14:** `91b72d7bb fix(websearch): fan out public
  searches and honor result constraints` (2026-11-19) adds a public fan-out and paged search to
  maho-ext-websearch, and the crate already has a `tinyfish.rs` provider. Upstream added the public fan-out
  provider (`0c3180bf7`, 2026-11-02) and TinyFish (`ec9b1d9f0`) inside the range. The ledger does not record
  this, so diff the behavior before porting.
- `3e0e3f64a fix(compaction): never cut on an entry without context messages` and `61bbeacc5` (#18 cache and
  request timing notices) are fork-local fixes. Upstream compaction changes in S17 (#5481) need a merge, not a
  blind port, around them.
- Stale references to note: `tools/PARITY-FORMAT.md` still says the audit needs senpi at `fe8c564b` and the faux
  tools need omo at `77f3067f`, and `tools/hg-v-ulw.mjs` records `upstreamPin: "77f3067f..."`. `77f3067f` does
  not exist in the oh-my-openagent clone. `tools/check-pins.mjs` checks only `SENPI_PIN`; nothing checks the omo
  pin.
- The Dori launcher (`scripts/dori/mhc-dori.sh`) isolates mhc from OmO with a shadow HOME, because mhc builds
  many `$HOME/.omo` paths directly (maho-config, maho-agent-home, team-core registry, context advisor, host
  probe). Any upstream change to those paths (`tools/omo-sites.json`, `check-omo-sites.mjs`) has to keep the
  shadow-HOME assumption valid. This was noted from the request; the script was not read or changed.

## 6. Repeatable sync procedure for each new omo release

1. **Detect the range.**
   - `git -C upstream/oh-my-openagent fetch --tags origin`, then take the newest `v5.1.N` tag.
   - Read the senpi version it ships:
     `git show vX:packages/omo-senpi/package.json | grep '"@code-yeongyu/senpi"'` (also the root
     `package.json`). Map that version to the senpi tag `v<version>` and check it with
     `git merge-base --is-ancestor <current pin> v<version>`.
   - The ranges are `<PINS senpi>..v<senpi version>` and `<PINS omo>..vX`. Record commit, merge and non-merge
     counts the way section 1 does, and how far `origin/main` and `origin/dev` are past the tags.
   - Keep both clones full (no `promisor`/`partialclonefilter`). Partial clones make every numstat lazily fetch
     blobs.
2. **Slice it.**
   - Run the area classifier that produced sections 2 and 3: `SOURCE_ROOTS` and the builtin/omo component lists
     from `tools/parity-audit.mjs` for crate mapping, and `parity.d` TS paths for omo packages.
   - Drop pure formatting commits by subject (`chore(fmt)`, `style:`) and report them separately.
   - Flag new top-level packages, new builtins and new omo components (`git ls-tree` diff of
     `packages/`, `builtin/`, `components/` between the two refs) as "no crate yet".
   - Flag deleted non-test TS files; each one is a candidate obsolete Rust module.
   - One slice = one group, ported on its own branch `sync/<omo-version>/<group>`, in the order of section 4.
3. **Port a slice.**
   - Check `SENPI_SRC` out at the new senpi tag and `OMO_SRC` at the new omo tag (worktrees, not the main
     clones).
   - Diff the group's TS paths between the two refs, port file for file (CONTRIBUTING.md: one TS file, one Rust
     module, same names and order).
   - Write the crate's `parity.d/<todo>.md` fragment with the new TS test case counts. A higher todo number
     closes earlier rows.
4. **Verify.**
   - Run `bun tools/golden/run.mjs --case <affected cases>` with the new `SENPI_SRC` to regenerate goldens (never
     by hand), then the Rust tests.
   - Run `SENPI_SRC=... bun tools/parity-audit.mjs --crate <crate>` for every touched crate. Exit 3 means
     `SENPI_SRC` is not at the pinned commit, so the audit runs only after the pin bump in step 5, or with
     the bump staged on the sync branch.
   - For omo groups, run `bun tools/golden/faux-harness.mjs --omo` and `faux-tui.mjs --omo` scenarios with
     `OMO_SRC` at the new tag.
   - Run `scripts/verify-local.sh` (fmt, clippy, nextest). GitHub Actions is disabled for this repository, so
     local runs are the only verification. Tests that compare against upstream read `~/code/senpi` at
     `SENPI_PIN`, so move that checkout too.
   - Run the Dori smoke: `omo host ensure/status`, `host session open/prompt/read/wait/state/close`,
     `omo -p`, one task child, one eval cell in each kernel, one monitor, one memory write. Use the mhc binary
     under the shadow HOME of `scripts/dori/mhc-dori.sh`.
5. **Move the pins in one change.** Update PINS.md (both rows), `tools/golden/pin.mjs`, `.github/workflows/ci.yml`
   and `mhc-binary.yml` `SENPI_PIN` (the workflows are manual-only, but `check-pins.mjs` still compares them), then run `bun tools/check-pins.mjs`. Add an omo pin check next to it, and fix
   the stale `fe8c564b`/`77f3067f` references in `tools/PARITY-FORMAT.md` and `tools/hg-v-ulw.mjs` at the same
   time.
6. **Record.** Append a dated entry to this directory (`docs/dori-sync/<omo-version>.md`) with the measured
   ranges, what was ported, what was deferred (LOW groups, relay/device-service, desktop) and why.
