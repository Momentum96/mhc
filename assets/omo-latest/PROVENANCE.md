# `assets/omo-latest` — provenance note

The original L-IF-2 authoring receipt (`authoring/assets.md`) is not present in this
checkout. The `mc:H` rows in the manifest retain that historical migration inventory.

| Item | Value |
| --- | --- |
| Overlay | `assets/omo-latest` (repository-owned latest skill assets) |
| Bytes taken from | `code-yeongyu/oh-my-openagent` v5.1.81 @ `5835a98e88438f28a8ae0ee18039fad4d5fd415b` |
| Pinned OMO (unchanged) | `5835a98e88438f28a8ae0ee18039fad4d5fd415b` |
| Pinned senpi (unchanged) | `99a26322f4a81d5ed95f0fd9f292938a6afbd8ff` |
| Selection | 28 skills (`manifest.json.selection` = `targetNames`); adds `eval-integrity`, `hyperplan`, `mass-ulw`, and `omo-relay` relative to the previous overlay |
| Retired | `adversarial-plan`, `start-work` |
| Precedence | `packages/omo-senpi/skills` over `packages/shared-skills/skills` |
| Copy mechanism | byte-exact file copy (mode preserved), filtered by `skill-source-filter.mjs` semantics |
| Installed loader | `OMO_SENPI_SKILLS_ROOT` → `<install>/skills` (`manifest.json.loader`) |
| git-bash sibling | native binary name `omo-git-bash`; staged from the single `--git-bash-binary` flag (required by the selected `--skill-source latest` staging) |
| Browser engine runtime | `browser` ships a build-materialized engine runtime (`runtime/browser/omowright/{index.js,page-bundle.js,LICENSE,manifest.json}`) produced by `assets/omo-latest/stage-omowright-runtime.mjs` from `omowright` 0.3.1 @ `bd3843cc9625d87cafa00c08cfc9f0c995d1aab6`; staged from the single `--omowright-runtime` flag (required by the selected `--skill-source latest` staging). Packaging of upstream skill content only — no desktop/browser implementation, engine authorization or extension bridge. |

`PINS.md` is not modified by this overlay or by either staging mode.

O06 MIT exception: `skills/ultrawork/SKILL.md` ships the existing
local MIT default contract with updated metadata. `skills/ultrawork/references/gpt.md`
is a separately authored MIT directive for GPT and DeepSeek V4.1 Flash, not
bytes copied from oh-my-openagent. `manifest.json.skills.ultrawork` binds both
files unchanged from the previous overlay. Other ultrawork files follow the upstream copy rules.

## License

omo-derived assets ship under `LicenseRef-SUL-1.0` (`LICENSES/SUL-1.0.md`).
`skills/browser/ATTRIBUTION.md` and the other per-skill `ATTRIBUTION.md` files carry their own
third-party notices; no third-party license text was stripped or rewritten.

The browser engine runtime is materialized from the Git commit of the locked npm release and carries
its own `manifest.json` (version, source digest, per-file sha256, immutable commit). It is not
committed (`runtime/.gitignore`), and its third-party terms are the upstream package's; nothing in
this overlay relicenses it.
