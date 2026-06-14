#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for gch-agent-idle-wrap idle logic."""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_SCRIPT = Path(__file__).resolve().parent / "gch-agent-idle-wrap.py"
_spec = importlib.util.spec_from_file_location("gch_agent_idle_wrap", _SCRIPT)
assert _spec and _spec.loader
_mod = importlib.util.module_from_spec(_spec)
sys.modules["gch_agent_idle_wrap"] = _mod
_spec.loader.exec_module(_mod)

apply_stream_kinds = _mod.apply_stream_kinds
idle_limit_secs = _mod.idle_limit_secs


def test_parallel_tool_keeps_long_idle_after_fast_tool_completes() -> None:
    """forge running + grep completes must not arm short idle (issue #312 scenario)."""
    in_flight = 0
    short = False
    source = None

    in_flight, short, source = apply_stream_kinds(
        ["short_idle_thinking"], in_flight, short, source
    )
    assert short is True
    assert idle_limit_secs(in_flight, short, 1200, 300) == 300

    in_flight, short, source = apply_stream_kinds(
        ["tool_call_started"], in_flight, short, source
    )
    assert in_flight == 1
    assert idle_limit_secs(in_flight, short, 1200, 300) == 1200

    in_flight, short, source = apply_stream_kinds(
        ["tool_call_started", "tool_call_completed"], in_flight, short, source
    )
    assert in_flight == 1
    assert idle_limit_secs(in_flight, short, 1200, 300) == 1200

    in_flight, short, source = apply_stream_kinds(
        ["tool_call_completed"], in_flight, short, source
    )
    assert in_flight == 0
    assert short is True
    assert idle_limit_secs(in_flight, short, 1200, 300) == 300


if __name__ == "__main__":
    test_parallel_tool_keeps_long_idle_after_fast_tool_completes()
    print("ok")
