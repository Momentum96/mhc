# Dori sync roadmap: how to resume the large ports

This turns `backlog.md` (measured upstream changes) into work packages that a later session can pick up one at a
time. `status-2026-11-22.md` and `status-2026-11-23.md` (P1, P2, open items, omarchy setup) record what already
landed. Read them first; this file only adds order,
boundaries, gates and the brief to hand to a worker.

Decision on record (2026-11-22): the owner chose stabilization first. The large ports below are parked, not
dropped. Restart them only on an explicit request, one package at a time, and report each package's measured
cost before starting the next.

## Where things are

| Item | Location |
|---|---|
| Build host | omarchy (Linux x86_64). Since 2026-11-23 every mhc build, test and smoke runs there under `~/work`, not on macmini |
| Fork, branch with Dori fixes | `Momentum96/mhc`, `dori/compat-20261122` (checkout `~/work/mhc` on omarchy) |
| Upstream sources (full clones) | `~/work/upstream/{senpi,oh-my-openagent}` on omarchy |
| Pinned senpi checkout | `~/work/upstream/senpi-pin` (worktree of `~/work/upstream/senpi` at the PINS.md pin, with `bun install --frozen-lockfile`); export `SENPI_SRC=$HOME/work/upstream/senpi-pin` for every test run |
| Target versions | omo `v5.1.81` with senpi `v2026.11.19-5`; `scripts/dori/sync-detect.sh` prints the current range |
| Isolated runs | `scripts/dori/mhc-dori.sh` (shadow HOME `~/.maho/home`, `MHC_BIN` selects the binary) |
| Packaged build | `dist/mhc` (local only, excluded via `.git/info/exclude`) |
| Build output | `/mnt/nvme-work/targets/<checkout or worktree name>` on the NVMe; each `target` is a symlink to it (`~/work` itself is on a slow HDD) |
| Latest status and open items | `docs/dori-sync/status-2026-11-23.md` |

## Rules that every package follows

1. One package = one branch `sync/<package-id>` cut from the current Dori branch, one worktree
   (`git worktree add ../mhc-wt/<package-id> -b sync/<package-id>`), its own `target/`. A worktree with a
   release build takes about 60 GB; remove it after merging.
2. Port file for file from the target tags (CONTRIBUTING.md). Write the crate's `parity.d/<todo>.md` fragment.
   Do not move the PINS.md pins inside a package: the pin moves once, after every HIGH package of a release is
   in (`backlog.md` section 6, step 5). Until then, any data generated from a newer upstream is recorded as a
   divergence row in `docs/divergences.md`.
3. Gates per package, all on omarchy (GitHub Actions is disabled), one cargo invocation at a time:
   `cargo fmt --all --check`, `cargo clippy -p <crates> --all-targets -- -D warnings`,
   `SENPI_SRC=$HOME/work/upstream/senpi-pin cargo nextest run -p <crates> --no-fail-fast`,
   `cargo build --release --workspace --bins`, repackage, then the Dori smoke (below). Known failure on the base
   branch: `maho-cli` `multi_session_entry::a_cli_built_host_answers_the_protocol_and_exits_on_its_empty_window`
   (probe times out after 15 s); it is not a package failure. Keep `CARGO_TARGET_DIR` off `/tmp` (a quota-limited
   tmpfs on omarchy).
4. Tests and golden/parity checks that compare against upstream read the pinned senpi checkout through
   `SENPI_SRC=$HOME/work/upstream/senpi-pin` (with `tools/golden/run.mjs` for goldens, P6 onward). Keep that
   checkout at the PINS.md pin; move it only with the pin (P11).
5. The real `~/.omo` is never written. Never run `claude login/logout`, never print credentials, never kill
   processes the worker did not start.
6. Build output lives on the NVMe. Before the first build in a checkout or worktree, make its `target` a symlink:
   `mkdir -p /mnt/nvme-work/targets/<name> && ln -s /mnt/nvme-work/targets/<name> target` (`<name>` is the
   checkout or worktree folder name). Do not move `~/work` itself. Several agents build on omarchy at once, so
   every cargo command runs as `flock $HOME/work/.mhc-cargo.lock cargo ...`.

