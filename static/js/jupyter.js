import { state, isJupyterMode } from './state.js';
import { showStatus } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { setEditorMode } from './modeConvert.js';

export function sendToJupyter() {
    if (!state.currentAST) {
        showStatus('❌ No equation to send. Build an equation first!', 'error');
        return;
    }

    if (!isJupyterMode) {
        showStatus('⚠️ Not in Jupyter mode. Use ?mode=jupyter in URL.', 'warning');
        return;
    }

    window.parent.postMessage({
        type: 'kleisEquation',
        payload: {
            ast: state.currentAST,
            timestamp: new Date().toISOString()
        }
    }, '*');

    showStatus('✅ Equation sent to Jupyter notebook!', 'success');
    console.log('Sent equation to Jupyter:', state.currentAST);
}

export function initJupyter() {
    window.addEventListener('message', function(event) {
        if (event.data && event.data.type === 'kleisInitialData') {
            console.log('Received initial AST from Jupyter:', event.data.payload);

            if (event.data.payload) {
                state.currentAST = event.data.payload;

                if (state.editorMode !== 'structural') {
                    setEditorMode('structural');
                }

                renderStructuralEditor();
                showStatus('✅ Equation loaded from Jupyter!', 'success');
            }
        }
    });

    if (isJupyterMode) {
        console.log('🟢 Kleis Editor running in Jupyter mode');
        window.parent.postMessage({ type: 'kleisRequestInitial' }, '*');
    }
}
