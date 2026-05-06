import { state } from './state.js';
import { showStatus, setNodeAtPath } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { saveToUndoStack } from './undoRedo.js';
import { insertTemplate } from './palette.js';

export function initializeMatrixBuilder() {
    const gridSelector = document.getElementById('matrix-grid-selector');

    for (let row = 0; row < 6; row++) {
        for (let col = 0; col < 6; col++) {
            const cell = document.createElement('div');
            cell.className = 'matrix-grid-cell';
            cell.dataset.row = row;
            cell.dataset.col = col;

            cell.addEventListener('mouseenter', () => {
                if (!state.matrixBuilderState.locked) {
                    highlightMatrixGrid(row + 1, col + 1);
                }
            });

            cell.addEventListener('click', () => {
                state.matrixBuilderState.locked = true;
                selectMatrixSize(row + 1, col + 1);
                highlightMatrixGrid(row + 1, col + 1);
                updateSizeDisplay();
            });

            gridSelector.appendChild(cell);
        }
    }

    gridSelector.addEventListener('mouseleave', () => {
        clearMatrixGrid();
        if (state.matrixBuilderState.rows <= 6 && state.matrixBuilderState.cols <= 6) {
            highlightMatrixGrid(state.matrixBuilderState.rows, state.matrixBuilderState.cols);
        }
    });

    document.getElementById('matrix-rows-input').addEventListener('input', (e) => {
        const value = Math.max(1, Math.min(10, parseInt(e.target.value) || 2));
        e.target.value = value;
        state.matrixBuilderState.rows = value;
        state.matrixBuilderState.cols = parseInt(document.getElementById('matrix-cols-input').value) || 2;
        state.matrixBuilderState.locked = false;
        updateSizeDisplay();
        if (value <= 6 && state.matrixBuilderState.cols <= 6) {
            highlightMatrixGrid(value, state.matrixBuilderState.cols);
        } else {
            clearMatrixGrid();
        }
    });

    document.getElementById('matrix-cols-input').addEventListener('input', (e) => {
        const value = Math.max(1, Math.min(10, parseInt(e.target.value) || 2));
        e.target.value = value;
        state.matrixBuilderState.cols = value;
        state.matrixBuilderState.rows = parseInt(document.getElementById('matrix-rows-input').value) || 2;
        state.matrixBuilderState.locked = false;
        updateSizeDisplay();
        if (state.matrixBuilderState.rows <= 6 && value <= 6) {
            highlightMatrixGrid(state.matrixBuilderState.rows, value);
        } else {
            clearMatrixGrid();
        }
    });

    document.querySelectorAll('.matrix-delimiter-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.matrix-delimiter-btn').forEach(b =>
                b.classList.remove('selected'));
            btn.classList.add('selected');
            state.matrixBuilderState.delimiter = btn.dataset.delimiter;
        });
    });

    document.addEventListener('click', (e) => {
        const modal = document.getElementById('matrix-builder-modal');
        if (e.target === modal) {
            closeMatrixBuilder();
        }
    });

    console.log('✓ Matrix builder initialized');
}

export function showMatrixBuilder() {
    const modal = document.getElementById('matrix-builder-modal');
    modal.classList.add('show');

    state.matrixBuilderState = { rows: 2, cols: 2, delimiter: 'bmatrix', locked: false };
    document.getElementById('matrix-rows-input').value = 2;
    document.getElementById('matrix-cols-input').value = 2;
    document.querySelectorAll('.matrix-delimiter-btn').forEach(b =>
        b.classList.remove('selected'));
    document.querySelector('[data-delimiter="bmatrix"]').classList.add('selected');

    clearMatrixGrid();
    highlightMatrixGrid(2, 2);
}

export function closeMatrixBuilder() {
    const modal = document.getElementById('matrix-builder-modal');
    modal.classList.remove('show');
    clearMatrixGrid();
}

