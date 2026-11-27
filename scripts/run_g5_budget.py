#!/usr/bin/env python3
"""Native-event G5-L1 capture. Build beforehand; this runner never compiles."""
import argparse,ctypes,hashlib,json,os,signal,socket,subprocess,tempfile,threading
from contextlib import ExitStack,contextmanager
from pathlib import Path
import g5_budget,g5_latency
ROOT=Path(__file__).resolve().parents[1]

def running_benchmark_pids(binary):
    result = subprocess.run(['/bin/ps', '-ax', '-o', 'pid=', '-o', 'comm='],
                            check=True, capture_output=True, text=True)
    rows = [line.strip().split(maxsplit=1) for line in result.stdout.splitlines()]
    return {int(row[0]) for row in rows if len(row) == 2 and row[1] == str(binary)}


@contextmanager
def native_key_source(path, binary):
    """Post keys from the authorized benchmark driver in both launch arms."""
    cg = ctypes.CDLL('/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics')
    cf = ctypes.CDLL('/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation')
    proc = ctypes.CDLL('/usr/lib/libproc.dylib')
    cg.CGPreflightPostEventAccess.restype = ctypes.c_bool
    if not cg.CGPreflightPostEventAccess():
        raise RuntimeError('The benchmark driver lacks permission to post native test keys')
    cg.CGEventCreateKeyboardEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint16, ctypes.c_bool]
    cg.CGEventCreateKeyboardEvent.restype = ctypes.c_void_p
    cg.CGEventSetIntegerValueField.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_int64]
    cg.CGEventPostToPid.argtypes = [ctypes.c_int, ctypes.c_void_p]
    cf.CFRelease.argtypes = [ctypes.c_void_p]
    proc.proc_pidpath.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_uint32]
    proc.proc_pidpath.restype = ctypes.c_int
    stopping = threading.Event()
    errors = []
    with socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM) as listener:
        listener.bind(str(path))
        listener.settimeout(.25)

        def post():
            expected_pid = None
            try:
                while not stopping.is_set():
                    try:
                        request = listener.recv(64)
                    except TimeoutError:
                        continue
                    pid, version = (int(x) for x in request.split())
                    if expected_pid is None:
                        buffer = ctypes.create_string_buffer(4096)
                        if proc.proc_pidpath(pid, buffer, len(buffer)) <= 0 or Path(os.fsdecode(buffer.value)) != binary:
                            raise RuntimeError('Key request did not come from the benchmark executable')
                        expected_pid = pid
                    if pid != expected_pid or not 1 <= version <= 10000:
                        raise RuntimeError('Unexpected native key request')
                    key = 7 if version % 2 else 51
                    for down in (True, False):
                        event = cg.CGEventCreateKeyboardEvent(None, key, down)
                        if not event:
                            raise RuntimeError('Could not create native keyboard event')
                        try:
                            if down:
                                # CGEventTypes.h kCGEventSourceUserData and native_input.m version tag.
                                cg.CGEventSetIntegerValueField(event, 42, 0x4c31000000000000 | version)
                            cg.CGEventPostToPid(pid, event)
                        finally:
                            cf.CFRelease(event)
            except Exception as error:
                errors.append(error)

        thread = threading.Thread(target=post, name='l1-native-key-source', daemon=True)
        thread.start()
        try:
            yield
        finally:
            stopping.set()
            thread.join()
    if errors:
        raise errors[0]


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--app',choices=['bare','lab','appkit'],required=True)
    p.add_argument('--launch',choices=['direct','services'],
                   help='Use the same bundle executable for direct and LaunchServices comparisons.')
    p.add_argument('--external-observer',action='store_true',
                   help='Read ScreenCaptureKit and post test keys from the benchmark driver in both launch arms.')
    p.add_argument('--physical',action='store_true',help='Measure physical X and Backspace keys instead of posting synthetic keys.')
    p.add_argument('--scene',choices=['typing','hover','drag','scroll'],default='typing')
    p.add_argument('--seconds',type=int,default=60)
    p.add_argument('--hz',choices=['10','30'],default='10')
    p.add_argument('--timeout',type=float)
    p.add_argument('--scheduler',choices=['0','1'],default='0')
    p.add_argument('--hl-fastpath',choices=['0','1'],
                   help='Override the independent PX-3 switch for profile comparisons.')
    p.add_argument('--trace-wakes',action='store_true')
    p.add_argument('--input-scoped',choices=['0','1'],default='0')
    p.add_argument('--target-hover',action='store_true',help='Use an NSEvent addressed to the benchmark window for hover.')
    p.add_argument('--switches',help='Four bits: immediate, draw-first, coalescing, any-change')
    p.add_argument('--damage', choices=['0', '1', 'verify'], default='0',
                   help='Explicit Stage G damage arm. The default is off.')
    p.add_argument('--passive',choices=['0','1'],required=True)
    p.add_argument('--pixels',choices=['0','1'],required=True)
    p.add_argument('--immediate',choices=['0','1'],default='1')
    p.add_argument('--any-change',choices=['0','1'],default='1')
    p.add_argument('--sparse',action='store_true')
    p.add_argument('--native-helper',type=Path)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    if a.scene == 'scroll' and a.app != 'lab':
        p.error('the scroll scene requires --app lab')
    a.output.mkdir(parents=True,exist_ok=False)
    assert 1<=a.seconds<=3600
    refs=ROOT/'target/headstart/g5-1'
    helper=a.native_helper or refs/'libG5Native.dylib'
    libs=[helper]+([refs/'libG5Pixels.dylib',refs/'libG5Refresh.dylib'] if a.app=='lab' else [])
    if os.environ.get('RUSTUI_G_R2_ALLOCATIONS') == '1':
        libs.append(refs / 'libGR2Allocations.dylib')
    bundles = {'bare': ('G5 Bare', 'latency-gpui'),
               'lab': ('RustUI Lab', 'rustui-lab'),
               'appkit': ('G5 AppKit', 'latency-appkit-display')}
    name, executable = bundles[a.app]
    bundle = refs/(name+'.app')
    binary = bundle/'Contents/MacOS'/executable
    if a.app == 'lab' and a.launch is None:
        binary = Path.home()/'.cache/rustui-target/release/rustui-lab'
    if a.app == 'appkit' and (a.scene != 'typing' or a.pixels != '1'):
        p.error('AppKit reference requires typing and ScreenCaptureKit')
    if a.external_observer and a.pixels != '1':
        p.error('--external-observer requires --pixels 1')
    if a.physical and a.scene != 'typing':
        p.error('--physical uses the X/Backspace typing workload')
    arm=a.switches or f'{a.immediate}{a.immediate}{a.immediate}{a.any_change}'
    if len(arm)!=4 or set(arm)-set('01'):
        p.error('--switches expects four bits')
    flags=dict(os.environ,RUSTUI_L1_NATIVE='1',RUSTUI_L1_PHYSICAL=str(int(a.physical)),RUSTUI_L1_LAB=str(int(a.app=='lab')),RUSTUI_L1_SPARSE=str(int(a.sparse)),
        RUSTUI_G5_REFERENCE='1',RUSTUI_DAMAGE=a.damage,RUSTUI_FRAME_LOG='buffered',RUSTUI_FRAME_PROFILE='1',RUSTUI_G0_LATENCY='1',RUSTUI_LATENCY_BREAKDOWN='1',
        RUSTUI_G5_PIXEL_CAPTURE=a.pixels,RUSTUI_G5_REFRESH=a.passive,RUSTUI_G5_SCENE=a.scene,RUSTUI_G5_INPUT_HZ=a.hz,RUSTUI_G0_TYPING_HZ=a.hz,
        RUSTUI_HL_FASTPATH=a.hl_fastpath or str(int(a.app=='lab' and a.scene=='typing')),GPUI_FAST_IMMEDIATE_PRESENT=arm[0],
        RUSTUI_DRAW_FIRST=arm[1],GPUI_FAST_IMMEDIATE_COALESCE=arm[2],GPUI_FAST_IMMEDIATE_ANY_CHANGE=arm[3],
        GPUI_FAST_NATIVE_SCHEDULER=a.scheduler,GPUI_FAST_INPUT_SCOPED_DEMAND=a.input_scoped,RUSTUI_L1_TARGET_HOVER=str(int(a.target_hover)),GPUI_FAST_TRACE_WAKES=str(int(a.trace_wakes)),
        GPUI_FAST_IMMEDIATE_HOLD_MS='250',GPUI_FAST_VIEW_DISPLAY_LINK='0',GPUI_FAST_FORCE_LEGACY_LINK='0',GPUI_FAST_FRAME_AHEAD='0',
        GPUI_FAST_IMMEDIATE_TRANSACTION='0',DYLD_INSERT_LIBRARIES=':'.join(str(f.resolve()) for f in libs))
    assert not a.sparse or (a.app=='bare' and a.seconds==120 and a.scene=='typing')
    observer_binary = refs/'l1-observer'
    artifacts = [binary,*libs]
    if a.external_observer:
        artifacts.append(observer_binary)
    before=g5_latency.collect_load(str(a.output/'load-start.txt'),[str(binary)],None)
    frozen={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in artifacts}
    (a.output/'method.json').write_text(json.dumps(dict(args=vars(a),flags={k:v for k,v in flags.items() if k.startswith(('RUSTUI_','GPUI_FAST_','DYLD_'))},frozen=frozen,load=before),indent=2,default=str)+'\n')
    cmd=[str(binary),str(a.seconds)] if a.app!='lab' else [str(binary),'--g0-scene',a.scene,'--duration',str(a.seconds)]
    log=a.output/'capture.log'
    stdout = a.output/'stdout.log'
    before_pids = running_benchmark_pids(binary.resolve()) if a.launch == 'services' else set()
    observer_log = a.output/'observer.log'
    with ExitStack() as resources:
        observer = None
        if a.external_observer:
            directory = Path(resources.enter_context(tempfile.TemporaryDirectory(prefix='l1-', dir='/tmp')))
            flags['RUSTUI_L1_CAPTURE_DIR'] = str(directory)
            if a.scene == 'typing' and not a.physical:
                key_socket = directory/'keys'
                resources.enter_context(native_key_source(key_socket,binary.resolve()))
                flags['RUSTUI_L1_KEY_SOCKET'] = str(key_socket)
            output = resources.enter_context(observer_log.open('w'))
            observer = subprocess.Popen([str(observer_binary),str(directory)],stdout=output,stderr=output)
        try:
            if a.launch == 'services':
                args = ['/usr/bin/open', '-n', '-W', '--stdout', str(stdout.resolve()), '--stderr', str(log.resolve())]
                for key,value in sorted(flags.items()):
                    if key.startswith(('RUSTUI_', 'GPUI_FAST_', 'DYLD_')):
                        args.extend(['--env', f'{key}={value}'])
                args.extend([str(bundle), '--args', *cmd[1:]])
                result = subprocess.run(args,cwd=ROOT,timeout=a.timeout or a.seconds+25)
            else:
                with stdout.open('w') as out, log.open('w') as err:
                    result=subprocess.run(cmd,cwd=ROOT,env=flags,stdout=out,stderr=err,timeout=a.timeout or a.seconds+20)
            assert result.returncode==0,('app exit',result.returncode)
        except subprocess.TimeoutExpired:
            if a.launch == 'services':
                for pid in running_benchmark_pids(binary.resolve()) - before_pids:
                    try:
                        os.kill(pid,signal.SIGTERM)
                    except ProcessLookupError:
                        pass
            (a.output/'failure.json').write_text(json.dumps({'error':'workload did not finish'})+'\n')
            raise
        finally:
            if observer:
                (directory/'stop').touch()
                try:
                    observer.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    observer.kill()
                    observer.wait()
        if observer and observer.returncode != 0:
            raise RuntimeError(f'ScreenCaptureKit observer exited {observer.returncode}; see {observer_log}')
    with log.open('ab') as out:
        for source in (stdout,observer_log):
            if source.exists():
                out.write(b'\n')
                out.write(source.read_bytes())
    assert frozen=={str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in artifacts},'build changed during capture'
    g5_budget.check_damage_allocations(log)
    after=g5_latency.collect_load(str(a.output/'load-end.txt'),[str(binary)],None)
    x=g5_budget.analyse_appkit(log) if a.app=='appkit' else g5_budget.analyse(log)
    g5_latency.annotate_load(x,before,after)
    load=x['machine_load']
    if max(load['start_background_cpu_percent'],load['end_background_cpu_percent']) <= 10:
        load['label']=(f"quiet capture ({load['start_background_cpu_percent']:.2f}% background at start; "
                       f"{load['end_background_cpu_percent']:.2f}% after)")
    load.pop('owner_authorized_bypass',None)
    (a.output/'summary.json').write_text(json.dumps(x,indent=2)+'\n')
    print(json.dumps({k:v for k,v in x.items() if k not in ('rows','next_drawable_outliers')},indent=2))
if __name__=='__main__':main()
