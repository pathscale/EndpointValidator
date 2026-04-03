#!/usr/bin/env python3
"""
Compare WebSocket load test results across different providers/machine types.

Usage:
    # Create virtual environment (one-time setup)
    python -m venv venv
    source venv/bin/activate
    pip install -r requirements.txt

    # Run the comparison script
    python compare_results.py [results_base_dir]

Expects folders named like: <provider>-<machine_type>/ or <provider>/
Containing JSON files from ws_simple benchmark.
"""

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np


COLORS = plt.cm.tab10.colors
MARKERS = ["o", "s", "^", "D", "v", "<", ">", "p", "h", "*"]
LINESTYLES = ["-", "--", "-.", ":"]

LATENCY_PERCENTILES = ["mean", "p50", "p95", "p99"]
LATENCY_LABELS = {"mean": "Mean", "p50": "P50", "p95": "P95", "p99": "P99"}


# ─────────────────────────────────────────────────────────────────────────────
# Data loading & extraction
# ─────────────────────────────────────────────────────────────────────────────

def parse_folder_name(folder_name: str) -> tuple[str, str]:
    """Parse '<provider>-<machine_type>' or '<provider>' into (provider, machine_type)."""
    match = re.match(r"^([a-z]+)-(.+)$", folder_name)
    if match:
        return match.group(1), match.group(2)
    return folder_name, "default"


def load_results_files(results_dir: Path) -> list[dict]:
    """Load all ws_simple_*.json files from a results directory."""
    results = []
    for f in sorted(results_dir.glob("ws_simple_*.json")):
        try:
            with open(f) as fp:
                data = json.load(fp)
                data["_source_file"] = f.name
                results.append(data)
        except (json.JSONDecodeError, IOError) as e:
            print(f"Warning: Could not load {f}: {e}")
    return results


def extract_config(data: dict) -> tuple[int, int]:
    """Extract (num_parallel, num_requests) from results JSON."""
    scenario = data.get("scenario", {})
    return (scenario.get("num_parallel", 0), scenario.get("num_requests", 0))


def extract_metrics(data: dict) -> dict:
    """Extract key metrics from results JSON."""
    results = data.get("results", {})
    latency = results.get("latency", {}) or {}
    return {
        "rps": results.get("rps", 0),
        "sent": results.get("sent", 0),
        "ok": results.get("ok", 0),
        "errors": results.get("errors", 0),
        "error_pct": results.get("error_pct", 0),
        "elapsed_s": results.get("elapsed_s", 0),
        "latency_mean_ms": latency.get("mean_ms", 0),
        "latency_p50_ms": latency.get("p50_ms", 0),
        "latency_p95_ms": latency.get("p95_ms", 0),
        "latency_p99_ms": latency.get("p99_ms", 0),
        "latency_min_ms": latency.get("min_ms", 0),
        "latency_max_ms": latency.get("max_ms", 0),
    }


def collect_all_results(base_dir: Path) -> dict[str, dict[str, list[dict]]]:
    """Collect results from all provider-machine folders."""
    all_results = defaultdict(lambda: defaultdict(list))

    for folder in sorted(base_dir.iterdir()):
        if not folder.is_dir() or folder.name == "graphs":
            continue
        provider, machine_type = parse_folder_name(folder.name)
        results = load_results_files(folder)
        if not results:
            continue
        for r in results:
            r["_provider"] = provider
            r["_machine_type"] = machine_type
            r["_folder"] = folder.name
        all_results[provider][machine_type].extend(results)

    return all_results


def group_by_config(results: list[dict]) -> dict[tuple[int, int], list[dict]]:
    """Group results by (num_parallel, num_requests)."""
    grouped = defaultdict(list)
    for r in results:
        grouped[extract_config(r)].append(r)
    return grouped


def average_metrics(metrics_list: list[dict]) -> dict:
    """Average multiple metric dicts."""
    if not metrics_list:
        return {}
    keys = metrics_list[0].keys()
    return {
        k: np.mean([m[k] for m in metrics_list])
        for k in keys
        if isinstance(metrics_list[0][k], (int, float))
    }


def filter_by_requests(grouped, num_requests_filter):
    if num_requests_filter is None:
        return grouped
    return {k: v for k, v in grouped.items() if k[1] == num_requests_filter}


def iter_provider_machines(all_results):
    """Yield (provider, machine_type, results, color, marker) with stable ordering."""
    idx = 0
    for provider in sorted(all_results):
        for machine_type in sorted(all_results[provider]):
            yield (
                provider,
                machine_type,
                all_results[provider][machine_type],
                COLORS[idx % len(COLORS)],
                MARKERS[idx % len(MARKERS)],
            )
            idx += 1


