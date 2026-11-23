#!/usr/bin/env bash
# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail
mock_root="$(cd -- "$(dirname -- "$0")" && pwd)"
exec cargo +1.96.1 run --quiet --locked --manifest-path "$mock_root/Cargo.toml" --target-dir "${CARGO_TARGET_DIR:-$mock_root/target}" -- "$@"
