# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

"""Compare fixed and routed pi coding tasks through Relay's Switchyard plugin."""

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
import tomllib
from pathlib import Path

TASKS = {
    "ticket-id": (
        "Implement normalize_ticket_id(value) in service.py. Strip surrounding whitespace, accept "
        "case-insensitive INC- followed by exactly four ASCII digits, return the uppercase canonical "
        "ID, and raise ValueError for any other input including non-string values. Write and run "
        "your own tests. Leave retry_delays unchanged."
    ),
    "retry-policy": (
        "Implement retry_delays(attempts, base, cap) in service.py. Return a list with attempts "
        "delays: min(base * 2**i, cap) for i starting at zero. attempts must be a nonnegative "
        "integer (not bool); base and cap must be positive numbers. Raise ValueError for invalid "
        "inputs. Support fractional base and cap. Write and run your own tests. "
        "Leave normalize_ticket_id unchanged."
    ),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="New directory; never modifies your pi setup")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--timeout", type=int, default=600, help="Seconds allowed for each task")
    args = parser.parse_args()
    names = ["NVIDIA_BASE_URL", "EFFICIENT_MODEL", "CAPABLE_MODEL", "SWITCHYARD_MANIFEST"]
    missing = [name for name in names if not os.environ.get(name)]
    if missing:
        parser.error("Set " + ", ".join(missing))
    if not args.prepare_only and not os.environ.get("NVIDIA_API_KEY"):
        parser.error("Set NVIDIA_API_KEY in the launching environment")
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    if not args.prepare_only and any(not shutil.which(name) for name in ["nemo-relay", "pi"]):
        parser.error("Install the Relay CLI and pi versions used by the pi quickstart first")
    source = Path(__file__).resolve().parent
    output = args.output.resolve()
    routes = (source / "routes.toml").read_text()
    for name in ["NVIDIA_BASE_URL", "EFFICIENT_MODEL", "CAPABLE_MODEL", "JUDGE_MODEL"]:
        value = os.environ.get(name) or os.environ["EFFICIENT_MODEL"]
        routes = routes.replace(json.dumps(f"__{name}__"), json.dumps(value))
    tomllib.loads(routes)
    output.mkdir(parents=True, exist_ok=False)
    (output / "routes.toml").write_text(routes)
    environment = os.environ.copy()
    environment["PI_CODING_AGENT_DIR"] = str(output / "pi")
    environment["XDG_CONFIG_HOME"] = str(output / "config")
    (output / "pi").mkdir()
    # A custom provider keeps the virtual route IDs out of pi's NVIDIA model catalog.
    provider = {
        "baseUrl": os.environ["NVIDIA_BASE_URL"],
        "api": "openai-completions",
        "apiKey": "relay-route-placeholder",
        "compat": {"supportsDeveloperRole": False},
        "models": [
            {
                "id": f"switchyard/{arm}",
                "name": arm,
                "reasoning": False,
                "input": ["text"],
                "contextWindow": 32768,
                "maxTokens": 4096,
            }
            for arm in ["baseline", "efficient", "routed"]
        ],
    }
    (output / "pi/models.json").write_text(json.dumps({"providers": {"relay-nvidia": provider}}, indent=2) + "\n")
    results = []
    for arm in ["baseline", "efficient", "routed"]:
        arm_dir = output / arm
        arm_dir.mkdir()
        (arm_dir / "config.toml").write_text('[agents.pi]\ncommand = "pi"\n')
        plugins = f"""version = 1

[[components]]
kind = "observability"
enabled = true
[components.config]
version = 4
enable_full_payloads = false
[components.config.atof]
enabled = true
[[components.config.atof.sinks]]
type = "file"
output_directory = {json.dumps(str(arm_dir / "logs"))}
filename = "events.jsonl"
mode = "append"
[components.config.opentelemetry]
enabled = false

[[plugins.dynamic]]
manifest = {json.dumps(str(Path(os.environ["SWITCHYARD_MANIFEST"]).resolve()))}
[plugins.dynamic.config]
priority = 0
switchyard_config_path = {json.dumps(str(output / "routes.toml"))}
[plugins.policy.overrides."nvidia.switchyard"]
attestation = "integrity_only"
"""
        tomllib.loads(plugins)
        (arm_dir / "plugins.toml").write_text(plugins)
        for task in TASKS:
            task_dir = arm_dir / task
            task_dir.mkdir()
            shutil.copyfile(source / "service.py", task_dir / "service.py")
    print(f"Prepared example: {output}")
    if args.prepare_only:
        return
    subprocess.run(["nemo-relay", "install", "pi"], env=environment, check=True)
    for arm in ["baseline", "efficient", "routed"]:
        arm_dir = output / arm
        command = ["nemo-relay", "--config", str(arm_dir / "config.toml")]
        # Dynamic manifests are added disabled. Explicit lifecycle enablement is required.
        subprocess.run(command + ["plugins", "enable", "nvidia.switchyard"], env=environment, check=True)
        subprocess.run(command + ["plugins", "validate", "nvidia.switchyard"], env=environment, check=True)
        for task, prompt in TASKS.items():
            task_dir = arm_dir / task
            started = time.monotonic()
            with (task_dir / "agent.log").open("w") as log:
                try:
                    run = subprocess.run(
                        command
                        + [
                            "pi",
                            "--",
                            "--provider",
                            "relay-nvidia",
                            "--model",
                            f"switchyard/{arm}",
                            "--no-session",
                            "-p",
                            prompt + " Use only local files and tools in this directory.",
                        ],
                        cwd=task_dir,
                        env=environment,
                        stdout=log,
                        stderr=subprocess.STDOUT,
                        timeout=args.timeout,
                    )
                    status = run.returncode
                except subprocess.TimeoutExpired:
                    status = "timeout"
            seconds = round(time.monotonic() - started, 3)
            try:
                check = subprocess.run(
                    [sys.executable, str(source / "acceptance.py"), str(task_dir), task],
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                accepted = check.returncode == 0
                (task_dir / "acceptance.log").write_text(check.stdout + check.stderr)
            except subprocess.TimeoutExpired:
                accepted = False
                (task_dir / "acceptance.log").write_text("Acceptance check timed out\n")
            results.append(
                {
                    "arm": arm,
                    "task": task,
                    "agent_status": status,
                    "accepted": status == 0 and accepted,
                    "seconds": seconds,
                }
            )
            (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))
    print("Inspect logs/events.jsonl for routing decisions, served models, usage, and fallback. No prices applied.")
    if any(not row["accepted"] for row in results):
        raise SystemExit("Some tasks failed; they remain in results.json for the comparison")


if __name__ == "__main__":
    main()
