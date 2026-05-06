import { state } from './state.js';
import { showStatus, parseSimpleInput, setNodeAtPath, getNodeValueAtPath } from './astUtils.js';
import { renderStructuralEditor } from './render.js';

export function classifyButtonType(latex) {
    if (/^[a-zA-Z0-9+\-=×÷±∓·∗∞<>]$/.test(latex)) return 'symbol';
    if (/^\\(alpha|beta|gamma|delta|epsilon|zeta|eta|theta|iota|kappa|lambda|mu|nu|xi|omicron|pi|rho|sigma|tau|upsilon|phi|chi|psi|omega)$/.test(latex)) return 'symbol';
    if (/^\\(Gamma|Delta|Theta|Lambda|Xi|Pi|Sigma|Upsilon|Phi|Psi|Omega)$/.test(latex)) return 'symbol';
    if (/^\\(leq|geq|neq|approx|equiv|in|notin|subset|subseteq|cup|cap|emptyset|to|Rightarrow|Leftrightarrow|forall|exists|neg|land|lor)$/.test(latex)) return 'symbol';
    if (/^\\(times|div|cdot|pm|mp|ast|partial|nabla|infty|square)$/.test(latex)) return 'symbol';
    if (/\\(sin|cos|tan|arcsin|arccos|arctan|ln|log|exp)\(/.test(latex)) return 'function';
    if (latex.includes('□') || latex.includes('\\frac') || latex.includes('\\begin') ||
        latex.includes('\\sqrt') || latex.includes('\\int') || latex.includes('\\sum') ||
        latex.includes('\\left') || latex.includes('\\langle') || latex.includes('\\binom')) {
        return 'template';
    }
    return 'symbol';
}

export function isInlineEditorActive() {
    return state.inlineEditorState.active && state.inlineEditorState.foreignObject &&
           state.inlineEditorState.foreignObject.style.display !== 'none';
}

export function showInlineEditor(marker) {
    console.log('showInlineEditor called with marker:', marker);
    state.inlineEditorState.marker = marker;
    state.inlineEditorState.active = true;

    const svg = document.querySelector('#structuralEditor svg');
    console.log('Found SVG:', svg);
    if (!svg) {
        console.error('No SVG found! Cannot show inline editor.');
        return;
    }

    let foreignObject = svg.querySelector('#inline-editor-foreign');
    console.log('Existing foreignObject:', foreignObject);

    if (!foreignObject) {
        console.log('Creating new foreignObject');
        foreignObject = document.createElementNS('http://www.w3.org/2000/svg', 'foreignObject');
        foreignObject.id = 'inline-editor-foreign';
        svg.appendChild(foreignObject);
        console.log('ForeignObject appended to SVG');

        const wrapper = document.createElementNS('http://www.w3.org/1999/xhtml', 'div');
        wrapper.style.width = '100%';
        wrapper.style.height = '100%';

        const input = document.createElementNS('http://www.w3.org/1999/xhtml', 'input');
        input.setAttribute('type', 'text');
        input.setAttribute('id', 'inline-input');
        input.setAttribute('class', 'inline-edit-input');
        input.setAttribute('autocomplete', 'off');
        input.setAttribute('spellcheck', 'false');
        input.setAttribute('placeholder', 'Type or click symbols...');
        input.style.width = '100%';
        input.style.height = '100%';
        input.style.boxSizing = 'border-box';

        wrapper.appendChild(input);
        foreignObject.appendChild(wrapper);
        console.log('Input element created in XHTML wrapper');

        setupInlineEditorHandlers(input);
        console.log('Keyboard handlers setup');
    }

    state.inlineEditorState.foreignObject = foreignObject;
    state.inlineEditorState.input = foreignObject.querySelector('input') || document.getElementById('inline-input');
    console.log('State updated. Input:', state.inlineEditorState.input);

    if (!state.inlineEditorState.input) {
        console.error('ERROR: Could not find input element!');
        return;
    }

    const bbox = marker.bbox;
    console.log('Marker bbox:', bbox);

    const posX = Math.max(0, bbox.x - 10);
    const posY = Math.max(0, bbox.y - 5);
    const width = Math.max(200, bbox.width + 40);
    const height = Math.max(40, bbox.height + 10);

    foreignObject.setAttribute('x', posX);
    foreignObject.setAttribute('y', posY);
    foreignObject.setAttribute('width', width);
    foreignObject.setAttribute('height', height);
    console.log('ForeignObject positioned at:', { x: posX, y: posY, width, height });

    foreignObject.style.display = 'block';
    foreignObject.style.overflow = 'visible';
    foreignObject.style.pointerEvents = 'all';
    foreignObject.setAttribute('visibility', 'visible');
    console.log('ForeignObject display set to block, visibility ensured');

    const currentValue = getNodeValueAtPath(state.currentAST, marker.path);
    state.inlineEditorState.input.value = currentValue || '';
    console.log('Input value set to:', currentValue);

    setTimeout(() => {
        state.inlineEditorState.input.focus();
        state.inlineEditorState.input.select();
        console.log('Input focused and selected');
    }, 10);

    marker.element.classList.add('editing-inline');
    document.body.classList.add('inline-editing');
    console.log('Visual feedback added. Editor should be visible now!');

    if (window.enableClickOutside) {
        window.enableClickOutside();
    }
}

export function hideInlineEditor(commit = true) {
    console.log('hideInlineEditor called, commit:', commit);
    if (!state.inlineEditorState.active) {
        console.log('Inline editor not active, returning');
        return;
    }

    if (commit && state.inlineEditorState.marker) {
        const value = state.inlineEditorState.input.value.trim();
        try {
            const node = parseSimpleInput(value);
            setNodeAtPath(state.currentAST, state.inlineEditorState.marker.path, node);
            renderStructuralEditor();
            if (value) {
                showStatus('✅ Value updated', 'success');
            } else {
                showStatus('✅ Cleared to placeholder', 'success');
            }
        } catch (error) {
            console.error('Inline edit error:', error);
            showStatus('⚠️ Invalid input', 'error');
            return;
        }
    }

    if (state.inlineEditorState.foreignObject) {
        state.inlineEditorState.foreignObject.style.display = 'none';
    }
    if (state.inlineEditorState.input) {
        state.inlineEditorState.input.value = '';
        state.inlineEditorState.input.classList.remove('error');
    }
    if (state.inlineEditorState.marker && state.inlineEditorState.marker.element) {
        state.inlineEditorState.marker.element.classList.remove('editing-inline');
    }

    document.body.classList.remove('inline-editing');
    state.inlineEditorState.active = false;
    state.inlineEditorState.marker = null;
}

export function appendToInlineEditor(text) {
    if (!isInlineEditorActive()) return;

    const input = state.inlineEditorState.input;
    const start = input.selectionStart || 0;
    const end = input.selectionEnd || 0;
    const currentValue = input.value;

    input.value = currentValue.substring(0, start) + text + currentValue.substring(end);

    const newPos = start + text.length;
    input.setSelectionRange(newPos, newPos);
    input.focus();
}

export function setupInlineEditorHandlers(input) {
    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            hideInlineEditor(true);
        } else if (e.key === 'Escape') {
            e.preventDefault();
            hideInlineEditor(false);
        } else if (e.key === 'Tab') {
            e.preventDefault();
            hideInlineEditor(true);
        }
    });

    let clickOutsideEnabled = false;
    document.addEventListener('click', (e) => {
        if (!clickOutsideEnabled) return;

        if (isInlineEditorActive()) {
            const foreignObject = state.inlineEditorState.foreignObject;
            const modal = document.getElementById('replace-confirm-modal');

            if (foreignObject && !foreignObject.contains(e.target) &&
                !modal.contains(e.target) &&
                !e.target.closest('.math-btn') &&
                !e.target.closest('.arg-overlay') &&
                !e.target.closest('.matrix-builder-modal')) {
                console.log('Click outside detected, hiding inline editor');
                hideInlineEditor(true);
            }
        }
    });

    window.enableClickOutside = function() {
        setTimeout(() => {
            clickOutsideEnabled = true;
            console.log('Click outside handler enabled');
        }, 100);
    };
}

