import { state } from './state.js';
import { showStatus, setNodeAtPath } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { saveToUndoStack } from './undoRedo.js';

export function showPiecewiseBuilder() {
    const modal = document.getElementById('piecewise-builder-modal');
    modal.classList.add('show');
    document.getElementById('piecewise-cases-input').value = 2;
    updatePiecewisePreview();
}

export function closePiecewiseBuilder() {
    const modal = document.getElementById('piecewise-builder-modal');
    modal.classList.remove('show');
}

export function updatePiecewisePreview() {
    const cases = parseInt(document.getElementById('piecewise-cases-input').value) || 2;
    const preview = document.getElementById('piecewise-preview');

    let text = 'f(x) = {';
    for (let i = 1; i <= cases; i++) {
        text += ` expr${i}  if cond${i}`;
        if (i < cases) text += '\n       {';
    }

    preview.textContent = text;
}

export function createPiecewiseFromBuilder() {
    const cases = parseInt(document.getElementById('piecewise-cases-input').value) || 2;

    if (state.editorMode === 'structural') {
        const exprs = [];
        const conds = [];

        for (let i = 0; i < cases; i++) {
            exprs.push({
                Placeholder: {
                    id: state.nextPlaceholderId++,
                    hint: `expr${i+1}`
                }
            });
            conds.push({
                Placeholder: {
                    id: state.nextPlaceholderId++,
                    hint: `cond${i+1}`
                }
            });
        }

        const ast = {
            Operation: {
                name: 'Piecewise',
                args: [
                    { Const: String(cases) },
                    { List: exprs },
                    { List: conds }
                ]
            }
        };

        if (state.activeEditMarker) {
            setNodeAtPath(state.currentAST, state.activeEditMarker.path, ast);
            state.activeEditMarker = null;
            document.querySelectorAll('.arg-overlay').forEach(el => {
                el.classList.remove('active-marker');
            });
        } else {
            saveToUndoStack();
            state.currentAST = ast;
        }

        renderStructuralEditor();
        showStatus(`✅ Created ${cases}-case piecewise function`, 'success');
    } else {
        let latex = '\\begin{cases}';
        for (let i = 0; i < cases; i++) {
            latex += '□ & \\text{if } □';
            if (i < cases - 1) latex += '\\\\';
        }
        latex += '\\end{cases}';

        const textarea = document.getElementById('latexInput');
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        textarea.value = text.substring(0, start) + latex + text.substring(end);
        const newPos = start + latex.length;
        textarea.setSelectionRange(newPos, newPos);
        textarea.focus();

        showStatus(`✅ Created ${cases}-case piecewise function`, 'success');
    }

    closePiecewiseBuilder();
}

export function initPiecewiseBuilder() {
    document.addEventListener('click', (e) => {
        const modal = document.getElementById('piecewise-builder-modal');
        if (e.target === modal) {
            closePiecewiseBuilder();
        }
    });
}
