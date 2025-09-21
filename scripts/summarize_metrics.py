#!/usr/bin/env python3
import argparse
import csv
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
    if not paths:
        paths = ["-"]
    bundles = []
    for path in paths:
        if path == "-" and len(paths) > 1:
            raise ValueError("stdin ('-') cannot be combined with other paths")
        handle = sys.stdin if path == "-" else open(path, "r", encoding="utf-8")
        metrics = []
        with handle:
            for line in handle:
                data = parse_line(line)
                if data is not None and "label" in data:
                    metrics.append(data)
        bundles.append((path, metrics))
    return bundles


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


def write_csv(rows, dest):
    fieldnames = [
        "file",
        "label",
        "count",
        "base_events",
        "predicted_events",
        "scenario_created",
        "scenario_retired",
        "scenario_alerts",
        "scenario_active_peak",
        "avg_elapsed_ms",
        "avg_base_throughput_per_sec",
    ]
    writer = csv.DictWriter(dest, fieldnames=fieldnames)
    writer.writeheader()
    for row in rows:
        writer.writerow(row)


def main():
    parser = argparse.ArgumentParser(description="Summarize Timely Worlds metrics JSON lines.")
    parser.add_argument("paths", nargs="*", help="Files to parse (defaults to stdin if none).")
    parser.add_argument("--pretty", action="store_true", help="Pretty-print JSON output (JSON mode only).")
    parser.add_argument(
        "--per-file",
        action="store_true",
        help="Summarize metrics separately for each input file.",
    )
    parser.add_argument(
        "--csv",
        action="store_true",
        help="Output CSV instead of JSON (implies --per-file).",
    )
    args = parser.parse_args()

    if args.csv:
        args.per_file = True

    bundles = load_metrics(args.paths)

    if args.per_file:
        rows = []
        summaries = {}
        for path, metrics in bundles:
            if not metrics:
                continue
            summary = summarize(metrics)
            summaries[path] = summary
            for label, stats in summary.items():
                row = {
                    "file": path,
                    "label": label,
                    "count": stats.get("count", 0),
                    "base_events": stats.get("base_events", 0),
                    "predicted_events": stats.get("predicted_events", 0),
                    "scenario_created": stats.get("scenario_created", 0),
                    "scenario_retired": stats.get("scenario_retired", 0),
                    "scenario_alerts": stats.get("scenario_alerts", 0),
                    "scenario_active_peak": stats.get("scenario_active_peak", 0),
                    "avg_elapsed_ms": stats.get("avg_elapsed_ms"),
                    "avg_base_throughput_per_sec": stats.get("avg_base_throughput_per_sec"),
                }
                rows.append(row)
        if args.csv:
            write_csv(rows, sys.stdout)
        else:
            if args.pretty:
                json.dump(summaries, sys.stdout, indent=2)
            else:
                json.dump(summaries, sys.stdout)
            sys.stdout.write("\n")
    else:
        merged = []
        for _path, metrics in bundles:
            merged.extend(metrics)
        if not merged:
            print("{}", end="\n")
            return
        summary = summarize(merged)
        if args.pretty:
            json.dump(summary, sys.stdout, indent=2)
        else:
            json.dump(summary, sys.stdout)
        sys.stdout.write("\n")
if __name__ == "__main__":
    main()
