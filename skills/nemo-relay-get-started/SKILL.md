---
name: nemo-relay-get-started
description: Use this skill when first-time NeMo Relay users want to try Relay, choose the least-complex supported quick start, or verify initial value through the CLI, a maintained integration, or direct Python, Node.js, or Rust instrumentation before production setup.
license: Apache-2.0
metadata:
  author: NVIDIA Corporation and Affiliates
---

# Get Started With NeMo Relay

Guide a new user to visible Relay value with the least complicated applicable
trial. Do not begin with production deployment or Relay's full architecture.

## Choose A Try-Now Path

Evaluate these paths in order. Use the first one that fits the user's stated
goal and existing environment.

1. **CLI try-now (default)**: choose this for a generic "try Relay" request or
   when the user wants value without modifying application code. Run Codex,
   Claude Code, or Hermes through the local CLI wrapper. Read
   [CLI Try-Now](references/cli-try-now.md).
2. **Built-in integrations try-now**: choose this when an existing LangChain,
   LangGraph, Deep Agents, or OpenClaw application owns the execution boundary.
   Prefer the maintained supported integration over manual wrapping. Read
   [Built-In Integrations Try-Now](references/built-in-integrations-try-now.md).
3. **Language-specific manual try-now**: choose this when the user's Python,
   Node.js, or Rust application directly owns its tool or LLM call sites and no
   maintained integration is the better boundary. Read
   [Manual Language Try-Now](references/manual-language-try-now.md).

Do not ask the user to choose among all three when their request, manifest, or
framework already identifies the boundary. For an unspecified request, use the
CLI path. When more than one CLI agent is available, ask one concise question
to select the agent.

## Resolve Installation Without Looping

Select the try-now path before choosing an install package.

- CLI path -> verify `nemo-relay --version`; if missing, use
  `nemo-relay-install` for the CLI outcome.
- Built-in integration path -> use `nemo-relay-install` for the named
  framework or harness package.
- Manual language path -> use `nemo-relay-install` for the detected language
  package.

If installation already succeeded, preserve the chosen path and continue from
its next step. Do not ask the install-path question again or bounce between the
install and get-started skills.

## Apply The Common First-Value Contract

Follow the selected reference, then:

1. Inspect the target environment and existing Relay configuration before
   proposing changes.
2. Explain the attachment boundary and show the exact minimal change.
3. Obtain confirmation before writing configuration, modifying application
   code, or launching a model-consuming run.
4. Use a non-sensitive, read-only trial and exercise one representative tool
   and LLM path when the selected surface exposes both.
5. Verify observable evidence from the selected path rather than treating a
   successful application result as proof that Relay is active.
6. Summarize the captured root, tool, and model relationships without dumping
   prompts, credentials, or complete event payloads.

Explain only the concepts visible in the result: the chosen attachment
boundary, scopes and parentage, captured lifecycle events, and how subscribers
or the Observability plugin make those events inspectable. Keep instrumentation
and export distinct.

## Stop After The Proof

Stop when the selected path's success checks pass. Recommend only the next
relevant workflow:

**Plugin progression:** Present the Observability plugin as the best first
plugin for most new users because it makes Relay activity visible and confirms
that the selected boundary is emitting useful events. After that proof, match
optional built-in plugins to the user's end goal instead of enabling them by
default:

- Adaptive -> adaptive runtime behavior and optimization
- NeMo Guardrails -> policy checks around managed execution
- PII Redaction -> sanitization of sensitive observability payloads
- Model Pricing -> cost estimates for managed LLM responses

Use the [plugin overview](https://docs.nvidia.com/nemo/relay/dev/configure-plugins/about)
to select the next component. Keep each addition incremental and verify its
behavior before layering in another plugin.

Other handoffs:

- Direct application expansion -> `nemo-relay-instrument-calls`
- Additional exporters or durable observability configuration ->
  `nemo-relay-plugin-observability`
- A different package or supported integration -> `nemo-relay-install`
- Persistent Claude Code or Codex loading -> CLI plugin-installation guidance
- Missing hooks, gateway traffic, or events -> `nemo-relay doctor`,
  `nemo-relay doctor --json`, or `nemo-relay-debug-runtime-integration`

Do not configure production OTLP backends, model pricing, guardrails, adaptive
tuning, custom plugins, Go or FFI examples during the quick start. Mention
optional plugins only as end-goal-driven next steps.

## Public Entry Points

- CLI: https://docs.nvidia.com/nemo/relay/dev/nemo-relay-cli/about
- Maintained integrations: https://docs.nvidia.com/nemo/relay/dev/supported-integrations/about
- Language quick starts: https://docs.nvidia.com/nemo/relay/dev/getting-started/quick-start
- Plugin selection: https://docs.nvidia.com/nemo/relay/dev/configure-plugins/about
