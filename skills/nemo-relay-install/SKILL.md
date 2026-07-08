---
name: nemo-relay-install
description: Use when choosing or running NeMo Relay package installation for the CLI, Python, Node.js, Rust, OpenClaw, Hermes, or maintained framework integrations before runtime configuration or quick-start setup
author: NVIDIA Corporation and Affiliates
license: Apache-2.0
---

# Install NeMo Relay

Use this skill when a user needs the correct NeMo Relay package, CLI, or
integration install path for an existing app, shell, runtime, or agent harness.

Stop after installation and basic availability checks. Do not configure runtime
behavior, write `plugins.toml`, create scopes, register middleware, or build a
first app example from this skill.

## Choose The Install Path

If the user asks to install NeMo Relay but does not identify a target, ask one
short clarifying question before giving commands:

> Which install path do you want: CLI for coding-agent/local gateway use,
> language package for a Python/Node.js/Rust app, or framework integration for
> LangChain, LangGraph, Deep Agents, OpenClaw, or Hermes?

Do not ask when the user already names a CLI, language, framework, harness,
source checkout, or target project file such as `pyproject.toml`, `package.json`,
or `Cargo.toml`.

- **CLI executable**: user wants the `nemo-relay` command for coding-agent hooks,
  gateway observability, or host-plugin installation.
- **Python package**: app owns tool or LLM call sites through the Python wrapper.
- **Node.js package**: app owns call sites through the JavaScript API.
- **Rust crates**: app uses NeMo Relay directly from Rust.
- **OpenClaw plugin**: OpenClaw owns the agent harness boundary.
- **Hermes plugin**: Hermes owns the agent runtime environment.
- **Python framework extras**: app uses maintained LangChain, LangGraph, or Deep
  Agents integrations.
- **Source checkout**: user is validating unpublished changes or contributing.
  Use repository development setup instead of package-manager install commands.

## Prerequisite Check

- Rust: 1.86 or newer.
- Python: 3.11 or newer.
- Node.js: 24 or newer.
- `uv`: prefer for Python project dependency management.

The primary documented install paths are Rust, Python, and Node.js. Treat Go and
raw FFI as source-first advanced surfaces, not normal package install paths.

## Install Commands

Use unpinned package commands for the latest compatible release unless the user
or project asks for an exact version.

### CLI

Supported Unix-like shell:

```bash
curl -fsSL https://raw.githubusercontent.com/NVIDIA/NeMo-Relay/main/install.sh | sh
nemo-relay --version
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/NVIDIA/NeMo-Relay/main/install.ps1 | iex
nemo-relay --version
```

The installer verifies the published checksum before replacing an existing
binary and does not invoke `sudo`.

Use Cargo when the user prefers a source build or needs an unsupported platform:

```bash
cargo install nemo-relay-cli
```

### Python

Project with `pyproject.toml`:

```bash
uv add nemo-relay
```

Active virtual environment without project metadata:

```bash
uv pip install nemo-relay
```

Use `pip install nemo-relay` only outside `uv`-managed environments.

### Node.js

```bash
npm install nemo-relay-node
```

### Rust

```bash
cargo add nemo-relay
```

Add `nemo-relay-adaptive` only when adaptive runtime primitives are already in
scope:

```bash
cargo add nemo-relay-adaptive
```

### Integrations

OpenClaw:

```bash
openclaw plugins install npm:nemo-relay-openclaw
openclaw gateway restart
```

Hermes:

```bash
pip install nemo-relay
hermes plugins enable observability/nemo_relay
```

LangChain, LangGraph, or Deep Agents:

```bash
uv add "nemo-relay[langchain,langgraph,deepagents]"
```

## Verify Installation Only

- CLI: run `nemo-relay --version`.
- Python: check that `python -c "import nemo_relay"` succeeds in the target
  environment.
- Node.js: check that the package is recorded in `package.json` or the lockfile.
- Rust: check that `Cargo.toml` includes the selected crate and the lockfile
  resolves it.
- OpenClaw and Hermes: check that their own plugin manager reports the plugin
  installed or enabled.

Do not treat a first scope, subscriber, gateway, plugin config, or LLM call as
part of installation verification.

## Hand Off After Install

- For a first working scope, tool call, or LLM call -> `nemo-relay-get-started`
- For local CLI host-plugin workflow -> NeMo Relay CLI docs or host-specific
  setup, not app runtime setup
- For runtime configuration, plugin files, observability, or adaptive behavior
  -> the matching plugin or instrumentation skill

## Common Mistakes

- Using repository development setup when the user only needs a published
  package.
- Installing the CLI when the user needs an application binding, or installing a
  binding when the user only needs the local `nemo-relay` executable.
- Pinning old versions unless the user or project explicitly requires that
  version.
- Continuing into `plugins.toml`, middleware registration, scopes, or quick-start
  examples before the install step has been verified.
