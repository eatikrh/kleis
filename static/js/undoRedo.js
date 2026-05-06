import { state, MAX_UNDO_HISTORY } from './state.js';
import { showStatus } from './astUtils.js';
import { renderStructuralEditor } from './render.js';

export function saveToUndoStack() {
    if (state.currentAST) {
        const snapshot = JSON.parse(JSON.stringify(state.currentAST));
        state.undoStack.push(snapshot);
        if (state.undoStack.length > MAX_UNDO_HISTORY) {
            state.undoStack.shift();
        }
        state.redoStack = [];
        updateUndoRedoButtons();
    }
}

export function undo() {
    if (state.undoStack.length === 0) {
        showStatus('Nothing to undo', 'error');
        return;
    }
    if (state.currentAST) {
        state.redoStack.push(JSON.parse(JSON.stringify(state.currentAST)));
    }
    state.currentAST = state.undoStack.pop();
    renderStructuralEditor();
    updateUndoRedoButtons();
    showStatus('✅ Undo', 'success');
}

export function redo() {
    if (state.redoStack.length === 0) {
        showStatus('Nothing to redo', 'error');
        return;
    }
    if (state.currentAST) {
        state.undoStack.push(JSON.parse(JSON.stringify(state.currentAST)));
    }
    state.currentAST = state.redoStack.pop();
    renderStructuralEditor();
    updateUndoRedoButtons();
    showStatus('✅ Redo', 'success');
}

export function updateUndoRedoButtons() {
    const undoBtn = document.getElementById('undoBtn');
    const redoBtn = document.getElementById('redoBtn');
    if (undoBtn) {
        undoBtn.disabled = state.undoStack.length === 0;
        undoBtn.title = `Undo (${state.undoStack.length} actions)`;
    }
    if (redoBtn) {
        redoBtn.disabled = state.redoStack.length === 0;
        redoBtn.title = `Redo (${state.redoStack.length} actions)`;
    }
}