Repackage on omarchy (Linux needs the sibling binaries, the staged omowright runtime from
`scripts/install-dev.sh`, and the host glibc as the dev-build ceiling):

```bash
MHC_GLIBC_CEILING=2.44 bun tools/package-native.mjs --binary target/release/mhc \
  --launcher-binary target/release/mhc-launcher --output dist/mhc --skill-source latest \
  --git-bash-binary target/release/omo-git-bash --desktop-engine-binary target/release/senpi-desktop-engine \
  --omowright-runtime <staged omowright runtime>
MHC_GLIBC_CEILING=2.44 bun tools/package-native.mjs --verify-only dist/mhc
```

Dori smoke (run through the wrapper with the packaged binary; omarchy's default model is not Claude, so name it):

```bash
W=scripts/dori/mhc-dori.sh
$W --no-session --model anthropic-subscription/claude-opus-5-5 -p 'Reply with exactly: OK'   # Claude subscription
$W --no-session --model chatgpt-subscription/gpt-6.1-sol -p 'Reply with exactly: OK'
$W doctor                                                         # categories usable
$W host ensure && $W host session open --cwd /tmp/x --name s --prompt 'Reply with exactly: PONG'
$W host session wait <id> --until idle --timeout 180000 && $W host session read <id> --tail 4 --messages
$W host session close <id> && $W host stop
# DAG: 3 parallel category-quick children + join writing a file, 5 runs in a row, each writes its file
```

## Work packages

Sizes are estimates from `backlog.md` section 4 (S < 1 day, M = days, L = 1-2 weeks, XL > 2 weeks of focused
porting). Cost note from 2026-11-22: each of the four compatibility fixes took one worker 0.5-2 h and roughly
4-30 M tokens. Expect an M package to cost several times that and an XL package tens of times.

| ID | Groups | Size | Depends on | Status | Done when |
|---|---|---|---|---|---|
| P0 | Stabilization: O09 catalog (claude-fable-5-2), intermittent child failures | S | - | done 2026-11-22 | doctor 10/10, DAG 5/5 |
| P1 | O07 LSP ledger, O06 ultrawork, O08 MCP timeouts/SSE, O16 js-kernel-guard, rest of O09 | S each | P0 | done 2026-11-23 (merged into `dori/compat-20261122`) | each group's TS diff ported, gates green |
| P2 | S07 ai (remove Anthropic OAuth paths, SIWC hardening, per-account catalog), rest of S08 (CLI `/login`, logout, account management, `recheckedAccounts`, `persistBlocks`) | M + M | P0 | done 2026-11-23 (merged into `dori/compat-20261122`) | Claude and ChatGPT accounts can be added, removed and fail over inside mhc |
| P3 | S09 tools core split + machine-ops crate | L | P2 | done 2026-11-24 (merged into `dori/compat-20261122`) | bash/read/edit/write on the `*-core` split, machine-ops crate exists |
| P4 | S05 + S16 kill-guard | L | P3 | done 2026-11-24 (merged into `dori/compat-20261122`) | `kill`/`pkill`/`killall` against a protected process is refused, own processes still killable |
| P5 | S12 terminal (background reporting, auto-reap, interrupt stops jobs) | M | P3 | done 2026-11-24 (merged into `dori/compat-20261122`) | monitor and bash session behavior matches omo 5.1.81 |
| P6 | S10 codemode, S11 extension host/api (rest), S13 mcp, S14 webfetch/websearch | M each | P2 | done 2026-11-24 (merged into `dori/compat-20261122`) | eval kernels with permission checks, MCP OAuth continuity |
| P7 | S06 agent durable store + execution ledger | XL | P3 | done 2026-11-24 (merged into `dori/compat-20261122`) | DurableStore contract and conformance suites pass |
| P8 | S03 core session (store-first SessionManager, move, import, lifecycle order), S20 cli session opening | L + M | P7 | done 2026-11-24 (merged into `dori/compat-20261122`) | sessions survive a restart the way omo's do |
| P9 | S01 + S02 rpc host daemon and compact recovery | XL | P8 | two host fixes landed (disconnect keeps the turn, `MAHO_` env) | host verbs Dori uses behave like omo 5.1.81; freeze current traces first |
| P10 | O05 task (designated chains, child dialogs, routing resolve), O04 memory (no sync), O01 thread (local tools, wake, host-ensure) | M + L + XL | P8 (O05), P9 (O01) | O04 memory done 2026-11-24; O05 task, O01 thread remaining | DAG, memory tool and thread tools match omo |
| P11 | Pin move: PINS.md, `tools/golden/pin.mjs`, workflow `SENPI_PIN`, omo pin check | S | P1-P10 | not started | `bun tools/check-pins.mjs` passes at the new pins |

LOW groups (O02 remote/device-service, O12 desktop, O13 other hosts, O15 web, S18/S19/S21 cosmetics, S22) stay
out unless Dori starts to need them.

## Brief template for a package worker

```text
Port package <ID> (<groups>) into our mhc fork on omarchy. Work only in the worktree ~/work/mhc-wt/<ID>
(branch sync/<ID>, cut from dori/compat-20261122 or its successor; its target is a symlink to
/mnt/nvme-work/targets/<ID>). Read AGENTS.md, CONTRIBUTING.md, docs/dori-sync/{backlog,roadmap}.md,
docs/dori-sync/status-2026-11-23.md (latest state and open items) and the AGENTS.md of each crate you touch. Upstream: senpi at tag
<senpi tag>, oh-my-openagent at tag <omo tag> under ~/work/upstream; tests use
SENPI_SRC=$HOME/work/upstream/senpi-pin; every cargo command runs under flock $HOME/work/.mhc-cargo.lock. Scope: the TS files of
<groups> listed in backlog.md; out of scope: <LOW parts>. Rules: roadmap.md "Rules that every package follows".
Wait for every build, test and run in the foreground inside one tool call with a large timeout; never end your
turn while anything you started is still running. Do not commit or push. Done when: <package "Done when">, plus
the gates and the Dori smoke. Report: files changed per TS source file, tests added, gate results, smoke output,
and the tokens and wall time the package took.
```

## Starting a package with agy

agy (the Antigravity CLI) has none of the earlier conversations, so everything it needs is in this folder:

1. Create the worktree: `git -C ~/work/mhc fetch origin && git -C ~/work/mhc worktree add ~/work/mhc-wt/<ID> -b sync/<ID> origin/dori/compat-20261122`.
2. Give it an NVMe target: `cd ~/work/mhc-wt/<ID> && mkdir -p /mnt/nvme-work/targets/<ID> && ln -s /mnt/nvme-work/targets/<ID> target`.
3. Open agy in `~/work/mhc-wt/<ID>` and tell it: "Run package <ID> following docs/dori-sync/roadmap.md."
4. When it reports back, check the gates it ran and the status line in the work-package table.

## Lessons from 2026-11-22

- Workers that start a long build in the background and end their turn are not woken again. Tell them to wait in
  the foreground (the brief above does), and check `target/release/mhc` mtimes before believing a "still
  building" status.
- A raw `target/release/mhc` has no bundled skills; anything beyond `-p` smoke uses the packaged `dist/mhc`.
  Never repackage `dist/` while a benchmark or smoke run uses it.
- The shadow HOME must not mirror `.pi`/`.senpi`: mhc's startup migrations import from them.
- Memory comparisons must split Claude CLI processes from the engine; the CLI is the same binary on both sides.
- On omarchy, prepend `~/.local/share/mise/installs/claude/latest` first on `PATH` for every smoke run so `claude` resolves directly to the ELF binary instead of `~/.local/bin/claude` (`mise use -g --quiet claude`, which can block under `host`).
- Hold `.mhc-smoke.lock` on FD 9 (`exec 9>$HOME/work/.mhc-smoke.lock && flock 9`) and append `9>&-` to every `mhc-dori.sh` call (with an `EXIT` trap running `host stop`) so daemonized `host ensure` supervisors never inherit the smoke-lock descriptor.
