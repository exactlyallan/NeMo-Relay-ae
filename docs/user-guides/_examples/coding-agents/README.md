<!-- SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved. -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Two coding-agent users, one managed Relay deployment

Alice uses Claude Code and Bob uses Codex CLI. Separate Linux accounts, homes,
Relay credentials, task repositories, MCP sessions, and workers share one system
daemon and administrator-owned plugin policy. This is a shared-host deployment;
it does not simulate separate workstation networks or enterprise SSO. The normal
provider credentials remain separate from Relay route credentials.

## Prepare a fresh host

Use an ordinary Brev Ubuntu VM or an equivalent disposable Linux host with
systemd, sudo, Git, and Python 3.11+. Install a verified Relay CLI binary that
includes `daemon managed-bundle`; use the version matching the docs. Install
supported Claude Code and Codex CLI executables so each account can run its
harness. Use the support matrix's minimum versions. Provider access must permit
gateway routing; use an Anthropic API key for Alice and an OpenAI API key for
Bob in the first example. A paid harness subscription alone does not establish
that API access. Do not put either key in the shared configuration.

Start Phoenix on loopback port 6006 on this host, or change the template's OTLP
endpoint to an approved receiving tier. The template enables full payloads only
to inspect the synthetic email redaction: use this example with synthetic tasks.
For an authenticated exporter, supply its environment to each user worker and
add its `header_env` configuration before provisioning.

Review `config.toml`, `plugins.toml`, `nemo-relay.service`, and `deploy.sh`, then:

```bash
sudo bash deploy.sh /absolute/path/to/verified/nemo-relay
```

The script refuses to replace existing accounts and deployment files. It installs
fixed administrator-owned paths, generates an immutable bundle for both
harnesses, creates fresh per-user credentials, and starts the daemon with
`--require-worker`. Incomplete worker startup therefore fails visibly. The
generated digest is outside the bundle. A partial setup failure needs inspection
or a fresh VM rather than blindly rerunning the script.

## Prepare Bob's managed Codex integration

The provisioning script creates an administrator-owned local marketplace beside
the immutable bundle, copies `codex/plugin-v1` including hidden files, and puts
the generated provider configuration and `hooks = true` in Bob's fresh Codex
configuration. As Bob, install the managed plugin:

```bash
codex plugin marketplace add /opt/nvidia/share/nemo-relay-codex-marketplace-v1
codex plugin add nemo-relay-managed-v1@nemo-relay-managed
```

Review and trust the generated hook handlers. `session.sh` supplies the matching
`NEMO_RELAY_CLIENT_TOKEN` for model headers; the plugin listing alone is not
acceptance evidence. Preserve existing authentication and unrelated settings
when adapting the example to an existing account.

## Run independent sessions

Open two terminals. Log into each account, authenticate its provider without
echoing or saving a key in source, then run the installed launcher:

```bash
# Terminal A
sudo -iu alice
# Supply ANTHROPIC_API_KEY through your approved credential method.
/opt/nvidia/share/relay-user-guide/session.sh
```

```bash
# Terminal B
sudo -iu bob
# Supply OPENAI_API_KEY or authenticate Codex for the approved API provider.
/opt/nvidia/share/relay-user-guide/session.sh
```

In both sessions, ask the agent to read
`/opt/nvidia/share/relay-user-guide/task.txt` and complete it in its current
repository. Keep both sessions open for the administrator checks. Approve only
the local file and test operations needed for this task. Alice's launcher loads
both generated Claude artifacts; Bob's installed managed plugin and provider
settings select the same daemon. Each harness starts MCP and its own worker.

## Accept the deployment

While both sessions are active, inspect:

```bash
curl -fsS http://127.0.0.1:47632/_nemo_relay/v1/workers/status
ps -eo user,pid,args | rg 'nemo-relay daemon (mcp|worker)'
sudo journalctl -u nemo-relay.service -n 50 --no-pager
```

Require two distinct ready worker IDs and worker processes owned by Alice and
Bob. The status endpoint reports assignments, not a live worker probe or a
user directory. Correlate the sessions with their run times and lifecycle logs.
In Phoenix, inspect `relay-managed-coding-agents`: require the model/tool activity
supported by each harness and check that `sample.employee@example.com` is
redacted from exported payload fields. The provider conversation and source
files still contain the original synthetic email. Neither a ready worker nor a
successful task proves delivery and redaction by itself.

Review each user's code and tests. Close Alice's harness, allow it to drain,
and confirm Bob can continue with his existing worker. Workers for two sessions
of the same user may be reused; workers for these two users must be distinct.
Telemetry labels are not enterprise identity-provider enrollment or offboarding.

## Apply the files elsewhere

The service, policy, bundle, and user-session boundaries are ordinary Relay
deployment artifacts. Brev only supplies the Linux host. Other supported
harnesses and local or remote daemon topologies are possible. For multiple
workstations, distribute policy and plugin files to every client: workers run
on those clients, and a remote daemon needs a trusted return route to them.
Use organization-managed harness policy for enforcement; Claude's `--settings`
and this reviewable launcher alone do not make its settings immutable.

The example creates disposable accounts and a system service. Use the daemon's
Linux removal runbook or delete the disposable VM after review. Keep credentials
out of screenshots and shared terminal output.

## Proof-of-concept validation

The service and launcher syntax, policy files, and generated managed bundle have been checked locally. The two-user systemd deployment, authenticated harness sessions, and destination redaction check still need a Linux host and provider access.
