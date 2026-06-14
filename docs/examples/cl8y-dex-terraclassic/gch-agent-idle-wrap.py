#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run Cursor agent; SIGTERM when stdout is idle (CLI hang workaround)."""
from __future__ import annotations

import json
import os
import select
import signal
import subprocess
import sys
import time

META_FILE = os.environ.get(
    "GCH_COMPLETE_META_FILE", "/tmp/gch-agent-complete-meta.json"
)


def classify_line(line: bytes) -> str:
    """Return activity kind for idle tracking."""
    stripped = line.strip()
    if not stripped:
        return "activity"
    try:
        obj = json.loads(stripped)
    except json.JSONDecodeError:
        if b'"type":"thinking"' in stripped and b'"subtype":"completed"' in stripped:
            return "short_idle_thinking"
        if b'"type":"tool_call"' in stripped and b'"subtype":"completed"' in stripped:
            return "tool_call_completed"
        if b'"type":"tool_call"' in stripped and b'"subtype":"started"' in stripped:
            return "tool_call_started"
        return "activity"
    if obj.get("type") == "thinking" and obj.get("subtype") == "completed":
        return "short_idle_thinking"
    if obj.get("type") == "tool_call":
        subtype = obj.get("subtype")
        if subtype == "completed":
            return "tool_call_completed"
        if subtype == "started":
            return "tool_call_started"
    return "activity"


def feed_lines(buf: bytes, chunk: bytes) -> tuple[bytes, list[str]]:
    buf += chunk
    kinds: list[str] = []
    while b"\n" in buf:
        line, buf = buf.split(b"\n", 1)
        kinds.append(classify_line(line))
    return buf, kinds


def apply_stream_kinds(
    kinds: list[str],
    in_flight_tools: int,
    use_short_idle: bool,
    last_short_idle_source: str | None,
) -> tuple[int, bool, str | None]:
    """Update idle state from parsed stream-json event kinds."""
    for kind in kinds:
        if kind == "tool_call_started":
            in_flight_tools += 1
        elif kind == "tool_call_completed":
            in_flight_tools = max(0, in_flight_tools - 1)
            if in_flight_tools == 0:
                use_short_idle = True
                last_short_idle_source = "tool_call"
        elif kind == "short_idle_thinking":
            if in_flight_tools == 0:
                use_short_idle = True
                last_short_idle_source = "thinking"
    return in_flight_tools, use_short_idle, last_short_idle_source


def idle_limit_secs(
    in_flight_tools: int,
    use_short_idle: bool,
    idle_long: int,
    idle_after_thinking: int,
) -> int:
    """Long idle while any tool is in flight (e.g. forge test with silent stdout)."""
    if in_flight_tools > 0:
        return idle_long
    if use_short_idle:
        return idle_after_thinking
    return idle_long


def write_complete_meta(meta: dict) -> None:
    with open(META_FILE, "w", encoding="utf-8") as f:
        json.dump(meta, f)


def main() -> int:
    if len(sys.argv) < 5:
        print(
            "usage: gch-agent-idle-wrap.py <idle_secs> <after_thinking_secs> <max_secs> -- <cmd...>",
            file=sys.stderr,
        )
        return 2

    idle_long = int(sys.argv[1])
    idle_after_thinking = int(sys.argv[2])
    max_secs = int(sys.argv[3])
    rest = sys.argv[4:]
    if not rest or rest[0] != "--":
        print("missing -- before command", file=sys.stderr)
        return 2
    cmd = rest[1:]
    if not cmd:
        print("missing command", file=sys.stderr)
        return 2

    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    assert proc.stdout is not None

    deadline = time.monotonic() + max_secs
    last_output = time.monotonic()
    saw_output = False
    use_short_idle = False
    last_short_idle_source: str | None = None
    in_flight_tools = 0
    line_buf = b""
    fd = proc.stdout.fileno()

    while proc.poll() is None:
        now = time.monotonic()
        if now >= deadline:
            proc.kill()
            proc.wait(timeout=30)
            print("gch-agent-idle-wrap: max runtime exceeded", file=sys.stderr)
            write_complete_meta(
                {
                    "reason": "max_runtime",
                    "max_secs": max_secs,
                }
            )
            return 124

        idle_limit = idle_limit_secs(
            in_flight_tools, use_short_idle, idle_long, idle_after_thinking
        )
        wait = min(5.0, idle_limit, deadline - now)
        ready, _, _ = select.select([fd], [], [], wait)
        if ready:
            chunk = os.read(fd, 65536)
            if chunk:
                line_buf, kinds = feed_lines(line_buf, chunk)
                in_flight_tools, use_short_idle, last_short_idle_source = (
                    apply_stream_kinds(
                        kinds,
                        in_flight_tools,
                        use_short_idle,
                        last_short_idle_source,
                    )
                )

                sys.stdout.buffer.write(chunk)
                sys.stdout.buffer.flush()
                last_output = time.monotonic()
                saw_output = True
            elif proc.poll() is not None:
                break
            else:
                # Hung CLI: fd readable but no bytes — sleep so we do not spin.
                time.sleep(min(1.0, max(0.0, idle_limit - (time.monotonic() - last_output))))

        idle_limit = idle_limit_secs(
            in_flight_tools, use_short_idle, idle_long, idle_after_thinking
        )
        if (
            saw_output
            and proc.poll() is None
            and (time.monotonic() - last_output) >= idle_limit
        ):
            idle_kind = "short" if in_flight_tools == 0 and use_short_idle else "long"
            label = f"{idle_kind} idle"
            if in_flight_tools > 0:
                label = f"{label} ({in_flight_tools} tool(s) in flight)"
            print(
                f"gch-agent-idle-wrap: no output for {idle_limit}s ({label}); stopping agent",
                file=sys.stderr,
            )
            meta: dict[str, object] = {
                "reason": "idle_timeout",
                "idle_kind": idle_kind,
                "idle_secs": idle_limit,
                "in_flight_tools": in_flight_tools,
            }
            if idle_kind == "short" and last_short_idle_source:
                meta["last_stream_event"] = last_short_idle_source
            write_complete_meta(meta)
            proc.send_signal(signal.SIGTERM)
            try:
                proc.wait(timeout=30)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=30)
            return 0

    return proc.wait() or 0


if __name__ == "__main__":
    sys.exit(main())
