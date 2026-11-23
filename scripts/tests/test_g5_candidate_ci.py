from pathlib import Path
import os,subprocess,tempfile,unittest
ROOT=Path(__file__).resolve().parents[2]
class CandidateCiTests(unittest.TestCase):
    def test_only_integration_branch_and_target_pr_select_candidates(self):
        workflow=(ROOT/'.github/workflows/ci.yml').read_text()
        script=workflow.split('      - name: Verify source state and exact patch replay\n',1)[1].split('        run: |\n',1)[1].split('      - name:',1)[0]
        script='\n'.join(line[10:] for line in script.splitlines())
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);(p/'scripts').mkdir();(p/'bin').mkdir()
            (p/'scripts/verify-release.sh').write_text('#!/bin/sh\nprintf "approved %s\\n" "$*"\n');(p/'scripts/verify-release.sh').chmod(0o755)
            (p/'bin/python3').write_text('#!/bin/sh\nprintf "candidate %s\\n" "$*"\n');(p/'bin/python3').chmod(0o755)
            for event,branch,base,candidate in [('pull_request','17/merge','g5-integration',True),('pull_request','g5-integration','main',False),('push','g5-integration','',True),('push','main','',False),('workflow_dispatch','g5-integration','',True),('workflow_dispatch','main','',False),('push','g5-integration;echo bad','',False)]:
                result=subprocess.run(['bash','-eu','-c',script],cwd=p,env=dict(os.environ,PATH=str(p/'bin')+os.pathsep+os.environ['PATH'],RUSTUI_CI_EVENT=event,RUSTUI_CI_BRANCH=branch,RUSTUI_CI_PR_BASE=base),capture_output=True,text=True)
                self.assertEqual(result.returncode,0,result.stderr)
                self.assertEqual(result.stdout.startswith('candidate '),candidate,(event,branch,base,result.stdout))
                if candidate:self.assertIn('--candidate-stack patches/candidate-fast-latency/latency-stack.json',result.stdout)
                else:self.assertEqual(result.stdout.strip(),'approved source')
        self.assertNotIn('--candidate-stack',(ROOT/'scripts/verify-release.sh').read_text())
        self.assertIn('branches: [main, g5-integration]',workflow)
