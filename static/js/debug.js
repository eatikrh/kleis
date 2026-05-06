import { state, API_BASE, COORDINATE_PREFERENCE } from './state.js';
import { showStatus, formatASTAsTree, countPlaceholdersInAST, countOperationsInAST, getASTDepth, countNodesInAST } from './astUtils.js';

export function toggleDebugPanel() {
    const panel = document.getElementById('debugPanel');
    if (panel.style.display === 'none') {
        panel.style.display = 'block';
        updateDebugPanel();
    } else {
        panel.style.display = 'none';
    }
}

export function updateDebugPanel() {
    const content = document.getElementById('debugASTContent');
    if (!state.currentAST) {
        content.textContent = 'No AST available. Create an expression first.';
        return;
    }

    const astJSON = JSON.stringify(state.currentAST, null, 2);
    const treeView = formatASTAsTree(state.currentAST, 0);

    let markerInfo = '=== Marker Placement Info ===\n\n';
    if (state.lastRenderResponse) {
        markerInfo += '--- Placeholders (from square glyph extraction) ---\n';
        if (state.lastRenderResponse.placeholders && state.lastRenderResponse.placeholders.length > 0) {
            state.lastRenderResponse.placeholders.forEach(ph => {
                markerInfo += `  ID ${ph.id}: (x=${ph.x.toFixed(2)}, y=${ph.y.toFixed(2)}, w=${ph.width}, h=${ph.height})\n`;
            });
        } else {
            markerInfo += '  (none)\n';
        }

        markerInfo += '\n--- Argument Bounding Boxes (semantic) ---\n';
        if (state.lastRenderResponse.argument_bounding_boxes && state.lastRenderResponse.argument_bounding_boxes.length > 0) {
            state.lastRenderResponse.argument_bounding_boxes.forEach(bbox => {
                markerInfo += `  Arg ${bbox.arg_index} [${bbox.node_id}]: (x=${bbox.x.toFixed(2)}, y=${bbox.y.toFixed(2)}, w=${bbox.width.toFixed(2)}, h=${bbox.height.toFixed(2)})\n`;
            });
        } else {
            markerInfo += '  (none)\n';
        }

        markerInfo += '\n--- Argument Slots (all editable positions) ---\n';
        if (state.lastRenderResponse.argument_slots && state.lastRenderResponse.argument_slots.length > 0) {
            state.lastRenderResponse.argument_slots.forEach(slot => {
                markerInfo += `  Slot ${slot.id}: is_placeholder=${slot.is_placeholder}, hint="${slot.hint}", path=[${slot.path}], role=${slot.role || 'null'}\n`;
            });
        } else {
            markerInfo += '  (none)\n';
        }

        markerInfo += '\n--- Coordinate System Used ---\n';
        markerInfo += `  COORDINATE_PREFERENCE = '${COORDINATE_PREFERENCE}'\n`;
        markerInfo += `  (semantic-first uses argument_bounding_boxes primarily)\n`;
    } else {
        markerInfo += '(Render the expression first to see marker data)\n';
    }

    content.textContent = '=== AST Structure ===\n\n' + astJSON + '\n\n' +
                          '=== Tree View ===\n\n' + treeView + '\n\n' +
                          '=== Statistics ===\n\n' +
                          `Placeholders: ${countPlaceholdersInAST(state.currentAST)}\n` +
                          `Operations: ${countOperationsInAST(state.currentAST)}\n` +
                          `Depth: ${getASTDepth(state.currentAST)}\n` +
                          `Total nodes: ${countNodesInAST(state.currentAST)}\n\n` +
                          markerInfo;
}

export function copyASTToClipboard() {
    if (!state.currentAST) {
        alert('No AST to copy');
        return;
    }
    const astJSON = JSON.stringify(state.currentAST, null, 2);
    navigator.clipboard.writeText(astJSON).then(() => {
        showStatus('✅ AST copied to clipboard', 'success');
    }).catch(err => {
        console.error('Failed to copy:', err);
        showStatus('❌ Failed to copy', 'error');
    });
}

export async function copyTypstToClipboard() {
    if (!state.currentAST) {
        showStatus('⚠️ No equation to export', 'warning');
        return;
    }
    try {
        showStatus('🔄 Generating Typst code...', 'info');
        const response = await fetch(`${API_BASE}/export_typst`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ast: state.currentAST })
        });
        const result = await response.json();
        if (result.success && result.typst) {
            await navigator.clipboard.writeText(result.typst);
            showStatus(`✅ Typst copied! Paste into your thesis: "${result.typst.substring(0, 50)}${result.typst.length > 50 ? '...' : ''}"`, 'success');
        } else {
            showStatus('❌ Failed to generate Typst: ' + (result.error || 'Unknown error'), 'error');
        }
    } catch (err) {
        console.error('Failed to export Typst:', err);
        showStatus('❌ Failed to export Typst: ' + err.message, 'error');
    }
}

export function downloadAST() {
    if (!state.currentAST) {
        alert('No AST to download');
        return;
    }
    const astJSON = JSON.stringify(state.currentAST, null, 2);
    const blob = new Blob([astJSON], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `kleis_ast_${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
    showStatus('✅ AST downloaded', 'success');
}
