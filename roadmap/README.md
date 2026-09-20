# Roadmaps

One directory per plan: `roadmap/<plan>/`. The path is the plan's identity.

- The plan store for `roadmap/<plan>` lives at `~/.claude/plan-dbs/duet__roadmap-<plan>/` and is opened with `.claude/plan-coordination/db.sh init roadmap/<plan>`. Always pass the repo-root-relative path, never `.`.
- Static documents (the plan graph, specs, ADRs) are markdown here and are git-tracked.
- Fluid state (next steps, work items, findings, reports) lives in the store, never in markdown.
- A throwaway probe path such as `roadmap/_hv-probe` is reserved for the Hypervisor Bootstrap Gate; nothing is committed under it.

No plan exists yet. Create the first one in Plan Mode with the Hypervisor or an Orchestrator.
