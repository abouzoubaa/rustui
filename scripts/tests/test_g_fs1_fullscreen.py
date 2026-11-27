import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from g_fs1_fullscreen import FIELDS, summarize


class FullscreenSummaryTest(unittest.TestCase):
    def test_last_size_interval_and_fallback_attribution(self):
        lines = ['G_FS1_ENTER 3', 'RUSTUI_G_R2_LOG_LOST 0']
        for event, t in [('start', 5004), ('end', 5007)]:
            lines.append('RUSTUI_G0_LOG ' + json.dumps(dict(event=event, epoch_ms=t, valid=True)))
        for i, size, mode, reason in [(1, 100, 'retained', 'prime'), (2, 200, 'direct', 'resize'),
                                     (3, 100, 'retained', 'prime'), (4, 100, 'partial', 'changed'),
                                     (5, 0, 'deferred', 'admission'), (6, 100, 'direct', 'verify_reference_budget')]:
            row = dict.fromkeys(FIELDS, 0)
            row.update(index=i, epoch_ms=5000 + i, drawable_w=size, drawable_h=size, drawable_format=80,
                       decision=mode, reason=reason, budget_requested=40000 if i == 6 else 0)
            if mode != 'deferred':
                lines.append('RUSTUI_FRAME_LOG ' + json.dumps(dict(event='gpu', mode=mode)))
            lines.append('G_FS1_FRAME ' + ' '.join(str(row[k]) for k in FIELDS))
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'capture.log'
            path.write_text('\n'.join(lines))
            result = summarize(path)
        self.assertEqual(result['draws'], [1, 5])
        self.assertEqual(result['after_last_size'], [1, 3])
        self.assertEqual(result['measured'], [1, 2])
        self.assertEqual(result['first_partial'], '4')
        self.assertEqual(result['non_submissions'], {'deferred/admission': 1})
        self.assertEqual(result['failures'][0]['reason'], 'verify_reference_budget')

