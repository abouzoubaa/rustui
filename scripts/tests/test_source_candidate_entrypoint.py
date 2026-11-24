import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_source_candidates import integration_body

class SourceCandidateEntrypointTests(unittest.TestCase):
    def test_all_existing_source_checks_are_retained_verbatim(self):
        source='header\n  source)\n    before\n    python3 scripts/apply_gpui_kit_09_stack.py --check\n    after\n    ;;\n  macos)\n'
        result=integration_body(source,Path('/candidate path/stack.json'))
        expected="    before\n    python3 scripts/apply_gpui_kit_09_stack.py --check --candidate-stack '/candidate path/stack.json'\n    after"
        self.assertEqual(result,expected)
        self.assertNotIn('--candidate-stack',source)

    def test_missing_approved_check_is_an_error_not_a_skipped_check(self):
        with self.assertRaises(ValueError):integration_body('  source)\n    other\n    ;;\n',Path('stack.json'))

if __name__=='__main__':unittest.main()
