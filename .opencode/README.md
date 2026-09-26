# opencode project integration

The `.claude` tree holds the canonical repository instructions, skills, and agent prompts. This directory exposes them to opencode through its documented project configuration. Treat `.claude` as the source and `.opencode` as a generated mirror.

| Claude source | opencode integration |
|---|---|
| CLAUDE.md files | `opencode.json` at the repository root registers the root `CLAUDE.md` through the `instructions` key. opencode also reads `CLAUDE.md` as a native fallback for `AGENTS.md`. The repository has no root `AGENTS.md`; see `.codex/README.md` for the reason. |
| .claude/agents | `agents/` holds one generated Markdown agent per canonical prompt. The generator rewrites only the front matter. The prompt body is identical byte-for-byte. |
| .claude/skills | `skills` is a symbolic link to `.claude/skills`. opencode also discovers `.claude/skills` and `.agents/skills` natively, so it logs one "duplicate skill name" warning per skill at startup. Set `OPENCODE_DISABLE_CLAUDE_CODE_SKILLS=1` to remove it. |
| .claude/settings.json | The `Read` deny rule for `target/` became a `permission.read` rule in `opencode.json`. Hooks are Claude configuration and are not ported. |

## Agent front matter

| Claude field | opencode field |
|---|---|
| File name | The agent identifier (`automated-orchestrator`, `duet-engineer`, ...). |
| `name`, `emoji`, `vibe` | Removed. opencode passes an unknown field to the model provider as a model option. |
| `model: opus` | `model: openrouter/deepseek/deepseek-v4.1-flash` (the `OPENCODE_MODEL` constant in `tools/bc_repo_guard/xtask/lang_rust/src/sync_agents.rs`). |
| `color: <name>` | The CSS hex value of that name. |
| No Claude field | `mode: all` for the five root actors (`automated-orchestrator`, `hypervisor`, `hypervisor-turn`, `orchestrator`, `orchestrator-turn`). `mode: subagent` for every other agent. |
| `description` | The same text, quoted as JSON so an unquoted colon parses. |

Agent prompts name other agents by their Claude display name, for example "Engineering Critic". opencode lists the same agent as `engineering-critic` in the `task` tool. The prompts are not modified.

## Configuration

`opencode.json` sets `model` and `small_model`, `subagent_depth: 2`, `instructions: ["CLAUDE.md"]`, and the `target/` read deny rule. OpenRouter credentials come from `opencode providers login` or `OPENROUTER_API_KEY`.

## Regenerate the agents

    cargo xtask sync-agents          # write
    cargo xtask sync-agents --check  # verify (the commit gate runs this)

The generator proves that each emitted file carries the canonical prompt body byte-for-byte before it writes the file. It fails when a color name has no hex mapping, and when a file in `agents/` has no canonical source.

## Validate

    opencode agent list
    opencode debug agent <identifier>
    opencode debug skill
    opencode debug config

References:

- https://opencode.ai/docs/agents/
- https://opencode.ai/docs/config/
- https://opencode.ai/docs/skills/
- https://opencode.ai/docs/rules/
