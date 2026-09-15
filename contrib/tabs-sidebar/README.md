# Tabs in the Spaces sidebar: test helper and proposal

This folder belongs to the fork branch that adds `ui.sidebar.spaces.show_tabs`.
It is not part of the Herdr release tooling.

- `herdr-tabs`: helper script that builds this branch and runs it in parallel
  with an installed stable Herdr. The test instance keeps its own config,
  server socket, sessions, saved machines, and sidebar preferences under
  `~/.herdr-tabs`. Run it without arguments for usage.
- `INSTALL.md`: prerequisites and install notes, including macOS.
- `PROPOSAL.md`: draft of the upstream idea discussion describing the feature
  and linking this prototype.

Quick start:

```bash
install -m 755 contrib/tabs-sidebar/herdr-tabs ~/.local/bin/herdr-tabs
herdr-tabs build
herdr-tabs run
```
