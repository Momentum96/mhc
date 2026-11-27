# `assets/omo-latest` — repository-owned latest skill assets

This tree is the **explicit source of the latest-mode skill assets**. It exists so
`tools/package-native.mjs --skill-source latest` can stage the latest skill set from the repository
itself, instead of inferring "latest" from a host checkout whose `PINS.md` commit is the *pinned*
revision. Nothing here is generated at build time and nothing here is written by the packaging
script.

## Why an overlay instead of the pinned checkout

`--omo-root` points at the **pinned** `oh-my-openagent` checkout (`PINS.md` → v5.1.81,
`5835a98e…`). The overlay is synced to that same commit, but latest mode reads this
self-describing tree rather than a host checkout. This makes the shipped roster, local MIT
exception, and browser runtime identity explicit.

## Modes

| Mode | Flag | Skill source | git-bash sibling | Notes |
| --- | --- | --- | --- | --- |
| pinned (default, baseline) | *(none)* / `--skill-source pinned` | `--omo-root` pinned checkout, `packages/omo-senpi/skills` over `packages/shared-skills/skills` | optional (`--git-bash-binary` may be absent) | uses the packager's pinned roster |
| latest (selected product) | `--skill-source latest` | this overlay only | **required** (`--git-bash-binary <built omo-git-bash>`) | stages `selection`, retires `retired`, never reads `--omo-root`; also **requires** `--omowright-runtime <dir>` for the `browser` skill's engine runtime |

`PINS.md` is never modified by either mode: `provenanceErrors()` requires both pins to be full
commits in every mode, and latest mode additionally requires the overlay's own `commit` to be a full
commit that is **not** the pinned omo commit, unless the overlay declares `pinCommit` equal to it
(an overlay synced exactly at the pin, as for omo v5.1.81).

## The selected product staging

The selected product staging is `--skill-source latest` **with** `--git-bash-binary` naming the
actual built `omo-git-bash` sibling. It is the only staging that delivers the git-bash MCP server:

```bash
node assets/omo-latest/stage-omowright-runtime.mjs \
  --source <omowright checkout @ bd3843cc9625d87cafa00c08cfc9f0c995d1aab6> \
  --target assets/omo-latest/runtime/browser/omowright
bun tools/package-native.mjs --binary "$BIN" --output "$E/install" \
  --skill-source latest --git-bash-binary "$BIN_DIR/omo-git-bash" \
  --omowright-runtime assets/omo-latest/runtime/browser/omowright
bun tools/package-native.mjs --verify-only "$E/install"
```

The absent-sibling case is the pinned **baseline**, not the git-bash delivery: the baseline stages
the sibling optionally, and `latest` refuses to run without it. `--git-bash-binary` is the single
git-bash flag — there are no aliases. The staged manifest declares the sibling in
`layout.gitBashMcp` and as a hashed `executables` entry, and `verify()` enforces both.

## Layout

| Path | Meaning |
| --- | --- |
| `manifest.json` | the machine-consumed overlay contract (identity, selection, retired, per-skill file lists, historical H1–H13 inventory, loader + content binding) |
| `skills/<name>/**` | the exact latest bytes of every selected skill, precedence already resolved |
| `skills/AGENTS.md` | the `packages/omo-senpi/skills/AGENTS.md` pool-root developer doc (carried verbatim, not a staged skill) |
| `stage-omowright-runtime.mjs` | the repository-owned materializer for the `browser` skill's engine runtime (build-time only; the repository holds no committed runtime) |
| `runtime/.gitignore` | ignores the materialized `runtime/browser/omowright/` directory, exactly as upstream gitignores the runtime |
| `PROVENANCE.md` | overlay-side provenance and license note |

## `manifest.json` (machine-consumed)

| Field | Meaning |
| --- | --- |
| `commit` / `sourceRepo` | the immutable source of the upstream bytes (v5.1.81, `5835a98e…`) |
| `pinCommit` / `pinSenpi` | the pins that stay unchanged |
| `versions` | `omoSenpi` 5.1.81 / `sharedSkills` 0.1.0, recorded into the staged manifest in latest mode |
| `precedence` | `packages/omo-senpi/skills` over `packages/shared-skills/skills` |
| `selection` | the 28 v5.1.81 skill names staged in latest mode (= `targetNames`) |
| `retired` | `adversarial-plan`, `start-work` |
| `poolDocs` | pool-root docs carried but not staged as skills |
| `skills` | per skill, the exact file list with `path`/`bytes`/`sha256`/upstream `source` |
| `targets` | the historical H1–H13 inventory at `targetInventoryProvenance.commit`; not the current file list |
| `loader` | the installed-loader binding: `OMO_SENPI_SKILLS_ROOT` → `<install>/skills`, consumed by `omo_mount.rs` |
| `contentBinding` | complete-content binding: see `contentBinding` counts, the rest bound at the gate |
| `browserRuntime` | the `browser` skill's build-materialized engine runtime: skill, target dir, materializer, immutable `omowright` identity, version and the exact runtime files |

