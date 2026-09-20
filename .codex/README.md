# Codex project integration

The `.claude` tree holds the canonical repository instructions, skills, and agent prompts. This directory exposes them through Codex configuration and APIs. Treat `.claude` as the source and `.codex` as an adapter.

| Claude source | Codex integration |
|---|---|
| CLAUDE.md files | `config.toml` registers CLAUDE.md and claude.md as native project-instruction fallback names. |
| .claude/agents | `agents/source` is a live mirror. Each generated TOML file registers one project-scoped Codex agent and embeds the complete matching Markdown file in `developer_instructions`. |
| .claude/skills | `skills` is a live mirror. `../.agents/skills` exposes the same folders at the repository skill-discovery path. |
| .claude/hooks | `hooks/source` is a live mirror. `hooks.json` maps the supported events, and `hooks/apply-patch-adapter.sh` translates the Codex `apply_patch` payload to the Claude Edit or Write payload before running the canonical post-edit hook. |
| .claude/settings.json | `config.toml` and `hooks.json` provide Codex-native instruction and lifecycle equivalents. |
| .claude/plan-coordination | A matching symbolic link keeps the plan-store tooling available at the same relative location. |

Keep the `CLAUDE.md` fallback. Adding a root `AGENTS.md` would take precedence over that fallback in the same directory.

Claude `InstructionsLoaded` has no matching Codex lifecycle event, so that observability hook is not registered.

## Regenerate the agents

After any agent prompt is added, removed, renamed, or edited:

    cargo xtask sync-agents

Check the mirror without changing files (the commit gate runs this):

    cargo xtask sync-agents --check

The generator parses every emitted TOML file and requires `developer_instructions` to equal the canonical Markdown source byte-for-byte after decoding.

Project hooks require trust. Open /hooks in Codex to review and trust their exact definitions.

References:

- https://learn.chatgpt.com/docs/agent-configuration/agents-md
- https://learn.chatgpt.com/docs/build-skills
- https://learn.chatgpt.com/docs/agent-configuration/subagents
- https://learn.chatgpt.com/docs/hooks