function highlightMatrixGrid(rows, cols) {
    const cells = document.querySelectorAll('.matrix-grid-cell');
    cells.forEach(cell => {
        const cellRow = parseInt(cell.dataset.row);
        const cellCol = parseInt(cell.dataset.col);
        if (cellRow < rows && cellCol < cols) {
            cell.classList.add('hover');
        } else {
            cell.classList.remove('hover');
        }
    });

    state.matrixBuilderState.rows = rows;
    state.matrixBuilderState.cols = cols;
    document.getElementById('matrix-rows-input').value = rows;
    document.getElementById('matrix-cols-input').value = cols;
    updateSizeDisplay();
}

function clearMatrixGrid() {
    document.querySelectorAll('.matrix-grid-cell').forEach(cell => {
        cell.classList.remove('hover');
    });
}

function selectMatrixSize(rows, cols) {
    state.matrixBuilderState.rows = rows;
    state.matrixBuilderState.cols = cols;
    document.getElementById('matrix-rows-input').value = rows;
    document.getElementById('matrix-cols-input').value = cols;
    updateSizeDisplay();
}

function updateSizeDisplay() {
    const display = document.getElementById('matrix-size-display');
    display.textContent = `${state.matrixBuilderState.rows} × ${state.matrixBuilderState.cols}`;
}

function createMatrixAST(rows, cols, delimiter) {
    const totalElements = rows * cols;
    const args = [];

    for (let i = 0; i < totalElements; i++) {
        const row = Math.floor(i / cols) + 1;
        const col = (i % cols) + 1;
        args.push({
            Placeholder: {
                id: state.nextPlaceholderId++,
                hint: `a${row}${col}`
            }
        });
    }

    let opName;
    if (delimiter === 'pmatrix') {
        opName = 'PMatrix';
    } else if (delimiter === 'vmatrix') {
        opName = 'VMatrix';
    } else {
        opName = 'Matrix';
    }

    return {
        Operation: {
            name: opName,
            args: [
                { Const: String(rows) },
                { Const: String(cols) },
                { List: args }
            ]
        }
    };
}

export function createMatrixFromBuilder() {
    const { rows, cols, delimiter } = state.matrixBuilderState;

    if (state.editorMode === 'structural') {
        const ast = createMatrixAST(rows, cols, delimiter);

        saveToUndoStack();

        if (state.activeEditMarker) {
            setNodeAtPath(state.currentAST, state.activeEditMarker.path, ast);
            state.activeEditMarker = null;
            document.querySelectorAll('.arg-overlay').forEach(el => {
                el.classList.remove('active-marker');
            });
        } else {
            state.currentAST = ast;
        }

        renderStructuralEditor();
        showStatus(`✅ Created ${rows}×${cols} matrix`, 'success');
    } else {
        let latex = `\\begin{${delimiter}}`;
        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                latex += '□';
                if (c < cols - 1) latex += '&';
            }
            if (r < rows - 1) latex += '\\\\';
        }
        latex += `\\end{${delimiter}}`;

        const textarea = document.getElementById('latexInput');
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        textarea.value = text.substring(0, start) + latex + text.substring(end);
        const newPos = start + latex.length;
        textarea.setSelectionRange(newPos, newPos);
        textarea.focus();

        showStatus(`✅ Created ${rows}×${cols} matrix`, 'success');
    }

    closeMatrixBuilder();
}

export function insertMatrixFromPalette(rows, cols, delimiter) {
    if (state.editorMode === 'structural') {
        const ast = createMatrixAST(rows, cols, delimiter);
        saveToUndoStack();
        if (state.activeEditMarker) {
            setNodeAtPath(state.currentAST, state.activeEditMarker.path, ast);
            state.activeEditMarker = null;
            document.querySelectorAll('.arg-overlay').forEach(el => el.classList.remove('active-marker'));
        } else {
            state.currentAST = ast;
        }
        renderStructuralEditor();
        showStatus(`✅ Inserted ${rows}×${cols} matrix`, 'success');
    } else {
        let latex = `\\begin{${delimiter}}`;
        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                latex += '□';
                if (c < cols - 1) latex += '&';
            }
            if (r < rows - 1) latex += '\\\\\\\\';
        }
        latex += `\\end{${delimiter}}`;
        insertTemplate(latex);
    }
}
