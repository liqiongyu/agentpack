# Minimal example config repo

This directory is a copy/pasteable example of an Agentpack config repo layout:

- `agentpack.yaml` (manifest)
- `modules/` (local modules: instructions, prompt, skill, and a Claude Code slash command)

It is similar to what `agentpack init` creates, but with `modules:` entries pre-filled so you can see a working end-to-end example.

## Try it (dry / read-only)

- `agentpack --repo docs/examples/minimal_repo doctor`
- `agentpack --repo docs/examples/minimal_repo plan`

## Use it for real

1. Copy this directory to your Agentpack repo path (default: `~/.agentpack/repo`), or point to it via `agentpack --repo <path>`.
2. Edit `targets.codex.options.codex_home` if needed.
3. (Recommended) Install operator assets into the config repo:
   - `agentpack bootstrap --scope project`
4. Run:
   - `agentpack update`
   - `agentpack preview --diff`
   - `agentpack deploy --apply`
