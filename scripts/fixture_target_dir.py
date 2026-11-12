"""Select fixture build caches that cannot overlap the application workspace cache."""

from __future__ import annotations

from pathlib import Path


class FixtureTargetError(ValueError):
    """The fixture target would reuse or contain workspace build artifacts."""


def fixture_target_root(kit_version: int, workspace_target_dir: Path) -> Path:
    if kit_version not in (8, 9):
        raise FixtureTargetError(f"unsupported GPUI Kit fixture version: {kit_version}")
    workspace = workspace_target_dir.expanduser().resolve()
    fixture = (
        Path.home() / ".cache" / f"rustui-target-fixture-kit{kit_version}"
    ).resolve()
    if fixture == workspace or fixture in workspace.parents or workspace in fixture.parents:
        raise FixtureTargetError(
            f"Kit {kit_version} fixture cache {fixture} overlaps workspace cache {workspace}"
        )
    return fixture
