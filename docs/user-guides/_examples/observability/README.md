<!-- SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved. -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Observe a support-triage agent

This companion to **Connecting to Observability Platforms** uses Hermes Agent's
native Relay integration to read a synthetic incident queue and write a handoff.
The same files run on a Brev CPU VM or a Linux development host. No Launchable is
required. Use a fresh directory for every run; the example creates its own files.

## Prepare a host

Use Python 3.11 or newer and a Hermes installation with native Relay support and
`chat --oneshot`. Install and authenticate Hermes using its installation guide,
select a tool-capable model, and inspect the NeMo Relay Plugins section of
`hermes doctor`. Hermes owns its Relay dependency; installing the Relay CLI is
not required for this example. Use a dedicated Hermes installation/profile with
no unrelated gateway session, and check that administrator plugin policy permits
the selected exporter. A separately running Hermes gateway can retain an earlier
configuration; do not stop a gateway used by other work.

In Brev, create an ordinary CPU VM, clone this guide branch, and use the same
instructions below from this directory. The instance supplies the host; the
workload and Relay configuration do not depend on Brev APIs. Keep provider and
destination keys in the process environment, outside the repository.

## Choose a destination

| Selection | `RELAY_OTLP_ENDPOINT` | Additional environment |
| --- | --- | --- |
| `phoenix` | Defaults to `http://127.0.0.1:6006/v1/traces` for an existing local Phoenix | `PHOENIX_AUTHORIZATION` containing `Bearer <api-key>` only if the intake requires it |
| `langsmith` | Your regional intake, e.g. `https://api.smith.langchain.com/otel/v1/traces` | `LANGSMITH_API_KEY`, `LANGSMITH_PROJECT` |
| `weave` | `https://trace.wandb.ai/agents/otel/v1/traces` | `WANDB_API_KEY`, `WANDB_ENTITY`, `WANDB_PROJECT` |
| `datadog` | Your running Collector's complete `/v1/traces` URL | Configure the Collector's Datadog exporter and credentials separately |

Provision Phoenix or the destination receiving tier before running. The script
does not install a server. The destination sections in the guide explain the
intake and UI checks. For an authenticated Collector, add its approved
`header_env` mapping to the generated file after `--prepare-only`; run Hermes
with `HERMES_NEMO_RELAY_PLUGINS_TOML` pointing to that file.

```bash
# Inspect a generated configuration without calling a model or exporter.
python3 run.py --destination phoenix --output /tmp/relay-triage-config --prepare-only

# After setting provider credentials and starting Phoenix:
python3 run.py --destination phoenix --output /tmp/relay-triage-phoenix
```

Select another destination with the same script and a new output directory.
The template renders secrets as environment references, never literal keys.

## Inspect the result

The local check requires all three incident IDs in `handoff.md`, the checkout
incident first, and model/tool scope events in `logs/events.jsonl`. Review the
actual triage decisions as well: these checks do not grade every recommendation.
In the destination UI, find service `relay-support-triage` at the run time and
check the agent/model/tool parentage against those local events. Local output
alone does not prove remote delivery. Payload capture is disabled; inspect the
exported fields permitted by your policy.

The example fixes the workload so destination differences are easier to inspect.
In an enterprise deployment, replace Hermes with your supported capture path,
the synthetic queue with your approved task, and the endpoint and service name
with your receiving tier. Retain the same trace verification.

## Proof-of-concept validation

Configuration generation for all four destinations and local runner syntax have been checked. A live Hermes run and destination UI verification still require an authenticated model and receiving platform.
