import { state, MIN_ZOOM, MAX_ZOOM, ZOOM_STEP } from './state.js';
import { showStatus, getNodeAtPath, getNodeById, parseSimpleInput, setNodeAtPath, getAllMarkers } from './astUtils.js';
import { renderStructuralEditor, applyZoom } from './render.js';
import { saveToUndoStack } from './undoRedo.js';
import { showInlineEditor } from './inlineEdit.js';

export function handleSlotClick(event, id, path, nodeId) {
    console.log('Clicked slot:', { id, path, nodeId });

    if (event) {
        event.stopPropagation();
        event.preventDefault();
    }

    try {
        const isModifierClick = event && (event.shiftKey || event.ctrlKey || event.metaKey);

        saveToUndoStack();

        state.activeEditMarker = {
            id: id,
            path: path,
            nodeId: nodeId,
            element: event.target,
            bbox: {
                x: parseFloat(event.target.getAttribute('x')),
                y: parseFloat(event.target.getAttribute('y')),
                width: parseFloat(event.target.getAttribute('width')),
                height: parseFloat(event.target.getAttribute('height'))
            }
        };

        document.querySelectorAll('.arg-overlay').forEach(el => {
            el.classList.remove('active-marker');
        });
        if (event.target) event.target.classList.add('active-marker');

        const node = nodeId ? getNodeById(state.currentAST, nodeId) : getNodeAtPath(state.currentAST, path);
        let val = "";
        if (node && node.Const) val = node.Const;
        if (node && node.Object) val = node.Object;

        if (isModifierClick) {
            showStatus('📍 Marker selected (dialog mode).', 'info');
            const input = prompt("Enter value (or click Cancel to insert template from palette):", val);
            if (input !== null) {
                const newNode = parseSimpleInput(input);
                setNodeAtPath(state.currentAST, path, newNode);
                state.activeEditMarker = null;
                renderStructuralEditor();
            }
        } else {
            showStatus('✨ Inline edit mode. Type directly or click symbols.', 'info');
            showInlineEditor(state.activeEditMarker);
        }
    } catch (error) {
        console.error('Error in handleSlotClick:', error);
        alert('Error: ' + error.message);
    }
}

export function handleSlotKeydown(event, id, path, nodeId) {
    if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        handleSlotClick(event, id, path, nodeId);
    }
}

export function toggleBoundingBoxes() {
    const checkbox = document.getElementById('showBoundingBoxes');
    const svg = document.querySelector('#structuralEditor svg');
    if (svg) {
        const g = svg.querySelector('#arg-overlays');
        if (g) g.setAttribute('visibility', checkbox.checked ? 'visible' : 'hidden');
    }
}

export function resetStructuralEditor() {
    state.currentAST = null;
    state.nextPlaceholderId = 0;
    renderStructuralEditor();
}

export function zoomIn() {
    state.currentZoom = Math.min(MAX_ZOOM, state.currentZoom + ZOOM_STEP);
    applyZoom();
}

export function zoomOut() {
    state.currentZoom = Math.max(MIN_ZOOM, state.currentZoom - ZOOM_STEP);
    applyZoom();
}

export function zoomReset() {
    state.currentZoom = 1.0;
    applyZoom();
}

export function focusNextMarker() {
    const markers = getAllMarkers();
    if (markers.length === 0) {
        showStatus('No markers available', 'error');
        return;
    }

    let currentIndex = -1;
    if (state.activeEditMarker) {
        currentIndex = markers.findIndex(m =>
            m.getAttribute('data-slot-id') === state.activeEditMarker.id
        );
    }

    const nextIndex = (currentIndex + 1) % markers.length;
    const nextMarker = markers[nextIndex];

    const slotId = nextMarker.getAttribute('data-slot-id');
    const pathStr = nextMarker.getAttribute('data-path');
    const nodeId = nextMarker.getAttribute('data-node-id');
    const path = pathStr ? JSON.parse(pathStr.replace(/&quot;/g, '"')) : [];

    state.activeEditMarker = {id: slotId, path: path, nodeId: nodeId};

    document.querySelectorAll('.arg-overlay, .placeholder-overlay').forEach(el => {
        el.classList.remove('active-marker');
    });
    nextMarker.classList.add('active-marker');
    nextMarker.scrollIntoView({ behavior: 'smooth', block: 'center' });

    showStatus(`📍 Marker ${slotId} selected. Press Enter to edit, or click template to insert.`, 'info');
}

export function focusPrevMarker() {
    const markers = getAllMarkers();
    if (markers.length === 0) {
        showStatus('No markers available', 'error');
        return;
    }

    let currentIndex = -1;
    if (state.activeEditMarker) {
        currentIndex = markers.findIndex(m =>
            m.getAttribute('data-slot-id') === state.activeEditMarker.id
        );
    }

    const prevIndex = currentIndex <= 0 ? markers.length - 1 : currentIndex - 1;
    const prevMarker = markers[prevIndex];

    const slotId = prevMarker.getAttribute('data-slot-id');
    const pathStr = prevMarker.getAttribute('data-path');
    const nodeId = prevMarker.getAttribute('data-node-id');
    const path = pathStr ? JSON.parse(pathStr.replace(/&quot;/g, '"')) : [];

    state.activeEditMarker = {id: slotId, path: path, nodeId: nodeId};

    document.querySelectorAll('.arg-overlay, .placeholder-overlay').forEach(el => {
        el.classList.remove('active-marker');
    });
    prevMarker.classList.add('active-marker');
    prevMarker.scrollIntoView({ behavior: 'smooth', block: 'center' });

    showStatus(`📍 Marker ${slotId} selected. Press Enter to edit, or click template to insert.`, 'info');
}

export function editActiveMarker() {
    if (!state.activeEditMarker) {
        showStatus('No marker selected', 'error');
        return;
    }

    const node = getNodeAtPath(state.currentAST, state.activeEditMarker.path);
    let val = "";
    if (node.Const) val = node.Const;
    if (node.Object) val = node.Object;

    const input = prompt("Enter value:", val);
    if (input !== null) {
        saveToUndoStack();
        const newNode = parseSimpleInput(input);
        setNodeAtPath(state.currentAST, state.activeEditMarker.path, newNode);
        state.activeEditMarker = null;
        renderStructuralEditor();
    }
}

export function clearActiveMarker() {
    state.activeEditMarker = null;
    document.querySelectorAll('.arg-overlay, .placeholder-overlay').forEach(el => {
        el.classList.remove('active-marker');
    });
    showStatus('Selection cleared', 'info');
}
