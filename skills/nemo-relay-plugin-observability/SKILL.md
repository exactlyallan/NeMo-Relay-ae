---
name: nemo-relay-plugin-observability
description: Use when choosing or configuring NeMo Relay observability plugins, subscribers, or exporters, including custom event handling, ATIF trajectories, OpenTelemetry OTLP traces, or OpenInference export
author: NVIDIA Corporation and Affiliates
license: Apache-2.0
---


# Configure Observability Plugins

Use this skill when an application developer wants visibility into NeMo Relay
activity, needs to choose an observability output, or needs to configure a
built-in observability subscriber or exporter.

## Choose The Output

- **Console or custom event handling**
  Use subscribers.
- **Portable execution trajectories**
  Use `AtifExporter`; read `references/atif.md`.
- **General OTLP tracing**
  Use the OpenTelemetry subscriber; read `references/opentelemetry.md`.
- **OpenInference-aware backends**
  Use the OpenInference subscriber; read `references/openinference.md`.

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
- Exporter-oriented subscribers translate the event stream into ATIF,
  OpenTelemetry, or OpenInference output.
- Event payloads reflect sanitized post-guardrail input and output when calls use
  managed helpers or manual lifecycle params provide those fields.
- Event fields include semantic input/output through the ATOF `data` field,
  typed profile data such as `model_name` and `tool_call_id`, and codec-provided
  annotated LLM request/response data for in-process subscribers and exporters.

## Shared Lifecycle

1. Create the exporter or subscriber.
2. Register it with a unique name before the relevant scoped work.
3. Run NeMo Relay-instrumented work inside scopes.
4. Deregister it.
5. Flush or shut down if the binding supports it and deterministic delivery is needed.

## Binding Names

- Python: `nemo_relay.subscribers.register(...)`,
  `AtifExporter`, `OpenTelemetrySubscriber`, and `OpenInferenceSubscriber`
- Node.js: root exports `registerSubscriber(...)`, `AtifExporter`,
  `OpenTelemetrySubscriber`, and `OpenInferenceSubscriber`
- Rust: `nemo_relay::api::subscriber` and `nemo_relay::observability::*`
- Go: source-first wrappers expose equivalent register, exporter, and subscriber
  lifecycle methods

## Load A Reference When

- You already know you need ATIF trajectories -> `references/atif.md`
- You already know you need OTLP/OpenTelemetry traces ->
  `references/opentelemetry.md`
- You already know you need OpenInference semantic traces ->
  `references/openinference.md`

## Use Another Skill When

- You need to package subscriber-based export behavior as a reusable plugin ->
  `nemo-relay-plugin-build`
- You are debugging missing telemetry -> `nemo-relay-debug-runtime-integration`

## Related Skills

- `nemo-relay-instrument-calls`
- `nemo-relay-instrument-typed-wrappers`
- `nemo-relay-plugin-build`
- `nemo-relay-debug-runtime-integration`
