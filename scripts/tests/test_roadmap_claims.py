"""Focused concurrency and ownership checks for roadmap session claims."""

from __future__ import annotations

import os
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "roadmap_claims.py"


class RoadmapClaimsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.repo = self.base / "repo"
        self.repo.mkdir()
        self.git(self.repo, "init", "-q")
        self.git(self.repo, "config", "user.email", "claims@example.invalid")
        self.git(self.repo, "config", "user.name", "Claims Test")
        (self.repo / "README.md").write_text("seed\n", encoding="utf-8")
        self.git(self.repo, "add", "README.md")
        self.git(self.repo, "commit", "-qm", "seed")
        self.wt1 = self.base / "wt1"
        self.wt2 = self.base / "wt2"
        self.git(self.repo, "worktree", "add", "-qb", "codex/claims-one", str(self.wt1), "HEAD")
        self.git(self.repo, "worktree", "add", "-qb", "codex/claims-two", str(self.wt2), "HEAD")
        self.state = self.base / "state"
        self.db = self.state / "rustui-roadmap"
        self.env = {**os.environ, "XDG_STATE_HOME": str(self.state)}

    @staticmethod
    def git(root: Path, *args: str) -> str:
        return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()

    def run_claim(self, root: Path, thread: str, *paths: str, lease: int = 900,
                  task: str = "M-test") -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT), "claim", "--task", task, "--thread", thread,
             "--worktree", str(root), "--stage", "Stage test", "--milestone", "M-test",
             "--next-action", "check claim", "--lease-seconds", str(lease), *paths],
            cwd=root, env=self.env, text=True, capture_output=True,
        )

    def test_parallel_overlapping_claims_have_one_winner(self) -> None:
        commands = [
            [sys.executable, str(SCRIPT), "claim", "--task", "M1", "--thread", "thread-one",
             "--worktree", str(self.wt1), "--stage", "Stage 1", "--milestone", "M1",
             "--next-action", "implement", "src"],
            [sys.executable, str(SCRIPT), "claim", "--task", "M2", "--thread", "thread-two",
             "--worktree", str(self.wt2), "--stage", "Stage 2", "--milestone", "M2",
             "--next-action", "implement", "src/components/button.rs"],
        ]
        processes = [subprocess.Popen(command, cwd=root, env=self.env, text=True,
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                     for command, root in zip(commands, (self.wt1, self.wt2))]
        results = [process.communicate() + (process.returncode,) for process in processes]
        self.assertEqual(sum(code == 0 for _, _, code in results), 1, results)

    def test_active_task_id_cannot_be_claimed_twice_on_disjoint_paths(self) -> None:
        first = self.run_claim(self.wt1, "thread-task-one", "src/one.rs", task="M-unique")
        self.assertEqual(first.returncode, 0, first.stderr)
        second = self.run_claim(self.wt2, "thread-task-two", "src/two.rs", task="M-unique")
        self.assertNotEqual(second.returncode, 0)
        self.assertIn("claim conflicts", second.stderr)

    def test_progress_renews_expired_lease_and_records_next_action(self) -> None:
        claim = self.run_claim(self.wt1, "thread-progress", "src", lease=60)
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        database_path = next(self.db.glob("claims-*.sqlite3"))
        with sqlite3.connect(database_path) as database:
            database.execute("UPDATE claims SET lease_until=? WHERE id=?",
                             (int(time.time()) - 1, claim_id))
        progress = subprocess.run(
            [sys.executable, str(SCRIPT), "progress", "--claim-id", claim_id,
             "--thread", "thread-progress", "--stage", "Stage 4",
             "--milestone", "M-progress", "--next-action", "run focused check",
             "--lease-seconds", "120"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(progress.returncode, 0, progress.stderr)
        row = next(item for item in json.loads(subprocess.run(
            [sys.executable, str(SCRIPT), "list"], cwd=self.wt1, env=self.env,
            text=True, capture_output=True, check=True,
        ).stdout) if item["claim_id"] == claim_id)
        self.assertEqual(row["state"], "active")
        self.assertEqual(row["next_action"], "run focused check")
        self.assertGreater(row["lease_until"], int(time.time()))

    def test_primary_or_dirty_worktree_cannot_be_claimed(self) -> None:
        primary = self.run_claim(self.repo, "primary", "README.md")
        self.assertNotEqual(primary.returncode, 0)
        (self.wt1 / "preexisting.txt").write_text("preserve\n", encoding="utf-8")
        dirty = self.run_claim(self.wt1, "dirty", "preexisting.txt")
        self.assertNotEqual(dirty.returncode, 0)
        self.assertIn("clean", dirty.stderr)

    def test_audit_accepts_owned_edits_and_rejects_unowned_edits(self) -> None:
        (self.wt1 / "src").mkdir()
        claim = self.run_claim(self.wt1, "thread-audit", "src")
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        (self.wt1 / "src" / "owned.rs").write_text("// owned\n", encoding="utf-8")
        command = [sys.executable, str(SCRIPT), "audit", "--claim-id", claim_id,
                   "--thread", "thread-audit"]
        allowed = subprocess.run(command, cwd=self.wt1, env=self.env, text=True, capture_output=True)
        self.assertEqual(allowed.returncode, 0, allowed.stderr)
        (self.wt1 / "README.md").write_text("outside claim\n", encoding="utf-8")
        rejected = subprocess.run(command, cwd=self.wt1, env=self.env, text=True, capture_output=True)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("outside this claim", rejected.stderr)
        self.git(self.wt1, "add", "README.md")
        self.git(self.wt1, "commit", "-qm", "out-of-scope change")
        committed = subprocess.run(command, cwd=self.wt1, env=self.env, text=True, capture_output=True)
        self.assertNotEqual(committed.returncode, 0)
        self.assertIn("outside this claim", committed.stderr)

    def test_adopt_claims_assigned_paths_and_fingerprints_other_dirt(self) -> None:
        (self.wt1 / "src").mkdir()
        (self.wt1 / "src" / "existing.rs").write_text("// existing\n", encoding="utf-8")
        (self.wt1 / "README.md").write_text("modified\n", encoding="utf-8")
        adopted = subprocess.run(
            [sys.executable, str(SCRIPT), "adopt", "--task", "M-adopt",
             "--thread", "thread-adopt", "--worktree", str(self.wt1),
             "--stage", "Stage 3", "--milestone", "M-adopt",
             "--next-action", "continue existing work", "src",
             "@shared/target/headstart/evidence/m-adopt.log"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(adopted.returncode, 0, adopted.stderr)
        result = json.loads(adopted.stdout)
        self.assertEqual(result["adopted_paths"], ["@shared/target/headstart/evidence/m-adopt.log", "src"])
        self.assertEqual(result["baseline_paths"], ["readme.md"])
        compact = subprocess.run([sys.executable, str(SCRIPT), "list"], cwd=self.wt1,
                                 env=self.env, text=True, capture_output=True, check=True)
        compact_row = next(item for item in json.loads(compact.stdout)
                           if item["claim_id"] == result["claim_id"])
        self.assertEqual(compact_row["baseline_path_count"], 1)
        self.assertNotIn("baseline_paths", compact_row)
        listing = subprocess.run([sys.executable, str(SCRIPT), "list", "--verbose"], cwd=self.wt1,
                                 env=self.env, text=True, capture_output=True, check=True)
        listed = next(item for item in json.loads(listing.stdout)
                      if item["claim_id"] == result["claim_id"])
        self.assertEqual(listed["baseline_paths"], ["readme.md"])
        audit = [sys.executable, str(SCRIPT), "audit", "--claim-id", result["claim_id"],
                 "--thread", "thread-adopt"]
        unchanged = subprocess.run(audit, cwd=self.wt1, env=self.env, text=True,
                                   capture_output=True)
        self.assertEqual(unchanged.returncode, 0, unchanged.stderr)
        (self.wt1 / "README.md").write_text("changed after adoption\n", encoding="utf-8")
        self.git(self.wt1, "add", "README.md")
        self.git(self.wt1, "commit", "-qm", "change baseline after adoption")
        changed = subprocess.run(audit, cwd=self.wt1, env=self.env, text=True,
                                 capture_output=True)
        self.assertNotEqual(changed.returncode, 0)
        self.assertIn("outside this claim", changed.stderr)

    def test_baseline_preserves_only_unchanged_preexisting_paths(self) -> None:
        claim = self.run_claim(self.wt1, "thread-baseline", "src/owned.rs")
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        sidecar = self.wt1 / "graft" / "cache.json"
        sidecar.parent.mkdir()
        sidecar.write_text('{"cached":true}\n', encoding="utf-8")

        baseline = subprocess.run(
            [sys.executable, str(SCRIPT), "baseline", "--claim-id", claim_id,
             "--thread", "thread-baseline", "--evidence", "present before baseline", "graft/cache.json"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(baseline.returncode, 0, baseline.stderr)
        self.assertEqual(json.loads(baseline.stdout)["baseline_paths"], ["graft/cache.json"])
        repeated = subprocess.run(
            [sys.executable, str(SCRIPT), "baseline", "--claim-id", claim_id,
             "--thread", "thread-baseline", "--evidence", "present before baseline",
             "graft/cache.json"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(repeated.returncode, 0, repeated.stderr)

        audit = [sys.executable, str(SCRIPT), "audit", "--claim-id", claim_id,
                 "--thread", "thread-baseline"]
        unchanged = subprocess.run(audit, cwd=self.wt1, env=self.env, text=True, capture_output=True)
        self.assertEqual(unchanged.returncode, 0, unchanged.stderr)
        history = json.loads(unchanged.stdout)["history"]
        self.assertEqual(len(history), 1)
        self.assertEqual(history[-1]["kind"], "baseline")
        self.assertEqual(history[-1]["added_paths"], ["graft/cache.json"])
        self.assertEqual(history[-1]["evidence"], "present before baseline")

        sidecar.write_text('{"cached":false}\n', encoding="utf-8")
        changed = subprocess.run(audit, cwd=self.wt1, env=self.env, text=True, capture_output=True)
        self.assertNotEqual(changed.returncode, 0)
        self.assertIn("outside this claim", changed.stderr)

    def test_baseline_rejects_paths_owned_by_another_active_claim(self) -> None:
        claim = self.run_claim(self.wt1, "thread-baseline-conflict", "src/owned.rs")
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        other = self.run_claim(self.wt2, "thread-path-owner", "graft/cache.json", task="M-other")
        self.assertEqual(other.returncode, 0, other.stderr)
        sidecar = self.wt1 / "graft" / "cache.json"
        sidecar.parent.mkdir()
        sidecar.write_text("preserve\n", encoding="utf-8")

        baseline = subprocess.run(
            [sys.executable, str(SCRIPT), "baseline", "--claim-id", claim_id,
             "--thread", "thread-baseline-conflict", "--evidence", "present before baseline",
             "graft/cache.json"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertNotEqual(baseline.returncode, 0)
        self.assertIn("conflicts with active claim", baseline.stderr)

    def test_rescope_releases_clean_paths_atomically_and_records_history(self) -> None:
        claim = self.run_claim(self.wt1, "thread-rescope", "old/file.rs")
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        rescope = subprocess.run(
            [sys.executable, str(SCRIPT), "rescope", "--claim-id", claim_id,
             "--thread", "thread-rescope", "--stage", "Stage 8",
             "--milestone", "M1222", "--next-action", "write the acceptance run sheet",
             "docs/new-run-sheet.md"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(rescope.returncode, 0, rescope.stderr)
        self.assertEqual(json.loads(rescope.stdout)["removed_paths"], ["old/file.rs"])

        listing = subprocess.run(
            [sys.executable, str(SCRIPT), "list", "--verbose"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True, check=True,
        )
        row = next(item for item in json.loads(listing.stdout) if item["claim_id"] == claim_id)
        self.assertEqual(row["paths"], ["docs/new-run-sheet.md"])
        audit = subprocess.run(
            [sys.executable, str(SCRIPT), "audit", "--claim-id", claim_id,
             "--thread", "thread-rescope"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(audit.returncode, 0, audit.stderr)
        history = json.loads(audit.stdout)["history"]
        self.assertEqual(history[-1]["kind"], "rescope")
        self.assertEqual(history[-1]["evidence"], '{"removed_paths": ["old/file.rs"]}')
        reclaimed = self.run_claim(
            self.wt2, "thread-reclaim", "old/file.rs", task="M-reclaim-old-path"
        )
        self.assertEqual(reclaimed.returncode, 0, reclaimed.stderr)

    def test_rescope_rejects_dropping_a_changed_claimed_path(self) -> None:
        claim = self.run_claim(self.wt1, "thread-rescope-dirty", "old/file.rs")
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        (self.wt1 / "old").mkdir()
        (self.wt1 / "old" / "file.rs").write_text("preserve\n", encoding="utf-8")
        rescope = subprocess.run(
            [sys.executable, str(SCRIPT), "rescope", "--claim-id", claim_id,
             "--thread", "thread-rescope-dirty", "--stage", "Stage 8",
             "--milestone", "M1222", "--next-action", "write the acceptance run sheet",
             "docs/new-run-sheet.md"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertNotEqual(rescope.returncode, 0)
        self.assertIn("cannot drop claimed paths containing changes", rescope.stderr)
        listing = subprocess.run(
            [sys.executable, str(SCRIPT), "list"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True, check=True,
        )
        row = next(item for item in json.loads(listing.stdout) if item["claim_id"] == claim_id)
        self.assertEqual(row["paths"], ["old/file.rs"])

    def test_rescope_rejects_conflicting_scope_without_mutating_claim(self) -> None:
        first = self.run_claim(self.wt1, "thread-rescope-conflict", "old/file.rs", task="M-first")
        second = self.run_claim(self.wt2, "thread-new-owner", "docs/new-run-sheet.md", task="M-second")
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        claim_id = json.loads(first.stdout)["claim_id"]
        rescope = subprocess.run(
            [sys.executable, str(SCRIPT), "rescope", "--claim-id", claim_id,
             "--thread", "thread-rescope-conflict", "--stage", "Stage 8",
             "--milestone", "M1222", "--next-action", "write the acceptance run sheet",
             "docs/new-run-sheet.md"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertNotEqual(rescope.returncode, 0)
        self.assertIn("rescope conflicts with active claims", rescope.stderr)
        listing = subprocess.run(
            [sys.executable, str(SCRIPT), "list"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True, check=True,
        )
        row = next(item for item in json.loads(listing.stdout) if item["claim_id"] == claim_id)
        self.assertEqual(row["paths"], ["old/file.rs"])

    def test_expired_claim_stays_locked_until_explicit_release(self) -> None:
        claim = self.run_claim(self.wt1, "thread-expiring", "src", lease=60)
        self.assertEqual(claim.returncode, 0, claim.stderr)
        database_path = next(self.db.glob("claims-*.sqlite3"))
        with sqlite3.connect(database_path) as database:
            database.execute("UPDATE claims SET lease_until=?", (int(time.time()) - 1,))
        contender = self.run_claim(self.wt2, "thread-contender", "src/file.rs")
        self.assertNotEqual(contender.returncode, 0)
        listing = subprocess.run([sys.executable, str(SCRIPT), "list"], cwd=self.wt1,
                                 env=self.env, text=True, capture_output=True, check=True)
        self.assertEqual(json.loads(listing.stdout)[0]["state"], "suspect")

    def test_handoff_transfers_identity_without_dropping_old_worktree_changes(self) -> None:
        claim = self.run_claim(self.wt1, "thread-handoff-old", "src/owned.rs")
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        old_change = self.wt1 / "src" / "owned.rs"
        old_change.parent.mkdir()
        old_change.write_text("old session work\n", encoding="utf-8")

        handoff = subprocess.run(
            [sys.executable, str(SCRIPT), "handoff", "--claim-id", claim_id,
             "--old-thread", "thread-handoff-old", "--new-thread", "thread-handoff-new",
             "--new-worktree", str(self.wt2), "--stage", "Stage 2", "--milestone", "M-next",
             "--next-action", "continue owned work", "--terminal-state", "superseded",
             "--confirmed-terminal", "--evidence", "old thread confirmed terminal"],
            cwd=self.wt1, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(handoff.returncode, 0, handoff.stderr)
        self.assertTrue(old_change.exists(), "handoff must preserve old worktree changes")

        audit = subprocess.run(
            [sys.executable, str(SCRIPT), "audit", "--claim-id", claim_id,
             "--thread", "thread-handoff-new"],
            cwd=self.wt2, env=self.env, text=True, capture_output=True,
        )
        self.assertEqual(audit.returncode, 0, audit.stderr)
        history = json.loads(audit.stdout)["history"]
        self.assertEqual(history[-1]["kind"], "handoff")
        self.assertEqual(history[-1]["to_base_head"], self.git(self.wt2, "rev-parse", "HEAD"))

    def test_legacy_claim_gets_branch_identity_backfilled(self) -> None:
        claim = self.run_claim(self.wt1, "thread-legacy", "src")
        self.assertEqual(claim.returncode, 0, claim.stderr)
        claim_id = json.loads(claim.stdout)["claim_id"]
        database_path = next(self.db.glob("claims-*.sqlite3"))
        with sqlite3.connect(database_path) as database:
            database.execute("UPDATE claims SET branch='' WHERE id=?", (claim_id,))
        listing = subprocess.run([sys.executable, str(SCRIPT), "list"], cwd=self.wt1,
                                 env=self.env, text=True, capture_output=True, check=True)
        row = next(item for item in json.loads(listing.stdout) if item["claim_id"] == claim_id)
        self.assertEqual(row["branch"], "codex/claims-one")


if __name__ == "__main__":
    unittest.main()
