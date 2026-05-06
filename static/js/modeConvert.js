import { state, API_BASE } from './state.js';
import { showStatus } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { renderEquation } from './verify.js';

export function setEditorMode(mode) {
    state.editorMode = mode;
    document.querySelectorAll('.mode-btn').forEach(b => b.classList.remove('active'));
    const activeBtn = document.querySelector(`.mode-btn[onclick*="'${mode}'"]`);
    if (activeBtn) {
        activeBtn.classList.add('active');
    }

    const textControls = document.getElementById('textControls');
    const latexInput = document.getElementById('latexInput');
    const structEditor = document.getElementById('structuralEditor');
    const structControls = document.getElementById('structuralControls');
    const inputLabel = document.getElementById('inputLabel');

    if (mode === 'text') {
        textControls.style.display = 'flex';
        latexInput.style.display = 'block';
        structEditor.style.display = 'none';
        structControls.style.display = 'none';
        inputLabel.textContent = '📝 LaTeX Input';

        if (state.currentAST) {
            convertStructuralToText();
        }
    } else {
        textControls.style.display = 'none';
        latexInput.style.display = 'none';
        structEditor.style.display = 'flex';
        structControls.style.display = 'block';
        inputLabel.textContent = '🔧 Structural Editor';

        const latex = latexInput.value.trim();
        if (latex && !state.currentAST) {
            convertTextToStructural(latex);
        } else if (!state.currentAST) {
            window.resetStructuralEditor();
        }
    }
}

async function convertStructuralToText() {
    try {
        showStatus('Converting to LaTeX...', 'info');

        const response = await fetch(`${API_BASE}/render_ast`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                ast: state.currentAST,
                format: 'latex'
            })
        });

        if (!response.ok) {
            throw new Error('Failed to render AST to LaTeX');
        }

        const data = await response.json();

        const latexInput = document.getElementById('latexInput');
        latexInput.value = data.output;

        await renderEquation();
        showStatus('✅ Converted to text mode!', 'success');
    } catch (error) {
        showStatus('❌ Error converting: ' + error.message, 'error');
        console.error('Conversion error:', error);
    }
}

async function convertTextToStructural(latex) {
    const previousAST = state.currentAST;

    try {
        showStatus('Converting LaTeX to structural format...', 'info');

        const response = await fetch(`${API_BASE}/parse`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ latex })
        });

        if (!response.ok) {
            throw new Error('Failed to parse LaTeX');
        }

        const data = await response.json();
        state.currentAST = data.ast;

        await renderStructuralEditor();
        showStatus('✅ Converted to structural mode!', 'success');
    } catch (error) {
        showStatus('❌ Error parsing LaTeX: ' + error.message + ' (keeping previous AST)', 'error');
        console.error('Conversion error:', error);

        if (previousAST) {
            state.currentAST = previousAST;
            await renderStructuralEditor();
        } else {
            window.resetStructuralEditor();
        }
    }
}
