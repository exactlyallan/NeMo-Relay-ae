<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Language Package Installation

Use this path when a Python, Node.js, or Rust application directly owns its
tool or model call sites. Preserve the project's package manager and install
only the package needed for the selected language.

## Python

Require Python 3.11 or newer. Prefer `uv` for a project with `pyproject.toml`:

```bash
uv add nemo-relay
```

For an active virtual environment without project metadata:

```bash
uv pip install nemo-relay
```

Use `pip install nemo-relay` only outside `uv`-managed environments. Verify in
the target environment:

```bash
python -c "import nemo_relay"
```

## Node.js

Require Node.js 24 or newer and an existing `package.json`:

```bash
npm install nemo-relay-node
```

Verify that the package and lockfile record `nemo-relay-node`.

## Rust

Require Rust 1.86 or newer and an existing `Cargo.toml`:

```bash
cargo add nemo-relay
```

Add `nemo-relay-adaptive` only when adaptive runtime primitives are already in
scope:

```bash
cargo add nemo-relay-adaptive
```

Verify that `Cargo.toml` includes the selected crate and the lockfile resolves
it. Do not configure adaptive behavior from the install skill.

Public references:

- Prerequisites: https://docs.nvidia.com/nemo/relay/getting-started/prerequisites
- Installation: https://docs.nvidia.com/nemo/relay/getting-started/installation
