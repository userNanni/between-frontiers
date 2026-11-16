#!/usr/bin/env python3
"""Cost of each simulation system, from bf-bench JSON reports.

    compare.py REPORT.json                  table of one run
    compare.py BASE.json NEW.json [--warn P]  table of both, with the change; a GitHub warning
                                            when the total grows more than P % (default 15)

Reads the reports of `bf-bench --game` (top-level `systems`) and of the synthetic world
(`run.systems`). Markdown goes to $GITHUB_STEP_SUMMARY when it is set, otherwise to stdout;
warnings go to stdout as GitHub workflow commands. The exit code is 0 unless a file is missing
or malformed: a slower PR is flagged, not blocked, since new rules may cost more on purpose.
"""

import argparse
import json
import os
import sys


def load(path):
    with open(path, encoding="utf-8") as f:
        report = json.load(f)
    run = report.get("run", report)
    slowest = run.get("slowest_tick_ms", run.get("slowest_tick", {}).get("ms", 0.0))
    # Reports written before the game mode recorded its systems have none: totals only.
    systems = run.get("systems", [])
    return {
        "systems": {s["name"]: s for s in systems},
        "order": [s["name"] for s in systems],
        "total": run["total_ms"],
        "slowest": slowest,
        "hash": run.get("final_hash", report.get("final_hash")),
        "years": report.get("years"),
    }


def change(base, new):
    if base <= 0:
        return "—"
    return f"{(new - base) / base * 100:+.1f} %"


def one(r):
    rows = ["| system | runs | total ms | mean µs | slowest ms |", "|---|---:|---:|---:|---:|"]
    for name in r["order"]:
        s = r["systems"][name]
        rows.append(
            f"| `{name}` | {s['runs']} | {s['total_ms']:.0f} | {s['mean_us']:.0f} "
            f"| {s['slowest_ms']:.1f} |"
        )
    rows.append(f"| **simulation** | | **{r['total']:.0f}** | | {r['slowest']:.1f} |")
    return rows


def both(base, new):
    rows = [
        "| system | base ms | this ms | change |",
        "|---|---:|---:|---:|",
    ]
    for name in dict.fromkeys(base["order"] + new["order"]):
        b = base["systems"].get(name, {}).get("total_ms", 0.0)
        n = new["systems"].get(name, {}).get("total_ms", 0.0)
        rows.append(f"| `{name}` | {b:.0f} | {n:.0f} | {change(b, n)} |")
    rows.append(
        f"| **simulation** | **{base['total']:.0f}** | **{new['total']:.0f}** "
        f"| **{change(base['total'], new['total'])}** |"
    )
    rows.append(
        f"| slowest tick | {base['slowest']:.1f} | {new['slowest']:.1f} "
        f"| {change(base['slowest'], new['slowest'])} |"
    )
    same = base["hash"] == new["hash"]
    rows.append("")
    rows.append(
        "Final hash: " + ("same as the base." if same else "**differs** from the base "
                          "(expected only when the PR changes the simulation or its data).")
    )
    return rows


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("reports", nargs="+", metavar="REPORT.json")
    p.add_argument("--warn", type=float, default=15.0, metavar="PERCENT")
    p.add_argument("--title", default="Cost per system")
    a = p.parse_args()
    if len(a.reports) > 2:
        p.error("one or two reports")
    runs = [load(r) for r in a.reports]
    lines = [f"### {a.title}", ""]
    lines += one(runs[0]) if len(runs) == 1 else both(*runs)
    markdown = "\n".join(lines) + "\n\n"
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as f:
            f.write(markdown)
    else:
        sys.stdout.write(markdown)
    if len(runs) == 2:
        base, new = runs
        grew = (new["total"] - base["total"]) / base["total"] * 100 if base["total"] > 0 else 0.0
        if grew > a.warn:
            print(
                f"::warning title=Simulation cost::{grew:+.1f} % against the base "
                f"({base['total']:.0f} → {new['total']:.0f} ms); see the job summary"
            )
            for name in new["order"]:
                b = base["systems"].get(name, {}).get("total_ms", 0.0)
                n = new["systems"][name]["total_ms"] if name in new["systems"] else 0.0
                if b > 0 and (n - b) / b * 100 > a.warn and n - b > 0.02 * base["total"]:
                    print(f"::warning title=System cost::{name}: {change(b, n)} ({b:.0f} → {n:.0f} ms)")


if __name__ == "__main__":
    main()
