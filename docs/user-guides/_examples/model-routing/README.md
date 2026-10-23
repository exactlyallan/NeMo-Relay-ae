<!-- SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved. -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Compare pi coding tasks with NVIDIA model routing

This companion uses **pi → Relay CLI gateway + Switchyard native plugin → NVIDIA
inference**. It extends the pi quickstart's installed extension, wrapped run,
OpenAI-compatible request format, and local ATOF output. It uses the native
plugin inside Relay, rather than Switchyard's standalone evaluation server.

The same source runs on a Brev CPU VM and a comparable Linux test host with
external model access. Clone the guide branch and run from this directory. A
premade Launchable and a GPU are unnecessary when inference is remote. This is
a small comparison fixture; use a larger representative workload before making
a production model-policy decision.

## Prerequisites and choices

Install pi 0.84.0 and Relay 0.11.0 as in the pi quickstart, with Node.js 24 and
Python 3.11+. Obtain and verify a compatible Switchyard plugin bundle for the
host architecture, then set `SWITCHYARD_MANIFEST` to its completed
`relay-plugin.toml`. Native plugins run inside Relay: inspect and trust the
bundle before loading it. Published bundles may require NVIDIA repository
access; use Switchyard's documented source-build path if needed. The script
does not download arbitrary native code or bypass a failed plugin validation.

Supply these variables through the launching environment:

| Variable | Meaning |
| --- | --- |
| `NVIDIA_BASE_URL` | Your NVIDIA endpoint's OpenAI Chat Completions base URL, including `/v1` |
| `NVIDIA_API_KEY` | Credential for that endpoint, used by the Switchyard plugin |
| `EFFICIENT_MODEL` | Tool-capable efficient Nemotron model ID served there |
| `CAPABLE_MODEL` | Tool-capable capable Nemotron model ID served there |
| `JUDGE_MODEL` | Optional; defaults to the efficient model and must support the configured JSON-object verdict |
| `SWITCHYARD_MANIFEST` | Absolute path to the verified native-plugin manifest |

Select exact model IDs from your accessible endpoint. Nano and Super are
candidates, not a tested pairing or a guaranteed quality ranking. Both targets
must support pi's tools and a context of at least 32768 tokens with 4096 output
tokens; adjust the generated `pi/models.json` limits for your actual models.
The judge uses `json_object` and Switchyard's local schema validation. Confirm
judge success in telemetry: failure can route every call to the capable model,
making a successful answer an inadequate routing check.

```bash
# Inspect configuration and task files without loading a plugin or calling a model.
python3 run.py --output /tmp/relay-routing-config --prepare-only

# With endpoint access, model IDs, and the verified bundle supplied:
python3 run.py --output /tmp/relay-routing-comparison
```

Use new output directories. The runner isolates pi's configuration with
`PI_CODING_AGENT_DIR` and Relay's user configuration with `XDG_CONFIG_HOME`.
It preserves the host's administrator policy. Secrets remain environment values;
the generated files contain only the key's variable name. The generated trust
override accepts digest-verified unsigned bundles; signed enterprise builds can
use a stricter policy.

## What runs

Three arms use the same pi model metadata, Relay gateway, native plugin, tools,
task inputs, and acceptance checks:

1. `baseline`: a Switchyard passthrough route always selects the capable model.
2. `efficient`: a passthrough route always selects the efficient model.
3. `routed`: a capability classifier selects between them, starting at threshold 0.8.

Each arm starts from the same source scaffold in a fresh task directory. One
task implements an incident-ID normalizer; another implements a capped retry
policy. The agent writes and runs its own tests. Separate acceptance checks run
afterward from this source directory, and `results.json` retains every task's
agent exit status, acceptance result, and elapsed time. This is not a sandboxed
benchmark; run only trusted tasks in the disposable example environment. A
nonzero result remains evidence to investigate, rather than disappearing from
the comparison.

## Inspect the result

Inspect each arm's `logs/events.jsonl` for model/tool activity and Switchyard
routing marks. Compare the requested route, selected target, served model,
fallback flag, routing overhead, and available token usage. The virtual provider
name `relay-nvidia` and virtual model IDs keep pi's route selection distinct from
the actual NVIDIA targets. The plugin owns the NVIDIA credential, so pi's custom
provider uses an inert placeholder key. If the plugin is disabled, that placeholder
must not be mistaken for working NVIDIA authentication.

`results.json` reports task acceptance and latency. It does not infer routing
success from acceptance, calculate savings, or apply prices. Review the trace
evidence separately and repeat with a larger workload. A routing run need not
select both models; if it always selects one, inspect judge errors and task mix
before interpreting that behavior.

## Adapt the setup

pi and NVIDIA-hosted models are the concrete example choices. Other
Relay-supported harnesses, compatible model endpoints, self-hosted NIMs, and
routing algorithms are possible. Keep the baseline, tools, acceptance criteria,
and evidence comparable when changing them. For an administrator-managed daemon,
install the bundle and policy in the worker's system configuration and load the
managed harness integration. Do not deploy this disposable personal configuration
as organization policy.

## Proof-of-concept validation

Configuration generation, pi 0.84.0 model discovery, acceptance checks, and runner success/failure handling have been checked locally. A live routing comparison still requires a compatible native-plugin bundle and accessible model endpoints.
