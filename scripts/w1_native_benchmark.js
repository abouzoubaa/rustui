// CUA REPL driver, not a standalone Node program. Bind the verification app with
// cua.getApp(bundlePath), open Layout, then call runW1Cycle(app, phase, session,
// records). Run three warm-up cycles, then 30 measured cycles per fresh launch.
// Every action derives its index from a fresh native accessibility observation.
// Durations include automation/observation overhead; they are NOT app latency.
async function runW1Cycle(app, phase, session, records) {
    const started = Date.now();
    let tree = await app.getAXState({disableDiffing: true, emit: false});
    const states = [];
    for (let step = 0; step <= 6; step++) {
        const label = step === 0 ? 'Reset W1' : 'Next W1 step';
        const line = tree.split('\n').find(x => x.trim().endsWith('button ' + label));
        if (!line) throw new Error('Missing control: ' + label);
        await app.click(Number(line.trim().split(' ')[0]));
        tree = await app.getAXState({disableDiffing: true, emit: false});
        const actual = tree.split('\n')
            .map(x => x.match(/button Increment (.+): count (\d+)$/))
            .filter(Boolean).map(x => [x[1], Number(x[2])]);
        const prefix = step === 3 ? 'Parent \u00b7 ' : '';
        const expected = [[prefix + 'Alpha', step >= 1 ? 1 : 0],
                          [prefix + 'Beta', step >= 2 ? 1 : 0]];
        if (step === 5) expected.reverse();
        if (JSON.stringify(actual) !== JSON.stringify(expected)) {
            throw new Error('W1 state mismatch at step ' + step + ': ' + JSON.stringify(actual));
        }
        states.push({step, children: actual});
    }
    const result = {session, phase, started_ms: started, ended_ms: Date.now(), states};
    records.push(result);
    return result;
}
