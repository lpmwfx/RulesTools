"""C# naming checks — from csharp/naming.md.

BANNED:
  - Banned generic variable names: data, info, value, item, object, temp, state,
    ctx, result, res, var, obj  (as standalone identifiers)
  - Boolean fields/properties not starting with is/has/can/should
  - Private fields not using _camelCase prefix
  - Interface name starting with 'I' but it's a class/struct/record (not interface)
    — inverse: interface NOT starting with 'I'
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "csharp/naming"

_BANNED_NAMES = {
    "data", "info", "value", "item", "object", "temp",
    "state", "ctx", "result", "res", "obj",
}

# Matches: var <banned> = / type <banned> = / (Type) <banned>
# Simple heuristic: local variable declaration  "Type bannedName ="  or  "var bannedName ="
_LOCAL_VAR = re.compile(
    r"(?:^|[;\{])\s*(?:var|\w[\w.<>\[\]?, ]*)\s+(\w+)\s*[=;(,)]"
)

# bool property/field without is/has/can/should prefix
# matches:  bool Enabled  /  bool Active  /  public bool Done
_BOOL_BAD = re.compile(
    r"\bbool\s+(?!is[A-Z_]|has[A-Z_]|can[A-Z_]|should[A-Z_]|Is[A-Z]|Has[A-Z]|Can[A-Z]|Should[A-Z])'?(\w+)"
)

# Interface declared without I prefix:  interface Foo  (not IFoo)
_INTERFACE_NO_I = re.compile(r"\binterface\s+([A-Z][a-z]\w*)\b")

# Private field not starting with _:  private (readonly|static|Type) fieldName
# Simplified: private ... fieldName where fieldName doesn't start with _
_PRIVATE_FIELD = re.compile(
    r"\bprivate\b(?:\s+(?:readonly|static|volatile))*\s+\w[\w.<>\[\]?]*\s+([a-zA-Z]\w*)\s*[=;{]"
)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue

        # --- Banned generic variable names ---
        for m in _LOCAL_VAR.finditer(raw):
            name = m.group(1)
            if name.lower() in _BANNED_NAMES:
                yield Issue(
                    file=path, line=lineno, col=m.start(1) + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-generic-names",
                    message=(
                        f"'{name}' — generic variable name. "
                        f"Use a domain-specific name that explains WHY it exists "
                        f"(e.g. 'parseResult', 'requestContext')"
                    ),
                )

        # --- Boolean without is/has/can/should ---
        if m := _BOOL_BAD.search(raw):
            name = m.group(1)
            # Skip parameter names and short-scope names
            if len(name) > 3:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/bool-prefix",
                    message=(
                        f"bool '{name}' — booleans must start with "
                        f"is/has/can/should (e.g. 'is{name.capitalize()}')"
                    ),
                )

        # --- Interface without I prefix ---
        if m := _INTERFACE_NO_I.search(raw):
            name = m.group(1)
            yield Issue(
                file=path, line=lineno, col=m.start(1) + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/interface-i-prefix",
                message=(
                    f"interface '{name}' — interfaces must start with 'I' "
                    f"(e.g. 'I{name}')"
                ),
            )

        # --- Private field without _ prefix ---
        if m := _PRIVATE_FIELD.search(raw):
            name = m.group(1)
            if not name.startswith("_") and not name[0].isupper():
                yield Issue(
                    file=path, line=lineno, col=m.start(1) + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/private-field-underscore",
                    message=(
                        f"private field '{name}' — private fields must use "
                        f"_camelCase prefix (e.g. '_{name}')"
                    ),
                )
