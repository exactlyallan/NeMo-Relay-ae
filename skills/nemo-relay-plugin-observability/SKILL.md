---
name: nemo-relay-plugin-observability
description: Use this skill when choosing or configuring NeMo Relay observability through the built-in plugin, subscribers, or exporters, including raw ATOF events, ATIF trajectories, OpenTelemetry traces, OpenInference export, or custom event handling.
license: Apache-2.0
metadata:
  author: NVIDIA Corporation and Affiliates
---

# Configure Observability Plugins

Start with one exporter managed by the built-in Observability plugin. This is
the default for reusable process configuration and the best first plugin for
most users because it makes Relay's captured activity visible.

Use manual subscriber or exporter APIs only when a test, script, or application
needs direct control over registration names, collection windows, or flush
timing. Both paths consume the same canonical event stream.

## Choose The Output

- **Console or custom event handling**
  Use a manual subscriber for short-lived in-process inspection.
- **Raw canonical lifecycle events**
  Use ATOF JSONL; read `references/atof.md`.
- **Portable execution trajectories**
  Use ATIF; read `references/atif.md`.
- **General OTLP tracing**
  Use OpenTelemetry; read `references/opentelemetry.md`.
- **OpenInference-aware backends**
  Use OpenInference; read `references/openinference.md`.

Choose one output first and verify it before adding another. ATOF is the
default local proof because it preserves the raw event stream with the least
translation. Add sanitization before exporters receive sensitive payloads.

## Embedded Event And Subscriber Model

- NeMo Relay emits one canonical event stream from scopes, marks, managed tool
  calls, managed LLM calls, middleware, and manual lifecycle APIs.
- Subscribers consume events without defining the event model. Multiple
  subscribers can observe the same stream for logging, export, analytics, or
  diagnostics.
- Global subscribers remain active process-wide until removed.
- Scope-local subscribers are owned by one active scope and disappear when that
  scope closes.
- Plugin-installed subscribers are reusable, configuration-driven runtime
  components.
- Exporter-oriented subscribers preserve raw ATOF or translate the event stream
  into ATIF, OpenTelemetry, or OpenInference output.
- Event payloads reflect sanitized post-guardrail input and output when calls use
  managed helpers or manual lifecycle params provide those fields.
- Event fields include semantic input/output through the ATOF `data` field,
  typed profile data such as `model_name` and `tool_call_id`, and codec-provided
  annotated LLM request/response data for in-process subscribers and exporters.

## Shared Lifecycle

1. Create the exporter or subscriber.
2. Register it with a unique name before the relevant scoped work.
3. Run NeMo Relay-instrumented work inside scopes.
4. Flush if deterministic delivery is needed and the binding supports it.
5. Deregister it, then shut it down when the process or subsystem is done.

## Binding Names

- Python: `nemo_relay.subscribers.register(...)`,
  `AtofExporter`, `AtifExporter`, `OpenTelemetrySubscriber`, and
  `OpenInferenceSubscriber`
- Node.js: root exports `registerSubscriber(...)`, `AtofExporter`,
  `AtifExporter`, `OpenTelemetrySubscriber`, and `OpenInferenceSubscriber`
- Rust: `nemo_relay::api::subscriber` and `nemo_relay::observability::*`
- Go: source-first wrappers expose equivalent register, exporter, and subscriber
  lifecycle methods

## Load A Reference When

- You need raw JSONL events for local debugging or offline inspection ->
  `references/atof.md`
- You already know you need ATIF trajectories -> `references/atif.md`
- You already know you need OTLP/OpenTelemetry traces ->
  `references/opentelemetry.md`
- You already know you need OpenInference semantic traces ->
  `references/openinference.md`

## Use Another Skill When

- You need to package subscriber-based export behavior as a reusable plugin ->
  `nemo-relay-plugin-build`
- You have not instrumented a scope, tool call, or LLM call yet ->
  `nemo-relay-get-started` or `nemo-relay-instrument-calls`
- You are debugging missing telemetry -> `nemo-relay-debug-runtime-integration`

## Related Skills

- `nemo-relay-instrument-calls`
- `nemo-relay-instrument-typed-wrappers`
- `nemo-relay-plugin-build`
- `nemo-relay-debug-runtime-integration`
