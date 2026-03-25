#!/usr/bin/env python3
import json
import subprocess
import sys
from collections import deque

SOURCES = {"dpw4", "geom-signal", "geom-spatial"}
TARGETS = {"replay-core", "replay-embed", "replay-host", "replay-cli"}


def package_name(pkg_id: str, id_to_name: dict[str, str]) -> str:
    return id_to_name.get(pkg_id, pkg_id)


def main() -> int:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        return result.returncode

    meta = json.loads(result.stdout)
    workspace_members = set(meta["workspace_members"])
    packages = meta["packages"]
    id_to_name = {pkg["id"]: pkg["name"] for pkg in packages}

    resolve_nodes = meta["resolve"]["nodes"]
    graph = {node["id"]: [dep["pkg"] for dep in node["deps"]] for node in resolve_nodes}

    source_ids = sorted(
        pkg_id
        for pkg_id in workspace_members
        if package_name(pkg_id, id_to_name) in SOURCES
    )

    violations: list[dict[str, object]] = []

    for source_id in source_ids:
        source_name = package_name(source_id, id_to_name)
        queue = deque([(source_id, [source_id])])
        seen = {source_id}

        while queue:
            current, path = queue.popleft()
            for nxt in sorted(graph.get(current, []), key=lambda i: package_name(i, id_to_name)):
                if nxt in seen:
                    continue
                seen.add(nxt)
                next_path = path + [nxt]
                target_name = package_name(nxt, id_to_name)
                if target_name in TARGETS:
                    named_path = [package_name(pkg_id, id_to_name) for pkg_id in next_path]
                    violations.append(
                        {
                            "from_package": source_name,
                            "to_package": target_name,
                            "path": named_path,
                            "path_len": len(named_path),
                        }
                    )
                    continue
                queue.append((nxt, next_path))

    violations.sort(
        key=lambda v: (
            str(v["from_package"]),
            str(v["to_package"]),
            int(v["path_len"]),
            "->".join(v["path"]),
        )
    )

    if violations:
        for violation in violations:
            print(json.dumps(violation, separators=(",", ":"), sort_keys=True))
        return 1

    print(json.dumps({"status": "ok", "violations": 0}, separators=(",", ":"), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
