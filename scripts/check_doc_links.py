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


@dataclass(frozen=True)
class Finding:
    path: Path
    line_no: int
    defect_class: str
    reference_text: str
    reason: str


def repo_facing_label(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def iter_public_docs(root: Path) -> list[Path]:
    files = [root / "README.md"]
    docs_dir = root / "docs"
    for path in sorted(docs_dir.rglob("*.md")):
        rel = path.relative_to(root).as_posix()
        if rel.startswith("docs/wip/"):
            continue
        if rel.startswith("docs/audits/"):
            continue
        if rel.startswith("docs/archive/"):
            continue
        if rel.startswith("docs/internal/"):
            continue
        if rel.startswith("docs/verification/releases/"):
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
    return match.group("quoted") is not None


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


def collect_findings(root: Path) -> list[Finding]:
    findings: list[Finding] = []
    for path in iter_public_docs(root):
        for line_no, line in iter_non_fenced_lines(path):
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
    return findings


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