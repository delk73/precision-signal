#!/usr/bin/env python3
"""Validate public documentation links using repository-local invariants."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote


MARKDOWN_LINK_RE = re.compile(r"\[(?P<label>[^\]]+)\]\((?P<target>[^)]+)\)")
SCHEME_RE = re.compile(r"^[a-zA-Z][a-zA-Z0-9+.-]*:")
FENCE_RE = re.compile(r"^(```|~~~)")
PATH_REF_RE = re.compile(
    r"(?P<quoted>`(?P<quoted_path>(?:\.\./|\./|docs/)?[A-Za-z0-9_./-]+\.md|docs/[A-Za-z0-9_./-]+/)`)|"
    r"(?P<plain>(?:\.\./|\./|docs/)?[A-Za-z0-9_./-]+\.md|docs/[A-Za-z0-9_./-]+/)"
)
NAVIGATION_CUE_RE = re.compile(
    r"\b(see|use|lives in|live in|defined in|routed to|routing lives in|"
    r"classified in|questions route to|defer to|called out in|for .* use|"
    r"remains in|contract is|authority is defined in|status is classified in)\b",
    re.IGNORECASE,
)
BULLET_RE = re.compile(r"^(?P<indent>\s*)-\s+")


@dataclass(frozen=True)
class Finding:
    path: Path
    line_no: int
    defect_class: str
    reference_text: str
    reason: str


def repo_facing_label(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


ROUTED_SUPPORT_INDEXES = {
    "docs/audits/AUDIT_INDEX.md",
    "docs/wip/WIP_INDEX.md",
}


def is_routed_support_index(rel: str) -> bool:
    return rel in ROUTED_SUPPORT_INDEXES


def iter_public_docs(root: Path) -> list[Path]:
    files = [root / "README.md"]
    docs_dir = root / "docs"
    for path in sorted(docs_dir.rglob("*.md")):
        rel = path.relative_to(root).as_posix()
        if is_routed_support_index(rel):
            files.append(path)
            continue
        if rel.startswith("docs/wip/"):
            continue
        if rel.startswith("docs/audits/"):
            continue
        if rel.startswith("docs/archive/"):
            continue
        if rel.startswith("docs/internal/"):
            continue
        if rel.startswith("docs/verification/releases/"):
            # Keep release-evidence pages out of general checks except the
            # routed release landing page.
            if rel == "docs/verification/releases/index.md":
                files.append(path)
            continue
        files.append(path)
    return files


def is_external_target(target: str) -> bool:
    return target.startswith("#") or SCHEME_RE.match(target) is not None


def resolve_markdown_target(source: Path, target: str) -> Path:
    target_path = unquote(target.split("#", 1)[0]).strip()
    return (source.parent / target_path).resolve()


def infer_prose_target(source: Path, candidate: str, root: Path) -> Path | None:
    clean = candidate.strip("`")
    choices: list[Path] = []
    if clean.startswith("docs/"):
        choices.append((root / clean).resolve())
    elif clean.startswith("../") or clean.startswith("./"):
        choices.append((source.parent / clean).resolve())
    else:
        choices.append((source.parent / clean).resolve())
        choices.append((root / clean).resolve())

    seen: set[Path] = set()
    for candidate_path in choices:
        if candidate_path in seen:
            continue
        seen.add(candidate_path)
        if candidate_path.exists():
            return candidate_path
    return None


def looks_like_navigation(line: str, match: re.Match[str]) -> bool:
    stripped = line.lstrip()
    if NAVIGATION_CUE_RE.search(line):
        return True
    if stripped.startswith(("- ", "* ")) and ":" in line[match.end() :]:
        return True
    return False


def remove_markdown_links(line: str) -> str:
    return MARKDOWN_LINK_RE.sub(lambda match: " " * (match.end() - match.start()), line)


def iter_non_fenced_lines(path: Path) -> list[tuple[int, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    result: list[tuple[int, str]] = []
    in_fence = False
    for index, line in enumerate(lines, start=1):
        if FENCE_RE.match(line.strip()):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        result.append((index, line))
    return result


INDEX_ROOTS = {
    "README.md",
    "docs/DOCS_INDEX.md",
    "docs/replay/REPLAY_INDEX.md",
}

ROUTING_PAGES = {
    "README.md",
    "docs/DOCS_INDEX.md",
    "docs/replay/REPLAY_INDEX.md",
    "docs/audits/AUDIT_INDEX.md",
    "docs/wip/WIP_INDEX.md",
    "docs/architecture/performance/PERFORMANCE_INDEX.md",
    "docs/demos/demo.md",
}

PARALLEL_ROUTE_ALLOWLIST = {
    "docs/START_HERE.md",
    "docs/VERIFICATION_GUIDE.md",
    "docs/RELEASE_SURFACE.md",
}

PARALLEL_ROUTE_TARGETS = {
    "docs/DOCS_INDEX.md",
    "docs/replay/REPLAY_INDEX.md",
    "docs/audits/AUDIT_INDEX.md",
    "docs/wip/WIP_INDEX.md",
    "docs/architecture/performance/PERFORMANCE_INDEX.md",
    "docs/demos/demo.md",
}


def normalize_markdown_target(source: Path, target: str, root: Path) -> str | None:
    if is_external_target(target):
        return None
    clean = unquote(target.split("#", 1)[0]).strip()
    if not clean or not clean.endswith(".md"):
        return None
    resolved = (source.parent / clean).resolve()
    try:
        return resolved.relative_to(root).as_posix()
    except ValueError:
        return None


def _outbound_targets(path: Path, root: Path) -> set[Path]:
    """Return resolved Markdown-link targets from a single file."""
    targets: set[Path] = set()
    if not path.exists():
        return targets
    for _, line in iter_non_fenced_lines(path):
        for match in MARKDOWN_LINK_RE.finditer(line):
            target = match.group("target").strip()
            if is_external_target(target):
                continue
            resolved = resolve_markdown_target(path, target)
            targets.add(resolved)
    return targets


def collect_reachable(root: Path) -> set[Path]:
    """BFS from index roots to find all transitively linked public docs."""
    public_set = {p.resolve() for p in iter_public_docs(root)}
    visited: set[Path] = set()
    queue: list[Path] = []
    for rel in INDEX_ROOTS:
        p = (root / rel).resolve()
        if p in public_set:
            visited.add(p)
            queue.append(p)
    while queue:
        current = queue.pop()
        for target in _outbound_targets(current, root):
            if target in visited:
                continue
            if target in public_set:
                visited.add(target)
                queue.append(target)
    return visited


def collect_orphan_findings(root: Path) -> list[Finding]:
    """Any new public doc must be linked from exactly one relevant index."""
    # Skip orphan check when no docs-level index root exists (e.g. synthetic test trees).
    if not any((root / rel).exists() for rel in INDEX_ROOTS if rel.startswith("docs/")):
        return []
    reachable = collect_reachable(root)
    findings: list[Finding] = []
    for path in iter_public_docs(root):
        resolved = path.resolve()
        rel = path.relative_to(root).as_posix()
        if rel in INDEX_ROOTS:
            continue
        if resolved not in reachable:
            findings.append(
                Finding(
                    path=path,
                    line_no=0,
                    defect_class="orphaned_doc",
                    reference_text=rel,
                    reason="public doc not reachable from any index",
                )
            )
    return findings


def collect_parallel_reader_path_findings(root: Path) -> list[Finding]:
    route_sources_by_target: dict[str, set[str]] = {}
    for rel_source in sorted(ROUTING_PAGES):
        source = root / rel_source
        if not source.exists():
            continue
        for _, line in iter_non_fenced_lines(source):
            for match in MARKDOWN_LINK_RE.finditer(line):
                target = normalize_markdown_target(source, match.group("target").strip(), root)
                if target is None:
                    continue
                route_sources_by_target.setdefault(target, set()).add(rel_source)

    findings: list[Finding] = []
    for target, sources in sorted(route_sources_by_target.items()):
        if len(sources) < 2:
            continue
        if target in PARALLEL_ROUTE_ALLOWLIST:
            continue
        if target not in PARALLEL_ROUTE_TARGETS:
            continue
        source_list = ", ".join(sorted(sources))
        first_source = sorted(sources)[0]
        findings.append(
            Finding(
                path=root / first_source,
                line_no=0,
                defect_class="parallel_reader_path",
                reference_text=target,
                reason=f"directly linked from multiple routing sources: {source_list}",
            )
        )
    return findings


def collect_findings(root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in iter_public_docs(root):
        lines = iter_non_fenced_lines(path)
        for idx, (line_no, line) in enumerate(lines):
            for match in MARKDOWN_LINK_RE.finditer(line):
                label = match.group("label")
                target = match.group("target").strip()
                if is_external_target(target):
                    continue
                resolved = resolve_markdown_target(path, target)
                if not resolved.exists():
                    findings.append(
                        Finding(
                            path=path,
                            line_no=line_no,
                            defect_class="broken_target",
                            reference_text=match.group(0),
                            reason="target does not resolve from source file",
                        )
                    )
                    continue
                if "../" in label or "./" in label:
                    findings.append(
                        Finding(
                            path=path,
                            line_no=line_no,
                            defect_class="bad_label",
                            reference_text=match.group(0),
                            reason="label exposes traversal instead of repo-facing text",
                        )
                    )

            prose = remove_markdown_links(line)
            for match in PATH_REF_RE.finditer(prose):
                candidate = match.group("quoted") or match.group("plain")
                if candidate is None:
                    continue
                if not looks_like_navigation(line, match):
                    continue
                inferred = infer_prose_target(path, candidate, root)
                if inferred is None:
                    findings.append(
                        Finding(
                            path=path,
                            line_no=line_no,
                            defect_class="ambiguous_target",
                            reference_text=candidate,
                            reason="non-clickable navigation target could not be inferred safely",
                        )
                    )
                    continue
                findings.append(
                    Finding(
                        path=path,
                        line_no=line_no,
                        defect_class="non_clickable_navigation",
                        reference_text=candidate,
                        reason=f"use [{repo_facing_label(inferred, root)}](...) instead of prose navigation",
                    )
                )

            if _is_dangling_bullet(lines, idx):
                findings.append(
                    Finding(
                        path=path,
                        line_no=line_no,
                        defect_class="dangling_bullet",
                        reference_text=line.strip(),
                        reason="bullet route heading ends with ':' but is immediately followed by a sibling bullet",
                    )
                )
    findings.extend(collect_orphan_findings(root))
    findings.extend(collect_parallel_reader_path_findings(root))
    return findings


def _is_dangling_bullet(lines: list[tuple[int, str]], idx: int) -> bool:
    line = lines[idx][1]
    bullet = BULLET_RE.match(line)
    if bullet is None:
        return False
    if not line.rstrip().endswith(":"):
        return False
    if MARKDOWN_LINK_RE.search(line) is not None:
        return False

    cur_indent = len(bullet.group("indent"))
    for _, next_line in lines[idx + 1 :]:
        if not next_line.strip():
            continue
        next_bullet = BULLET_RE.match(next_line)
        if next_bullet is None:
            return False
        next_indent = len(next_bullet.group("indent"))
        return next_indent <= cur_indent
    return False


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate public documentation links and navigation semantics.",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="repository root to validate",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    findings = collect_findings(root)
    if not findings:
        print("PASS: documentation link integrity")
        return 0

    print("FAIL: documentation link integrity")
    for finding in findings:
        rel = finding.path.relative_to(root).as_posix()
        print(
            f"{rel}:{finding.line_no}: {finding.defect_class}: "
            f"{finding.reference_text} ({finding.reason})"
        )
    return 1


if __name__ == "__main__":
    sys.exit(main())
