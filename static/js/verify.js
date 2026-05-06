import { state, API_BASE } from './state.js';
import { showStatus } from './astUtils.js';

export async function renderEquation() {
    const latexInput = document.getElementById('latexInput');
    if (!latexInput) return;

    const latex = latexInput.value;
    const previewDiv = document.getElementById('preview');

    if (!latex.trim()) {
        showStatus('Please enter a LaTeX equation', 'error');
        return;
    }

    try {
        previewDiv.innerHTML = `\\[${latex}\\]`;
        if (window.MathJax) await MathJax.typesetPromise([previewDiv]);
        showStatus('✅ Rendered successfully!', 'success');
    } catch (error) {
        showStatus('❌ Error: ' + error.message, 'error');
    }
}

export async function verifyWithZ3() {
    let ast;
    if (state.editorMode === 'structural' && state.currentAST) {
        ast = state.currentAST;
    } else {
        const latexInput = document.getElementById('latexInput');
        if (!latexInput || !latexInput.value.trim()) {
            showStatus('Please enter an expression to verify', 'error');
            return;
        }
        try {
            const parseResponse = await fetch(`${API_BASE}/parse`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ latex: latexInput.value })
            });
            const parseResult = await parseResponse.json();
            if (!parseResult.success) {
                showStatus('❌ Parse error: ' + parseResult.error, 'error');
                return;
            }
            ast = parseResult.ast;
        } catch (e) {
            showStatus('❌ Failed to parse: ' + e.message, 'error');
            return;
        }
    }

    showStatus('🔄 Verifying with Z3...', 'info');

    try {
        const response = await fetch(`${API_BASE}/verify`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ast: ast })
        });
        const result = await response.json();

        if (!result.success) {
            showStatus('❌ Verification error: ' + result.error, 'error');
            return;
        }

        const resultEmoji = result.result === 'valid' ? '✅' :
                           result.result === 'invalid' ? '❌' :
                           result.result === 'unknown' ? '❓' : '⚠️';

        let message = `${resultEmoji} Z3: ${result.result.toUpperCase()}`;
        if (result.kleis_syntax) {
            message += `\n📝 Kleis: ${result.kleis_syntax}`;
        }
        if (result.counterexample) {
            message += `\n⚡ Counterexample: ${result.counterexample}`;
        }

        showStatus(message, result.result === 'valid' ? 'success' :
                           result.result === 'invalid' ? 'error' : 'warning');

        const debugContent = document.getElementById('debugASTContent');
        if (debugContent) {
            const existingContent = debugContent.textContent;
            debugContent.textContent = `--- Z3 Verification ---\nResult: ${result.result}\nKleis: ${result.kleis_syntax}\n${result.counterexample ? 'Counterexample: ' + result.counterexample : ''}\n\n${existingContent}`;
        }
    } catch (e) {
        showStatus('❌ Verification failed: ' + e.message, 'error');
    }
}

export async function checkSatisfiable() {
    let ast;
    if (state.editorMode === 'structural' && state.currentAST) {
        ast = state.currentAST;
    } else {
        const latexInput = document.getElementById('latexInput');
        if (!latexInput || !latexInput.value.trim()) {
            showStatus('Please enter an expression to check', 'error');
            return;
        }
        try {
            const parseResponse = await fetch(`${API_BASE}/parse`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ latex: latexInput.value })
            });
            const parseResult = await parseResponse.json();
            if (!parseResult.success) {
                showStatus('❌ Parse error: ' + parseResult.error, 'error');
                return;
            }
            ast = parseResult.ast;
        } catch (e) {
            showStatus('❌ Failed to parse: ' + e.message, 'error');
            return;
        }
    }

    showStatus('🔄 Checking satisfiability...', 'info');

    try {
        const response = await fetch(`${API_BASE}/check_sat`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ast: ast })
        });
        const result = await response.json();

        if (!result.success) {
            showStatus('❌ Check error: ' + result.error, 'error');
            return;
        }

        const resultEmoji = result.result === 'satisfiable' ? '✅' :
                           result.result === 'unsatisfiable' ? '❌' :
                           result.result === 'unknown' ? '❓' : '⚠️';

        let message = `${resultEmoji} SAT: ${result.result.toUpperCase()}`;
        if (result.kleis_syntax) {
            message += `\n📝 Kleis: ${result.kleis_syntax}`;
        }
        if (result.example) {
            message += `\n💡 Example: ${result.example}`;
        }

        showStatus(message, result.result === 'satisfiable' ? 'success' :
                           result.result === 'unsatisfiable' ? 'error' : 'warning');

        const debugContent = document.getElementById('debugASTContent');
        if (debugContent) {
            const existingContent = debugContent.textContent;
            debugContent.textContent = `--- SAT Check ---\nResult: ${result.result}\nKleis: ${result.kleis_syntax}\n${result.example ? 'Example: ' + result.example : ''}\n\n${existingContent}`;
        }
    } catch (e) {
        showStatus('❌ SAT check failed: ' + e.message, 'error');
    }
}

export async function checkTypesDebounced() {
    clearTimeout(state.typeCheckTimeout);
    state.typeCheckTimeout = setTimeout(async () => {
        await checkTypes();
    }, 500);
}

export async function checkTypes() {
    if (!state.currentAST) {
        hideTypeIndicator();
        return;
    }

    try {
        const indicator = document.getElementById('type-indicator');
        const content = document.getElementById('type-indicator-content');

        indicator.style.display = 'block';
        indicator.style.borderLeftColor = '#999';
        content.innerHTML = '⏳ Type checking...';
        content.style.color = '#999';

        const response = await fetch(`${API_BASE}/type_check`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ast: state.currentAST })
        });

        const result = await response.json();

        if (result.success) {
            indicator.style.borderLeftColor = '#4CAF50';
            content.innerHTML = `<span style="color: #4CAF50;">✓ Type:</span> <span style="color: #333; font-weight: 600;">${result.type_name}</span>`;
            if (result.suggestion) {
                content.innerHTML += `<br><span style="color: #FF9800; font-size: 12px;">💡 ${result.suggestion}</span>`;
            }
        } else {
            indicator.style.borderLeftColor = '#f44336';
            content.innerHTML = `<span style="color: #f44336;">✗ ${result.error}</span>`;
            if (result.suggestion) {
                content.innerHTML += `<br><span style="color: #FF9800; font-size: 12px;">💡 ${result.suggestion}</span>`;
            }
        }
    } catch (error) {
        console.error('Type check failed:', error);
        hideTypeIndicator();
    }
}

export function hideTypeIndicator() {
    const indicator = document.getElementById('type-indicator');
    if (indicator) {
        indicator.style.display = 'none';
    }
}
