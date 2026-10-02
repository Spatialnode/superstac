"""Render the recorded local benchmark. Requires matplotlib; no network access."""
import json
from pathlib import Path
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

root = Path(__file__).resolve().parents[1]
folder = root / "docs/public/benchmarks"
data = json.loads((folder / "geoparquet-v0.3.json").read_text())
plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 11, "svg.fonttype": "none"})
fig, ax = plt.subplots(figsize=(9, 3.8), layout="constrained", facecolor="#f8fafc")
ax.set_facecolor("#f8fafc")
values = [data["baseline_median_ms"], data["pruned_median_ms"]]
ax.barh(["Without row-group pruning", "With row-group pruning"], values,
        color=["#94a3b8", "#0f766e"], height=0.48)
ax.invert_yaxis()
ax.set_xlim(0, max(values) * 1.26)
for y, value in enumerate(values):
    ax.text(value + max(values) * 0.025, y, f"{value:.1f} ms", va="center", weight="bold", color="#0f172a")
ax.set_xlabel("Median search latency · 5 runs · lower is better")
ax.set_title("Selective local search over 100,000 metadata records", loc="left", pad=16, weight="bold")
ax.spines[["top", "right", "left"]].set_visible(False)
ax.tick_params(axis="y", length=0)
ax.grid(axis="x", alpha=0.15)
ax.set_axisbelow(True)
fig.supxlabel("Synthetic time-ordered points · Apple M3 Pro · Rust release build\nSame results verified; this is not a provider API comparison.", fontsize=9, color="#475569")
fig.savefig(folder / "geoparquet-v0.3.svg")
fig.savefig(folder / "geoparquet-v0.3.png", dpi=180)
print("Wrote benchmark SVG and PNG")
