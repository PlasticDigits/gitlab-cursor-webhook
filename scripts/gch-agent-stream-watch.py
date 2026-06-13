#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Pretty-print Cursor agent --output-format stream-json to the terminal."""
from __future__ import annotations

import json
import sys


def _emit_text(value: object) -> None:
    if isinstance(value, str) and value.strip():
        sys.stdout.write(value if value.endswith("\n") else value + "\n")
        sys.stdout.flush()


def _walk(obj: object, *, depth: int = 0) -> None:
    if depth > 4:
        return
    if isinstance(obj, str):
        _emit_text(obj)
        return
    if isinstance(obj, list):
        for item in obj:
            _walk(item, depth=depth + 1)
        return
    if not isinstance(obj, dict):
        return

    t = obj.get("type")
    subtype = obj.get("subtype")

    if t == "tool_call":
        tc = obj.get("tool_call") or {}
        shell = (tc.get("shellToolCall") or {}).get("args") or {}
        desc = shell.get("description") or shell.get("command")
        if desc:
            label = desc if len(str(desc)) <= 120 else str(desc)[:117] + "..."
            if subtype == "started":
                print(f"[tool] {label}", flush=True)
            elif subtype == "completed":
                print(f"[tool done] {label}", flush=True)
        elif subtype == "started":
            print("[tool] (shell)", flush=True)

    if t == "thinking" and subtype == "completed":
        print("--- thinking done ---", flush=True)
    elif t in ("tool_call", "tool", "function_call", "function"):
        # Legacy/alternate shapes (skip if shellToolCall already printed above)
        if t == "tool_call" and (obj.get("tool_call") or {}).get("shellToolCall"):
            pass
        else:
            name = (
                obj.get("name")
                or obj.get("tool")
                or obj.get("function")
                or obj.get("toolName")
                or (obj.get("call") or {}).get("name")
                or "?"
            )
            print(f"[tool] {name}", flush=True)
    elif t == "tool_result":
        err = obj.get("error") or obj.get("isError")
        if err:
            print(f"[tool-result] ERROR: {err}", flush=True)

    for key in ("text", "delta", "content", "message", "output", "result", "arguments"):
        if key in obj:
            _walk(obj[key], depth=depth + 1)


def main() -> int:
    for raw in sys.stdin:
        line = raw.strip()
        if not line:
            continue
        try:
            _walk(json.loads(line))
        except json.JSONDecodeError:
            print(line, flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