export function showReplaceConfirmation(currentText, template) {
    state.inlineEditorState.pendingTemplate = template;

    const modal = document.getElementById('replace-confirm-modal');
    const message = document.getElementById('replace-message');

    message.textContent = `Replace "${currentText}" with template?`;
    modal.classList.add('show');

    document.getElementById('replace-confirm-yes').onclick = () => {
        modal.classList.remove('show');
        hideInlineEditor(false);
        // Late-bind to avoid circular import
        window.insertTemplate(template);
    };

    document.getElementById('replace-confirm-no').onclick = () => {
        modal.classList.remove('show');
        if (state.inlineEditorState.input) {
            state.inlineEditorState.input.focus();
        }
    };
}

export function classifyAllButtons() {
    document.querySelectorAll('.math-btn').forEach(btn => {
        const onclick = btn.getAttribute('onclick');
        if (!onclick) return;

        const match = onclick.match(/insert(Symbol|Template)\('([^']+)'\)/);
        if (!match) return;

        const [_, funcType, latex] = match;
        const buttonType = funcType === 'Symbol' ? 'symbol' : classifyButtonType(latex);

        btn.setAttribute('data-button-type', buttonType);
    });
    console.log('✓ All palette buttons classified');
}

export function initializeInlineEditing() {
    classifyAllButtons();
    console.log('✓ Inline editing system initialized');
}
