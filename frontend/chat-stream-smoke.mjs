function extractUsageDraftTokens(usage) {
    if (!usage || typeof usage !== 'object') return null;

    const candidates = [
        'accepted_prediction_tokens',
        'accepted_speculative_tokens',
        'accepted_prediction_count',
        'accepted_speculative_count',
        'accepted_tokens',
        'draft_tokens',
        'speculative_tokens'
    ];

    for (let i = 0; i < candidates.length; i++) {
        const key = candidates[i];
        const value = usage[key];
        if (typeof value === 'number' && Number.isFinite(value) && value >= 0) {
            return value;
        }
    }

    return null;
}

function detectDraftTokenSignal(choice, delta) {
    const checks = [
        ['choice.speculative', choice && choice.speculative],
        ['choice.speculative_token', choice && choice.speculative_token],
        ['choice.is_speculative', choice && choice.is_speculative],
        ['choice.draft', choice && choice.draft],
        ['choice.is_draft', choice && choice.is_draft],
        ['delta.speculative', delta && delta.speculative],
        ['delta.speculative_token', delta && delta.speculative_token],
        ['delta.is_speculative', delta && delta.is_speculative],
        ['delta.draft', delta && delta.draft],
        ['delta.is_draft', delta && delta.is_draft],
        ['delta.prediction', delta && delta.prediction],
        ['delta.is_prediction', delta && delta.is_prediction],
        ['delta.predicted', delta && delta.predicted]
    ];

    for (let i = 0; i < checks.length; i++) {
        const label = checks[i][0];
        const value = checks[i][1];

        if (typeof value === 'boolean') {
            return { isDraft: value, source: label, raw: value };
        }

        if (typeof value === 'string' && value.toLowerCase() === 'draft') {
            return { isDraft: true, source: label, raw: value };
        }
    }

    return { isDraft: false, source: '', raw: null };
}

function simulateSendStream(lines) {
    let streamBuffer = '';
    let streamDone = false;
    const streamDebugState = {
        usageSeen: false,
        draftSignalsLogged: 0
    };

    let totalTokens = 0;
    let draftTokens = 0;
    let draftTokensFromSignals = 0;
    let hasUsageDraftTokens = false;
    let fullText = '';

    const processStreamLine = (line) => {
        const trimmedLine = String(line || '').trim();
        if (!trimmedLine || !trimmedLine.startsWith('data: ')) {
            return false;
        }

        const dataStr = trimmedLine.slice(6).trim();
        if (dataStr === '[DONE]') {
            return true;
        }

        try {
            const data = JSON.parse(dataStr);

            if (data.usage) {
                streamDebugState.usageSeen = true;
                const usageDraft = extractUsageDraftTokens(data.usage);
                if (typeof data.usage.completion_tokens === 'number') {
                    totalTokens = data.usage.completion_tokens;
                }

                if (typeof usageDraft === 'number') {
                    hasUsageDraftTokens = true;
                    draftTokens = usageDraft;
                }
            }

            if (!data.choices || data.choices.length === 0) {
                return false;
            }

            const choice = data.choices[0];
            const delta = choice && choice.delta ? choice.delta : {};
            const draftSignal = detectDraftTokenSignal(choice, delta);

            if (delta && delta.content) {
                const isDraft = draftSignal.isDraft === true;

                if (isDraft && !hasUsageDraftTokens) {
                    draftTokensFromSignals += 1;
                }

                fullText += String(delta.content);
            }

            if (typeof data.usage?.completion_tokens === 'number') {
                totalTokens = data.usage.completion_tokens;
            }
        } catch (e) {
            // intentionally ignored to mimic UI behavior
        }

        return false;
    };

    const chunks = Array.isArray(lines) ? lines : [];
    for (const chunk of chunks) {
        streamBuffer += chunk;
        const splitLines = streamBuffer.split(/\r?\n/);
        streamBuffer = splitLines.pop() || '';

        for (const line of splitLines) {
            if (processStreamLine(line)) {
                streamDone = true;
                break;
            }
        }

        if (streamDone) {
            break;
        }
    }

    if (!streamDone && streamBuffer.trim()) {
        processStreamLine(streamBuffer);
    }

    const finalDraftTokens = hasUsageDraftTokens ? draftTokens : draftTokensFromSignals;
    const mainTokens = Math.max(0, (totalTokens || 0) - (finalDraftTokens || 0));

    return {
        streamDone,
        fullText,
        totalTokens,
        draftTokens,
        draftTokensFromSignals,
        finalDraftTokens,
        mainTokens,
        usageSeen: streamDebugState.usageSeen,
        lineRest: streamBuffer
    };
}

function assertEqual(actual, expected, label) {
    if (actual !== expected) {
        throw new Error(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
    }
}

function runTest(name, fn) {
    try {
        fn();
        console.log(`PASS: ${name}`);
    } catch (error) {
        console.error(`FAIL: ${name}`);
        throw error;
    }
}

runTest('usage tokens preferred over draft signal', () => {
    const result = simulateSendStream([
        'data: {"choices":[{"delta":{"content":"Hel"}}]}\n',
        'data: {"usage":{"completion_tokens":10,"accepted_speculative_tokens":3}}\n',
        'data: {"choices":[{"delta":{"content":"lo "}}]}'
    ]);

    assertEqual(result.streamDone, false, 'streamDone');
    assertEqual(result.fullText, 'Hello ', 'fullText');
    assertEqual(result.totalTokens, 10, 'totalTokens');
    assertEqual(result.usageSeen, true, 'usageSeen');
    assertEqual(result.finalDraftTokens, 3, 'finalDraftTokens');
    assertEqual(result.mainTokens, 7, 'mainTokens');
});

runTest('trailing partial line is processed when no DONE', () => {
    const result = simulateSendStream([
        'data: {"choices":[{"delta":{"content":"tail"}}]}'
    ]);

    assertEqual(result.streamDone, false, 'streamDone');
    assertEqual(result.fullText, 'tail', 'fullText');
});

runTest('DONE line stops processing immediately', () => {
    const result = simulateSendStream([
        'data: {"choices":[{"delta":{"content":"before"}}]}\n',
        'data: [DONE]\n',
        'data: {"choices":[{"delta":{"content":"after-done"}}]}'
    ]);

    assertEqual(result.fullText, 'before', 'fullText');
    assertEqual(result.streamDone, true, 'streamDone');
});

runTest('draft signal fallback used when usage missing', () => {
    const result = simulateSendStream([
        'data: {"choices":[{"delta":{"content":"A","speculative_token":true}}]}\n',
        'data: {"choices":[{"delta":{"content":"B","is_draft":true}}]}\n',
        'data: {"choices":[{"delta":{"content":"C","prediction":"draft"}}]}\n',
        'data: [DONE]\n'
    ]);

    assertEqual(result.streamDone, true, 'streamDone');
    assertEqual(result.totalTokens, 0, 'totalTokens');
    assertEqual(result.finalDraftTokens, 3, 'finalDraftTokens');
    assertEqual(result.mainTokens, 0, 'mainTokens');
});

console.log('chat stream smoke checks complete');
