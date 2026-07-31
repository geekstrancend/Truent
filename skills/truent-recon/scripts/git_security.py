#!/usr/bin/env python3
"""git_security.py — mine a repo's git history for security signal.

Where past bugs were fixed, where code churns, what changed late and went
un-reviewed, which vendored deps have drifted from upstream, and where the
tech-debt markers cluster — history predicts where the next bug lives. This
turns that history into a ranked, machine-readable pre-audit map so an
auditor (human or the truent-audit skill) spends attention where it pays.

Pure standard library. Shells out to `git`. Degrades gracefully: a shallow
clone, a non-repo, or an empty history yields a valid report with empty
sections and a `notes` explanation rather than an error.

Usage:
  git_security.py [--repo .] [--src-dir auto] [--since 400] [--json out.json]
"""
import argparse
import json
import os
import re
import subprocess
import sys
from collections import Counter, defaultdict

FIX_RE = re.compile(r"\b(fix|bug|patch|vuln|security|exploit|audit|hack|revert|hotfix|CVE)\b", re.I)
SRC_EXT = (".sol", ".rs", ".move", ".vy", ".cairo")
DEBT_RE = re.compile(r"\b(TODO|FIXME|HACK|XXX|BUG|WORKAROUND|UNSAFE|DO NOT|@audit)\b")
SECURITY_HINT_RE = re.compile(
    r"(withdraw|transfer|mint|burn|deposit|borrow|liquidat|swap|price|oracle|"
    r"admin|owner|auth|access|role|upgrade|delegate|call|sign|verify|bridge|"
    r"collateral|reward|fee|vault|stake)",
    re.I,
)


def git(repo, *args):
    try:
        out = subprocess.run(
            ["git", "-C", repo, *args],
            capture_output=True, text=True, timeout=60,
        )
        return out.stdout if out.returncode == 0 else ""
    except Exception:
        return ""


def is_src(path):
    return path.endswith(SRC_EXT)


def in_src_dir(path, src_dir):
    return src_dir in ("", ".", "auto") or path.startswith(src_dir.rstrip("/") + "/") or path.startswith(src_dir)


def detect_src_dir(repo):
    for name in ("src", "contracts", "programs", "sources"):
        if os.path.isdir(os.path.join(repo, name)):
            return name
    return "auto"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    ap.add_argument("--src-dir", default="auto")
    ap.add_argument("--since", type=int, default=400, help="how many recent commits to weigh")
    ap.add_argument("--json", default="-")
    a = ap.parse_args()

    repo = a.repo
    src_dir = detect_src_dir(repo) if a.src_dir == "auto" else a.src_dir
    notes = []

    if not git(repo, "rev-parse", "--is-inside-work-tree").strip():
        report = {"_error": "not a git repository", "repo": repo}
        _emit(report, a.json)
        return

    # ── repo shape ──
    commit_count = len(git(repo, "log", "--oneline", "-n", "5000").splitlines())
    authors = git(repo, "log", "--format=%ae", "-n", "5000").split()
    shape = {
        "commits_sampled": commit_count,
        "contributors": len(set(authors)),
        "top_author_share": round((Counter(authors).most_common(1)[0][1] / len(authors)) if authors else 0, 2),
        "src_dir": src_dir,
    }
    if git(repo, "rev-parse", "--is-shallow-repository").strip() == "true":
        notes.append("shallow clone — history-based signal is partial; fetch full history for best results")

    # ── churn + fix-density per source file ──
    # `git log --name-only` with subjects, walk once.
    log = git(repo, "log", "-n", str(a.since), "--name-only",
              "--pretty=format:__COMMIT__%H%x1f%s")
    churn = Counter()
    fix_touches = Counter()
    last_touch_order = {}
    order = 0
    cur_is_fix = False
    for line in log.splitlines():
        if line.startswith("__COMMIT__"):
            order += 1
            subj = line.split("\x1f", 1)[1] if "\x1f" in line else ""
            cur_is_fix = bool(FIX_RE.search(subj))
            continue
        path = line.strip()
        if not path or not is_src(path) or not in_src_dir(path, src_dir):
            continue
        churn[path] += 1
        if cur_is_fix:
            fix_touches[path] += 1
        last_touch_order.setdefault(path, order)  # smaller order = more recent

    # fix candidates = files most often touched by fix/security commits
    fix_candidates = [
        {"file": f, "fix_commits": n, "total_changes": churn[f]}
        for f, n in fix_touches.most_common(15)
    ]

    # churn hotspots = most-changed source files (churn correlates with defects)
    hotspots = [
        {"file": f, "changes": n, "fix_ratio": round(fix_touches[f] / n, 2)}
        for f, n in churn.most_common(15)
    ]

    # late changes = security-relevant source touched in the most-recent commits
    late = sorted(
        [f for f, o in last_touch_order.items() if o <= 25],
        key=lambda f: last_touch_order[f],
    )
    late_changes = [
        {"file": f, "recency_rank": last_touch_order[f]}
        for f in late
        if _looks_security_relevant(os.path.join(repo, f))
    ][:15]

    # ── forked deps: tracked, modified files under vendor/lib dirs ──
    forked = []
    for dep_dir in ("lib", "vendor", "node_modules", "deps"):
        if not os.path.isdir(os.path.join(repo, dep_dir)):
            continue
        # files under dep dir that have local commits (i.e. we forked them)
        touched = git(repo, "log", "-n", "500", "--name-only", "--pretty=format:",
                      "--", dep_dir).split()
        for f in sorted(set(x for x in touched if is_src(x)))[:10]:
            forked.append({"file": f, "dep_dir": dep_dir})

    # ── tech-debt markers in source ──
    debt = _scan_debt(repo, src_dir)

    report = {
        "repo_shape": shape,
        "fix_candidates": fix_candidates,
        "churn_hotspots": hotspots,
        "late_changes": late_changes,
        "forked_deps": forked[:15],
        "tech_debt": debt[:20],
        "notes": notes,
        "how_to_use": (
            "Rank audit attention by: fix_candidates (historically bug-prone) and "
            "churn_hotspots (high change = high defect density) first, then "
            "late_changes (recently touched, least reviewed), then forked_deps "
            "(drift from upstream) and tech_debt (self-flagged risk). Feed these "
            "files as the priority set to `truent scan` / the truent-audit skill."
        ),
    }
    _emit(report, a.json)


def _looks_security_relevant(abspath):
    try:
        with open(abspath, "r", errors="ignore") as fh:
            head = fh.read(20000)
        return bool(SECURITY_HINT_RE.search(head))
    except Exception:
        return True  # can't read → don't filter it out


def _scan_debt(repo, src_dir):
    out = []
    base = repo if src_dir in ("auto", "", ".") else os.path.join(repo, src_dir)
    for root, dirs, files in os.walk(base):
        dirs[:] = [d for d in dirs if d not in (".git", "node_modules", "target", "lib", "vendor", "out")]
        for fn in files:
            if not fn.endswith(SRC_EXT):
                continue
            p = os.path.join(root, fn)
            try:
                with open(p, "r", errors="ignore") as fh:
                    for i, line in enumerate(fh, 1):
                        if DEBT_RE.search(line):
                            out.append({
                                "file": os.path.relpath(p, repo),
                                "line": i,
                                "marker": line.strip()[:120],
                            })
            except Exception:
                continue
    return out


def _emit(report, dst):
    text = json.dumps(report, indent=2)
    if dst in ("-", None):
        print(text)
    else:
        with open(dst, "w") as fh:
            fh.write(text)


if __name__ == "__main__":
    main()
