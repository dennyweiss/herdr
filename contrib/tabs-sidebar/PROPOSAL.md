# [idea] Show tabs nested under workspaces in the Spaces sidebar

## Problem

I use one workspace per repository and one tab per issue, because my work runs inside a docker compose stack and worktrees do not fit that setup. With three projects and around ten issues in flight, the sidebar only shows the three workspaces. To see which issue tabs exist, or which one needs attention, I have to open the navigator (`prefix+g`) or activate each workspace. The Agents panel helps, but it lists only tabs that currently contain an agent.

## Proposal

An opt-in setting that lists each workspace's tabs as indented rows under the workspace in the expanded Spaces sidebar, reusing the tree style already used for worktree children:

```toml
[ui.sidebar.spaces]
show_tabs = true   # default false
```

```text
  ● acme-api                ▾
    main  ↑2
    ├─ ● #482 oauth callback
    └─ · 2
  ○ storefront              ▾
    develop
    ├─ ◐ #491 tenant migration
    ├─ ● #503 fix telemetry
    └─ ○ #510 spike
  · docs-site               ▸
```

Behavior:

- Tab rows appear only when a workspace has two or more tabs, or a custom-named tab. A single unnamed tab shows nothing, so the default look does not change.
- Each row shows the tab's rolled-up agent status glyph (same glyph set and colors as elsewhere) and its label.
- Left click focuses the tab (`TabFocus`, or endpoint activation with a Tab target for saved SSH machines). Right click opens the existing tab context menu.
- A chevron on the workspace row folds the tab list. Workspaces that already carry the worktree group chevron use a "Hide tabs / Show tabs" context menu entry instead. Folded workspaces are remembered per client next to the collapsed worktree groups.
- Compact rail, mobile layout, navigator, tab bar, keyboard navigation, and the server protocol are unchanged.

## Why this shape

It is a client-side projection only. `ClientShellSnapshot` already carries every tab with `workspace_id`, `label`, `custom_label`, `focused`, and `agent_status`, so no server state, API field, or wire change is needed. `workspace_entries` stays untouched; tab rows are a separate layer, so drag and drop, workspace navigation, and the mobile switcher keep seeing workspaces only.

## Prototype

I have a working implementation on my fork, with tests for rendering, hit testing, clicks, folding and persistence, worktree nesting, and multi-machine rows: https://github.com/dennyweiss/herdr/pull/1

I know unsolicited PRs are closed here, so I am not opening one. If you think the idea fits, I am happy to adapt the prototype to whatever shape you prefer, or for your agents to take it from here.
