import { state, templateMap, astTemplates } from './state.js';
import { showStatus, latexToUnicode, setNodeAtPath, renumberPlaceholders } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { saveToUndoStack } from './undoRedo.js';
import { isInlineEditorActive, appendToInlineEditor, hideInlineEditor, showReplaceConfirmation } from './inlineEdit.js';
import { renderEquation } from './verify.js';

export function insertSymbol(latex) {
    const symbol = latexToUnicode(latex);

    if (state.editorMode === 'structural') {
        if (isInlineEditorActive()) {
            appendToInlineEditor(symbol);
            return;
        }

        if (state.activeEditMarker) {
            const symbolNode = { Object: symbol };
            setNodeAtPath(state.currentAST, state.activeEditMarker.path, symbolNode);

            state.activeEditMarker = null;
            document.querySelectorAll('.arg-overlay').forEach(el => {
                el.classList.remove('active-marker');
            });

            renderStructuralEditor();
            showStatus('✅ Symbol inserted', 'success');
        } else {
            state.currentAST = { Object: symbol };
            renderStructuralEditor();
            showStatus('✅ Symbol inserted', 'success');
        }
        return;
    }

    const textarea = document.getElementById('latexInput');
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const text = textarea.value;
    textarea.value = text.substring(0, start) + latex + text.substring(end);
    const newPos = start + latex.length;
    textarea.setSelectionRange(newPos, newPos);
    textarea.focus();
    renderEquation();
}

export function insertTemplate(template) {
    if (state.editorMode === 'structural') {
        if (isInlineEditorActive()) {
            const currentInput = state.inlineEditorState.input.value.trim();
            if (currentInput) {
                showReplaceConfirmation(currentInput, template);
                return;
            } else {
                hideInlineEditor(false);
            }
        }

        if (state.activeEditMarker) {
            insertStructuralTemplateAt(template, state.activeEditMarker.path);
        } else {
            insertStructuralTemplate(template);
        }
    } else {
        const textarea = document.getElementById('latexInput');
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        textarea.value = text.substring(0, start) + template + text.substring(end);
        const placeholderIdx = template.indexOf('□');
        const newPos = start + (placeholderIdx >= 0 ? placeholderIdx : template.length);
        textarea.setSelectionRange(newPos, newPos + (placeholderIdx >= 0 ? 1 : 0));
        textarea.focus();
    }
}

export function insertStructuralTemplateAt(latexTemplate, path) {
    console.log('insertStructuralTemplateAt:', latexTemplate, 'at path:', path);

    const name = templateMap[latexTemplate];
    if (!name) {
        alert('Template not implemented in structural mode yet.');
        return;
    }

    let ast = astTemplates[name];
    if (!ast) {
        ast = { Placeholder: { id: state.nextPlaceholderId++, hint: name } };
    } else {
        ast = JSON.parse(JSON.stringify(ast));
        renumberPlaceholders(ast);
    }

    setNodeAtPath(state.currentAST, path, ast);

    state.activeEditMarker = null;
    document.querySelectorAll('.arg-overlay').forEach(el => {
        el.classList.remove('active-marker');
    });

    renderStructuralEditor();
    showStatus('✅ Template inserted', 'success');
}

export function insertStructuralTemplate(latexTemplate) {
    const name = templateMap[latexTemplate];
    console.log('insertStructuralTemplate called with:', latexTemplate);
    console.log('Mapped to template name:', name);

    if (!name) {
        alert('Template not implemented in structural mode yet.');
        return;
    }

    saveToUndoStack();

    let ast = astTemplates[name];
    console.log('AST template before clone:', ast);

    if (!ast) {
        console.log('No AST template found, using fallback');
        ast = { Placeholder: { id: state.nextPlaceholderId++, hint: name } };
    } else {
        ast = JSON.parse(JSON.stringify(ast));
        console.log('AST after clone:', ast);
        renumberPlaceholders(ast);
        console.log('AST after renumber:', ast);
    }

    state.currentAST = ast;
    console.log('Final currentAST:', state.currentAST);
    renderStructuralEditor();
}
