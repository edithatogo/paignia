"""Render an admitted static v1 wire game as a Mermaid flowchart."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
import subprocess


def stable_id(kind, identifier):
    return kind + sha256(identifier.encode("utf-8")).hexdigest()


def label(value):
    return "".join(
        character if character.isalnum() or character in " .,:=()/−-"
        else f"#{ord(character)};"
        for character in str(value)
    )


def compact_name(identifier):
    base, separator, fragment = identifier.rpartition("#")
    if separator:
        return f"{base.rsplit('/', 1)[-1]}/{fragment}".lstrip("/")
    return identifier.rsplit(":", 1)[-1].rsplit("/", 1)[-1]


def diagram(request):
    lines = [
        "flowchart TD",
        "  accTitle: Finite decision game",
        "  accDescr: Decision and chance branches with labeled actions and terminal payoffs.",
    ]
    for item in sorted(request["nodes"], key=lambda entry: entry["id"]):
        node_id = stable_id("n", item["id"])
        node = item["node"]
        name = label(compact_name(item["id"]))
        if node["kind"] == "decision":
            owner = label(compact_name(node["player"]))
            lines.append(f'  {node_id}{{"{name}: {owner}"}}')
        elif node["kind"] == "chance":
            lines.append(f'  {node_id}(("{name}: chance"))')
        else:
            payoffs = sorted(node["payoffs"], key=lambda entry: entry["player"])
            utilities = ", ".join(
                f'{compact_name(payoff["player"])} = {payoff["value"]}' for payoff in payoffs
            )
            lines.append(f'  {node_id}["{name}: {label(utilities)}"]')

        edges = node.get("actions", node.get("outcomes", []))
        for edge in edges:
            edge_id = stable_id("e", edge["id"])
            target = stable_id("n", edge["target"])
            action = compact_name(edge["action"]) if "action" in edge else edge["probability"]
            lines.append(f'  {node_id} {edge_id}@-->|"{label(action)}"| {target}')
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--solver", required=True, type=Path)
    parser.add_argument("request", type=Path)
    args = parser.parse_args()
    with args.request.open("rb") as stream:
        request = stream.read((1 << 20) + 1)
    subprocess.run(
        [args.solver.resolve()], input=request, stdout=subprocess.DEVNULL, timeout=30, check=True,
    )
    print(diagram(json.loads(request)), end="")


if __name__ == "__main__":
    main()
