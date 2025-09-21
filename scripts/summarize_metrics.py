#!/usr/bin/env python3
import argparse
import json
import sys
from collections import defaultdict
from statistics import mean


def parse_line(line):
    idx = line.find("json=")
    if idx == -1:
        if line.startswith("{") and line.rstrip().endswith("}"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                return None
        return None
    payload = line[idx + 5 :].strip()
    if not payload:
        return None
    try:
        return json.loads(payload)
    except json.JSONDecodeError:
        return None


def load_metrics(paths):
    metrics = []
    if not paths:
        paths = ["-"]
    for path in paths:
        handle = sys.stdin if path == "-" else open(path, "r", encoding="utf-8")
        with handle:
            for line in handle:
                data = parse_line(line)
                if data is not None and "label" in data:
                    metrics.append(data)
    return metrics


def summarize(metrics):
    grouped = defaultdict(list)
    for item in metrics:
        grouped[item["label"]].append(item)
    results = {}
    for label, items in grouped.items():
        totals = defaultdict(float)
        elapsed = []
        for entry in items:
            for key in (
                "base_events",
                "predicted_events",
                "scenario_created",
                "scenario_retired",
                "scenario_alerts",
                "scenario_active_peak",
            ):
                if key in entry:
                    totals[key] += entry[key]
            if "elapsed_ms" in entry and entry["elapsed_ms"] is not None:
                elapsed.append(entry["elapsed_ms"])
        avg_elapsed = mean(elapsed) if elapsed else None
        throughput = None
        if avg_elapsed and totals["base_events"] > 0:
            throughput = (totals["base_events"] / len(items)) / (avg_elapsed / 1000.0)
        results[label] = {
            "count": len(items),
            **{k: totals[k] for k in totals},
        }
        if avg_elapsed is not None:
            results[label]["avg_elapsed_ms"] = avg_elapsed
        if throughput is not None:
            results[label]["avg_base_throughput_per_sec"] = throughput
    return results


def main():
    parser = argparse.ArgumentParser(description="Summarize Timely Worlds metrics JSON lines.")
    parser.add_argument("paths", nargs="*", help="Files to parse (defaults to stdin if none).")
    parser.add_argument("--pretty", action="store_true", help="Pretty-print JSON output.")
    args = parser.parse_args()

    metrics = load_metrics(args.paths)
    if not metrics:
        print("{}", end="\n")
        return
    summary = summarize(metrics)
    if args.pretty:
        json.dump(summary, sys.stdout, indent=2)
    else:
        json.dump(summary, sys.stdout)
    sys.stdout.write("\n")
if __name__ == "__main__":
    main()
