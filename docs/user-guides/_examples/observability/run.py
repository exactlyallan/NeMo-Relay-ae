# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

"""Run the same Hermes support-triage scenario against one OTLP destination."""

import argparse
import json
import os
import shutil
import subprocess
import tomllib
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", choices=["phoenix", "langsmith", "weave", "datadog"], required=True)
    parser.add_argument("--output", type=Path, required=True, help="New directory for this run")
    parser.add_argument("--prepare-only", action="store_true", help="Write configuration without starting Hermes")
    args = parser.parse_args()
    directory = args.output.resolve()
    endpoint = os.environ.get("RELAY_OTLP_ENDPOINT", "http://127.0.0.1:6006/v1/traces")
    if args.destination != "phoenix" and not os.environ.get("RELAY_OTLP_ENDPOINT"):
        parser.error("Set RELAY_OTLP_ENDPOINT to the destination's complete trace URL")
    if not endpoint.startswith(("https://", "http://127.0.0.1:", "http://localhost:")):
        parser.error("Use HTTPS for a destination outside this host")
    source = Path(__file__).resolve().parent
    config = (source / "plugins.toml").read_text()
    for placeholder, value in {
        "__LOG_DIRECTORY__": str(directory / "logs"),
        "__PROJECTION__": "gen_ai" if args.destination == "weave" else "openinference",
        "__TRACE_ENDPOINT__": endpoint,
    }.items():
        config = config.replace(json.dumps(placeholder), json.dumps(value))
    required = []
    if args.destination == "phoenix" and os.environ.get("PHOENIX_AUTHORIZATION"):
        config += '\n[components.config.opentelemetry.endpoints.header_env]\nauthorization = "PHOENIX_AUTHORIZATION"\n'
    elif args.destination == "langsmith":
        required = ["LANGSMITH_API_KEY", "LANGSMITH_PROJECT"]
        config += '\n[components.config.opentelemetry.endpoints.header_env]\n"x-api-key" = "LANGSMITH_API_KEY"\n'
        config += '\n[components.config.opentelemetry.endpoints.headers]\n"Langsmith-Project" = '
        config += json.dumps(os.environ.get("LANGSMITH_PROJECT", "relay-support-triage")) + "\n"
    elif args.destination == "weave":
        required = ["WANDB_API_KEY", "WANDB_ENTITY", "WANDB_PROJECT"]
        config += '\n[components.config.opentelemetry.endpoints.header_env]\n"wandb-api-key" = "WANDB_API_KEY"\n'
        config += "\n[components.config.opentelemetry.endpoints.resource_attributes]\n"
        config += '"wandb.entity" = ' + json.dumps(os.environ.get("WANDB_ENTITY", "")) + "\n"
        config += '"wandb.project" = ' + json.dumps(os.environ.get("WANDB_PROJECT", "")) + "\n"
    # Datadog credentials belong to the receiving Collector, not this process.
    if not args.prepare_only:
        missing = [name for name in required if not os.environ.get(name)]
        if missing:
            parser.error("Set " + ", ".join(missing))
        if not shutil.which("hermes"):
            parser.error("Install and authenticate Hermes with native Relay support first")
    tomllib.loads(config)
    directory.mkdir(parents=True, exist_ok=False)
    (directory / "plugins.toml").write_text(config)
    shutil.copyfile(source / "requests.csv", directory / "requests.csv")
    print(f"Example files: {directory}")
    if args.prepare_only:
        return
    environment = os.environ.copy()
    environment["HERMES_NEMO_RELAY_PLUGINS_TOML"] = str(directory / "plugins.toml")
    subprocess.run(
        [
            "hermes",
            "chat",
            "--oneshot",
            "-q",
            (
                "Read requests.csv using a tool. Prioritize the incidents by customer impact and create "
                "handoff.md with a short action for each request ID. Put INC-1042 first. "
                "Use only the supplied synthetic data and local file tools; do not contact outside services."
            ),
        ],
        cwd=directory,
        env=environment,
        check=True,
    )
    handoff = (directory / "handoff.md").read_text()
    ids = ["INC-1042", "INC-1043", "INC-1044"]
    if any(item not in handoff for item in ids) or handoff.index(ids[0]) > min(handoff.index(i) for i in ids[1:]):
        raise SystemExit("Handoff did not include every request with INC-1042 first")
    events = [json.loads(line) for line in (directory / "logs/events.jsonl").read_text().splitlines() if line.strip()]
    categories = {event.get("category") for event in events if event.get("kind") == "scope"}
    if not {"llm", "tool"} <= categories:
        raise SystemExit("Local capture is incomplete: inspect Hermes Relay diagnostics and events.jsonl")
    print("Handoff and local model/tool events found. Now inspect the correlated trace in your destination UI.")


if __name__ == "__main__":
    main()