## Browser engine runtime (build-materialized, never committed)

Upstream ships `skills/browser/runtime/omowright/{index.js,page-bundle.js,LICENSE,manifest.json}` — the
`browser` skill's engine, bundled from the root `omowright` devDependency
(`^0.3.1`, locked in `bun.lock` to 0.3.1, whose npm `gitHead` matches tag `v0.3.1` at
`bd3843cc9625d87cafa00c08cfc9f0c995d1aab6`) at build/prepack time by
`packages/shared-skills/stage-omowright-runtime.mjs`. The directory is gitignored and shipped through
the skill's `.npmignore`; without it, `skills/browser/scripts/omowright.mjs` throws *"omowright is
not staged in this skill"*.

This overlay carries the same content, materialized by its own repository-owned source:
`assets/omo-latest/stage-omowright-runtime.mjs` (a port of the upstream materializer: same source
digest entries, same `bun build` invocation, plus the immutable commit in the runtime manifest). The
materialized directory is **not** committed (`runtime/.gitignore`); it is produced at the final gate
and staged explicitly:

```bash
node assets/omo-latest/stage-omowright-runtime.mjs --source <omowright checkout> --target <dir>
bun tools/package-native.mjs --binary "$BIN" --output "$E/install" --skill-source latest \
  --git-bash-binary "$BIN_DIR/omo-git-bash" --omowright-runtime <dir>
```

`manifest.json.browserRuntime` declares the runtime's identity (package, dependency spec, immutable commit,
version) and the exact runtime files. Staging requires an explicit `--omowright-runtime <dir>` whose
own `manifest.json` records that same commit, records the runtime version and source digest plus
each runtime file's sha256 in the staged package manifest, and **refuses** a content-only `browser`
tree. `verify()` requires every declared runtime file to be present and hashed.

This is packaging of upstream skill content only: it adds no desktop/browser implementation, no
engine or permission-protocol authorization (`dec-a-02` stays an owner choice) and no extension
bridge.

## Integrity rules the packaging enforces

1. **Explicit entries only.** Staging copies exactly `selection`; nothing is published by directory
   exclusion and nothing outside the selection is staged.
2. **No silent short ship.** Each selected skill's on-disk file set must equal its `skills` entry
   exactly. A missing or extra file refuses the run (`latest overlay is incomplete`).
3. **Declared hashes bind the bytes.** A non-null `sha256` is checked against the file before it is
   staged; a mismatch refuses the run. Every staged file is hashed into the package manifest.
4. **Retirement is declared, not implied.** `retired` names are refused if they also appear in
   `selection`.
5. **The selected delivery includes the sibling.** `latest` refuses to stage without
   `--git-bash-binary`, and `verify()` enforces the declared `omo-git-bash` sibling.
6. **The browser engine runtime ships with its identity.** `latest` refuses to stage without
   `--omowright-runtime <dir>`, refuses a runtime whose recorded commit is not the
   `browserRuntime.dependency.commit` the overlay declares, and `verify()` requires every declared
   runtime file to be present and hashed.

## Precedence and the copy filter

The overlay is the **already-merged** latest skill set. Where a name exists in both upstream pools
(`init-deep`, `ulw-plan`, `ulw-research`), the `packages/omo-senpi/skills` copy wins, exactly as
`sync-skills.mjs` does. `ulw-research` additionally carries the shared-pool assets that
`native-skill-sources.mjs` overlays (`scripts/**` and the seven declared shared references);
the shadowed shared `SKILL.md` is not shipped. Unlike `sync-skills.mjs`, the overlay does not
rewrite the copied text.

The upstream v5.1.81 `skill-source-filter.mjs` excludes `.gitignore`, `.npmignore`,
`pyrightconfig.json`, `knip.workspace.jsonc`, `openai.yaml` (Senpi option), `*.test.ts`,
`*.pyc`, nested test directories, and caches. A skill's own top-level `test/` and `tests/`
directories may ship.

## Provenance and license

With the two local MIT ultrawork files listed in `PROVENANCE.md` excepted, skill files here
are byte copies of the immutable checkout at `commit`, under the same terms as the
rest of the skill set: omo-derived assets ship under `LicenseRef-SUL-1.0`
(`LICENSES/SUL-1.0.md`). Third-party notices in the source trees are carried unchanged.
`PROVENANCE.md` records the provenance.
