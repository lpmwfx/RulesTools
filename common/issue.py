"""Issue dataclass — VSCode problem-matcher compatible output format."""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path


class Severity(str, Enum):
    ERROR = "error"
    WARNING = "warning"
    INFO = "info"


@dataclass(order=True)
class Issue:
    """One rule violation.

    Output format:  path:line:col: severity rule: message
    Matches VSCode default problem matcher pattern.
    """

    file: Path
    line: int
    col: int
    severity: Severity
    rule: str       # e.g. "rust/errors/no-unwrap" or "global/file-limits/slint"
    message: str

    # sort key — file then line
    sort_index: tuple = field(init=False, repr=False, compare=True)

    def __post_init__(self) -> None:
        self.sort_index = (str(self.file), self.line, self.col)

    def __str__(self) -> str:
        return (
            f"{self.file}:{self.line}:{self.col}: "
            f"{self.severity.value} {self.rule}: {self.message}"
        )
