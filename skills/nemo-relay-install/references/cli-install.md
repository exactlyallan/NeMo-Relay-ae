<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# CLI Installation

Use this path for the `nemo-relay` executable, a temporary coding-agent run,
local gateway use, or explicit persistent host-plugin installation.

## Check Prerequisites

- Confirm the operating system and architecture have a published CLI asset.
- Use Cargo when the user prefers a source build or needs an unsupported
  platform.
- For a transparent run, confirm the selected `codex`, `claude`, or `hermes`
  command is already on `PATH`.

## Install

For a supported Unix-like shell:

```bash
curl -fsSL https://raw.githubusercontent.com/NVIDIA/NeMo-Relay/main/install.sh | sh
```

For Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/NVIDIA/NeMo-Relay/main/install.ps1 | iex
```

The published installer verifies the checksum before replacing an existing
binary and does not invoke `sudo`.

For a source build:

```bash
cargo install nemo-relay-cli
```

## Verify

Run:

```bash
nemo-relay --version
```

For transparent-run readiness, optionally preview the generated temporary hook
configuration, gateway environment, gateway URL, and final command:

```bash
nemo-relay run --agent <agent> --dry-run --print
```

After installation, hand a generic trial to `nemo-relay-get-started`. Its
default path launches the selected coding agent with `nemo-relay codex`,
`nemo-relay claude`, `nemo-relay hermes`, or `nemo-relay run -- <command>`.
The wrapper is temporary for that process.

Use persistent host-plugin installation only when the user explicitly wants
Claude Code or Codex to load Relay through the host plugin system. Validate that
path with `nemo-relay doctor --plugin <host>`.

Public references:

- Installation: https://docs.nvidia.com/nemo/relay/getting-started/installation
- Transparent run: https://docs.nvidia.com/nemo/relay/dev/nemo-relay-cli/basic-usage#transparent-run