def build_parallelism_series(results, metric_key, num_requests_filter=None):
    """Return (x_vals, y_vals) averaged by parallelism for a single metric."""
    grouped = filter_by_requests(group_by_config(results), num_requests_filter)
    if not grouped:
        return [], []
    data = defaultdict(list)
    for (parallel, _), runs in grouped.items():
        avg = average_metrics([extract_metrics(r) for r in runs])
        data[parallel].append(avg.get(metric_key, 0))
    x_vals = sorted(data)
    y_vals = [np.mean(data[p]) for p in x_vals]
    return x_vals, y_vals


def save(fig, out: Path):
    fig.savefig(out, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved: {out.name}")


# ─────────────────────────────────────────────────────────────────────────────
# Plots
# ─────────────────────────────────────────────────────────────────────────────

def plot_rps_vs_parallelism(all_results, output_dir, num_requests_filter=None):
    """Throughput (RPS) vs parallelism for all providers."""
    fig, ax = plt.subplots(figsize=(12, 7))

    for provider, machine_type, results, color, marker in iter_provider_machines(all_results):
        x, y = build_parallelism_series(results, "rps", num_requests_filter)
        if x:
            ax.plot(x, y, marker=marker, linewidth=2, markersize=8,
                    label=f"{provider} ({machine_type})", color=color)

    ax.set_xlabel("Num Parallel (Workers)", fontsize=12)
    ax.set_ylabel("Requests Per Second (RPS)", fontsize=12)
    ax.set_title("Throughput vs Parallelism", fontsize=14)
    ax.legend(loc="best", fontsize=10)
    ax.grid(True, alpha=0.3)
    fig.tight_layout()

    suffix = f"_requests{num_requests_filter}" if num_requests_filter else ""
    save(fig, output_dir / f"rps_vs_parallelism{suffix}.png")


def plot_rps_efficiency(all_results, output_dir, num_requests_filter=None):
    """RPS per worker vs parallelism — shows how efficiently the server scales."""
    fig, ax = plt.subplots(figsize=(12, 7))

    for provider, machine_type, results, color, marker in iter_provider_machines(all_results):
        grouped = filter_by_requests(group_by_config(results), num_requests_filter)
        if not grouped:
            continue
        data = defaultdict(list)
        for (parallel, _), runs in grouped.items():
            avg = average_metrics([extract_metrics(r) for r in runs])
            if parallel > 0:
                data[parallel].append(avg.get("rps", 0) / parallel)
        x_vals = sorted(data)
        y_vals = [np.mean(data[p]) for p in x_vals]
        if x_vals:
            ax.plot(x_vals, y_vals, marker=marker, linewidth=2, markersize=8,
                    label=f"{provider} ({machine_type})", color=color)

    ax.set_xlabel("Num Parallel (Workers)", fontsize=12)
    ax.set_ylabel("RPS per Worker", fontsize=12)
    ax.set_title("Throughput Efficiency vs Parallelism\n(higher = better scaling)", fontsize=14)
    ax.legend(loc="best", fontsize=10)
    ax.grid(True, alpha=0.3)
    fig.tight_layout()

    suffix = f"_requests{num_requests_filter}" if num_requests_filter else ""
    save(fig, output_dir / f"rps_efficiency{suffix}.png")


def plot_latency_percentiles(all_results, output_dir, num_requests_filter=None):
    """
    All latency percentiles (mean, p50, p95, p99) on one chart.
    Each provider/machine gets a color; percentiles are distinguished by line style.
    """
    fig, ax = plt.subplots(figsize=(13, 8))
    percentile_styles = {
        "mean": (":", 1.5),
        "p50":  ("-", 1.5),
        "p95":  ("--", 2.0),
        "p99":  ("-.", 2.5),
    }

    for provider, machine_type, results, color, marker in iter_provider_machines(all_results):
        for pct, (ls, lw) in percentile_styles.items():
            x, y = build_parallelism_series(results, f"latency_{pct}_ms", num_requests_filter)
            if x:
                ax.plot(x, y, marker=marker, linewidth=lw, markersize=6,
                        linestyle=ls, color=color,
                        label=f"{provider} ({machine_type}) — {LATENCY_LABELS[pct]}")

    ax.set_xlabel("Num Parallel (Workers)", fontsize=12)
    ax.set_ylabel("Latency (ms)", fontsize=12)
    ax.set_title("Latency Percentiles vs Parallelism", fontsize=14)
    ax.legend(loc="upper left", fontsize=8, ncol=2)
    ax.grid(True, alpha=0.3)
    fig.tight_layout()

    suffix = f"_requests{num_requests_filter}" if num_requests_filter else ""
    save(fig, output_dir / f"latency_percentiles{suffix}.png")


def plot_latency_fan(all_results, output_dir, num_requests_filter=None):
    """
    Per provider/machine: P50 as solid line with a shaded band from P50 to P99,
    showing the latency spread. One subplot per provider/machine.
    """
    series_list = [
        (prov, mtype, res, col, mrk)
        for prov, mtype, res, col, mrk in iter_provider_machines(all_results)
        if filter_by_requests(group_by_config(res), num_requests_filter)
    ]
    if not series_list:
        return

    n = len(series_list)
    ncols = min(n, 2)
    nrows = (n + ncols - 1) // ncols
    fig, axes = plt.subplots(nrows, ncols, figsize=(7 * ncols, 5 * nrows), squeeze=False)

    for idx, (provider, machine_type, results, color, _) in enumerate(series_list):
        ax = axes[idx // ncols][idx % ncols]
        x_p50, y_p50 = build_parallelism_series(results, "latency_p50_ms", num_requests_filter)
        x_p95, y_p95 = build_parallelism_series(results, "latency_p95_ms", num_requests_filter)
        x_p99, y_p99 = build_parallelism_series(results, "latency_p99_ms", num_requests_filter)
        x_mean, y_mean = build_parallelism_series(results, "latency_mean_ms", num_requests_filter)

        if x_p50:
            ax.fill_between(x_p99, y_p50, y_p99, alpha=0.15, color=color, label="P50–P99 band")
            ax.fill_between(x_p95, y_p50, y_p95, alpha=0.25, color=color, label="P50–P95 band")
            ax.plot(x_p99, y_p99, "--", linewidth=1.5, color=color, alpha=0.7, label="P99")
            ax.plot(x_p95, y_p95, "-.", linewidth=1.5, color=color, alpha=0.8, label="P95")
            ax.plot(x_p50, y_p50, "-", linewidth=2, color=color, label="P50")
            ax.plot(x_mean, y_mean, ":", linewidth=1.5, color=color, alpha=0.7, label="Mean")

        ax.set_title(f"{provider} ({machine_type})", fontsize=12)
        ax.set_xlabel("Num Parallel (Workers)", fontsize=10)
        ax.set_ylabel("Latency (ms)", fontsize=10)
        ax.legend(fontsize=8)
        ax.grid(True, alpha=0.3)

    # Hide any unused subplots
    for idx in range(n, nrows * ncols):
        axes[idx // ncols][idx % ncols].set_visible(False)

    fig.suptitle("Latency Distribution Fan (P50 / P95 / P99)", fontsize=14, y=1.02)
    fig.tight_layout()

    suffix = f"_requests{num_requests_filter}" if num_requests_filter else ""
    save(fig, output_dir / f"latency_fan{suffix}.png")


def plot_latency_distribution_bars(all_results, output_dir, num_requests_filter=None):
    """
    For each parallelism level, show grouped bars of p50/p95/p99 per provider/machine.
    Helps compare latency distributions at identical load.
    """
    # Gather all parallelism levels that appear
    all_parallel = set()
    for _, _, results, _, _ in iter_provider_machines(all_results):
        grouped = filter_by_requests(group_by_config(results), num_requests_filter)
        for (parallel, _) in grouped:
            all_parallel.add(parallel)

    if not all_parallel:
        return

    all_parallel = sorted(all_parallel)
    providers = list(iter_provider_machines(all_results))
    n_providers = len(providers)
    percentiles = ["p50", "p95", "p99"]
    n_pct = len(percentiles)

    for parallel_val in all_parallel:
        fig, ax = plt.subplots(figsize=(max(10, n_providers * 3), 7))

        bar_width = 0.8 / n_providers
        x_base = np.arange(n_pct)

        for pidx, (provider, machine_type, results, color, _) in enumerate(providers):
            grouped = filter_by_requests(group_by_config(results), num_requests_filter)
            # Get all runs for this parallelism
            runs = [r for (par, _), rs in grouped.items() if par == parallel_val for r in rs]
            if not runs:
                continue

            avg = average_metrics([extract_metrics(r) for r in runs])
            heights = [avg.get(f"latency_{p}_ms", 0) for p in percentiles]
            offsets = x_base + pidx * bar_width - (n_providers - 1) * bar_width / 2
            bars = ax.bar(offsets, heights, width=bar_width * 0.9, color=color,
                          label=f"{provider} ({machine_type})", alpha=0.85)
            for bar, h in zip(bars, heights):
                ax.text(bar.get_x() + bar.get_width() / 2, h + 0.5,
                        f"{h:.0f}", ha="center", va="bottom", fontsize=7)

        ax.set_xticks(x_base)
        ax.set_xticklabels([p.upper() for p in percentiles], fontsize=12)
        ax.set_ylabel("Latency (ms)", fontsize=12)
        ax.set_title(f"Latency Distribution — {parallel_val} Workers", fontsize=14)
        ax.legend(loc="upper left", fontsize=9)
        ax.grid(True, alpha=0.3, axis="y")
        fig.tight_layout()

        suffix = f"_p{parallel_val}" if num_requests_filter is None else f"_p{parallel_val}_r{num_requests_filter}"
        save(fig, output_dir / f"latency_dist_bars{suffix}.png")


def plot_error_rate_vs_parallelism(all_results, output_dir, num_requests_filter=None):
    """Error rate (%) vs parallelism."""
    fig, ax = plt.subplots(figsize=(12, 7))

    for provider, machine_type, results, color, marker in iter_provider_machines(all_results):
        x, y = build_parallelism_series(results, "error_pct", num_requests_filter)
        if x:
            ax.plot(x, y, marker=marker, linewidth=2, markersize=8,
                    label=f"{provider} ({machine_type})", color=color)

    ax.set_xlabel("Num Parallel (Workers)", fontsize=12)
    ax.set_ylabel("Error Rate (%)", fontsize=12)
    ax.set_title("Error Rate vs Parallelism", fontsize=14)
    ax.yaxis.set_major_formatter(ticker.PercentFormatter())
    ax.legend(loc="best", fontsize=10)
    ax.grid(True, alpha=0.3)
    fig.tight_layout()

    suffix = f"_requests{num_requests_filter}" if num_requests_filter else ""
    save(fig, output_dir / f"error_rate_vs_parallelism{suffix}.png")


def plot_rps_latency_scatter(all_results, output_dir, num_requests_filter=None):
    """
    Scatter: RPS vs P99 latency. Each point is one (provider, machine, parallelism) combo.
    Annotated with parallelism level. Points connected per provider to show the scaling curve.
    """
    fig, ax = plt.subplots(figsize=(12, 7))

    for provider, machine_type, results, color, marker in iter_provider_machines(all_results):
        grouped = filter_by_requests(group_by_config(results), num_requests_filter)
        if not grouped:
            continue

        rps_vals, lat_vals, labels = [], [], []
        for (parallel, _), runs in sorted(grouped.items()):
            avg = average_metrics([extract_metrics(r) for r in runs])
            rps_vals.append(avg.get("rps", 0))
            lat_vals.append(avg.get("latency_p99_ms", 0))
            labels.append(parallel)

        if not rps_vals:
            continue

        # Connect points with a faint line to show the scaling trajectory
        ax.plot(lat_vals, rps_vals, "-", linewidth=1, color=color, alpha=0.3)
        ax.scatter(lat_vals, rps_vals, marker=marker, s=100, alpha=0.85,
                   label=f"{provider} ({machine_type})", color=color, zorder=3)

        for lat, rps, label in zip(lat_vals, rps_vals, labels):
            ax.annotate(str(label), (lat, rps), fontsize=7, alpha=0.8,
                        xytext=(4, 4), textcoords="offset points")

    ax.set_xlabel("Latency P99 (ms)", fontsize=12)
    ax.set_ylabel("Requests Per Second (RPS)", fontsize=12)
    ax.set_title("Throughput vs Latency Trade-off\n(annotations = num parallel workers)", fontsize=14)
    ax.legend(loc="best", fontsize=10)
    ax.grid(True, alpha=0.3)
    fig.tight_layout()

    suffix = f"_requests{num_requests_filter}" if num_requests_filter else ""
    save(fig, output_dir / f"rps_latency_scatter{suffix}.png")


def plot_summary_table(all_results, output_dir):
    """Matplotlib summary table image."""
    headers = ["Provider", "Machine", "Files", "Max RPS", "Best P99 (ms)", "Avg P99 (ms)", "Avg Err %"]
    rows = []

    for provider, machine_type, results, _, _ in iter_provider_machines(all_results):
        all_metrics = [extract_metrics(r) for r in results]
        if not all_metrics:
            continue
        max_rps = max(m.get("rps", 0) for m in all_metrics)
        min_p99 = min((m.get("latency_p99_ms", float("inf")) for m in all_metrics), default=float("inf"))
        avg_p99 = np.mean([m.get("latency_p99_ms", 0) for m in all_metrics])
        avg_err = np.mean([m.get("error_pct", 0) for m in all_metrics])
        rows.append([
            provider, machine_type, len(results),
            f"{max_rps:.1f}",
            f"{min_p99:.0f}" if min_p99 != float("inf") else "N/A",
            f"{avg_p99:.1f}",
            f"{avg_err:.2f}%",
        ])

    if not rows:
        return

    fig, ax = plt.subplots(figsize=(12, len(rows) * 0.6 + 2))
    ax.axis("tight")
    ax.axis("off")
    table = ax.table(cellText=rows, colLabels=headers, cellLoc="center", loc="center",
                     colColours=["#d0d0d0"] * len(headers))
    table.auto_set_font_size(False)
    table.set_fontsize(10)
    table.scale(1.2, 1.6)
    plt.title("Summary by Provider / Machine", fontsize=14, pad=20)
    fig.tight_layout()
    save(fig, output_dir / "summary_table.png")


# ─────────────────────────────────────────────────────────────────────────────
# Console summary
# ─────────────────────────────────────────────────────────────────────────────

def print_console_summary(all_results):
    """Print a text summary table to stdout."""
    col_w = [12, 14, 6, 10, 13, 13, 13, 8]
    headers = ["Provider", "Machine", "Files", "Max RPS", "Best P99 (ms)", "Avg P99 (ms)", "Avg Mean (ms)", "Err %"]
    sep = "─" * (sum(col_w) + len(col_w) * 3 + 1)

    def row_str(cells):
        return "│ " + " │ ".join(str(c).ljust(w) for c, w in zip(cells, col_w)) + " │"

    print(f"\n{sep}")
    print(row_str(headers))
    print(sep)

    for provider, machine_type, results, _, _ in iter_provider_machines(all_results):
        all_metrics = [extract_metrics(r) for r in results]
        if not all_metrics:
            continue
        max_rps = max(m.get("rps", 0) for m in all_metrics)
        min_p99 = min((m.get("latency_p99_ms", float("inf")) for m in all_metrics), default=float("inf"))
        avg_p99 = np.mean([m.get("latency_p99_ms", 0) for m in all_metrics])
        avg_mean = np.mean([m.get("latency_mean_ms", 0) for m in all_metrics])
        avg_err = np.mean([m.get("error_pct", 0) for m in all_metrics])
        print(row_str([
            provider, machine_type, len(results),
            f"{max_rps:.1f}",
            f"{min_p99:.0f}" if min_p99 != float("inf") else "N/A",
            f"{avg_p99:.1f}",
            f"{avg_mean:.1f}",
            f"{avg_err:.2f}%",
        ]))

    print(sep)


# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

def main():
    base_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")

    if not base_dir.exists():
        print(f"Error: Directory '{base_dir}' does not exist")
        sys.exit(1)

    print(f"Scanning for results in: {base_dir.absolute()}")
    all_results = collect_all_results(base_dir)

    if not all_results:
        print("No results found. Expected folders like: <provider>-<machine_type>/")
        sys.exit(1)

    print("\nFound results:")
    for provider, machines in sorted(all_results.items()):
        for machine_type, results in sorted(machines.items()):
            print(f"  {provider} / {machine_type}: {len(results)} result files")

    print_console_summary(all_results)

    output_dir = base_dir / "graphs"
    output_dir.mkdir(exist_ok=True)
    print(f"\nGenerating graphs in: {output_dir}\n")

    # Collect all unique request counts for filtered views
    all_request_counts = {
        extract_config(r)[1]
        for machines in all_results.values()
        for results in machines.values()
        for r in results
    }

    print("1. RPS vs Parallelism...")
    plot_rps_vs_parallelism(all_results, output_dir)
    for req_count in sorted(all_request_counts)[:5]:
        plot_rps_vs_parallelism(all_results, output_dir, num_requests_filter=req_count)

    print("2. RPS Efficiency (RPS per worker)...")
    plot_rps_efficiency(all_results, output_dir)

    print("3. Latency Percentiles (mean/P50/P95/P99 combined)...")
    plot_latency_percentiles(all_results, output_dir)

    print("4. Latency Fan (per provider, shaded P50–P99 band)...")
    plot_latency_fan(all_results, output_dir)

    print("5. Latency Distribution Bars (per parallelism level)...")
    plot_latency_distribution_bars(all_results, output_dir)

    print("6. Error Rate vs Parallelism...")
    plot_error_rate_vs_parallelism(all_results, output_dir)

    print("7. RPS vs Latency Scatter...")
    plot_rps_latency_scatter(all_results, output_dir)

    print("8. Summary Table...")
    plot_summary_table(all_results, output_dir)

    print("\nDone!")


if __name__ == "__main__":
    main()
