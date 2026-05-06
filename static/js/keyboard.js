import { state } from './state.js';
import { undo, redo } from './undoRedo.js';
import { zoomIn, zoomOut, zoomReset, focusNextMarker, focusPrevMarker, editActiveMarker, clearActiveMarker } from './slotHandlers.js';

export function initKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        if (state.editorMode === 'structural') {
            if (e.target.matches('input, textarea')) {
                return;
            }

            if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) {
                e.preventDefault();
                undo();
            } else if ((e.metaKey || e.ctrlKey) && (e.key === 'Z' || (e.key === 'z' && e.shiftKey))) {
                e.preventDefault();
                redo();
            }
            else if ((e.metaKey || e.ctrlKey) && (e.key === '=' || e.key === '+')) {
                e.preventDefault();
                zoomIn();
            } else if ((e.metaKey || e.ctrlKey) && e.key === '-') {
                e.preventDefault();
                zoomOut();
            } else if ((e.metaKey || e.ctrlKey) && e.key === '0') {
                e.preventDefault();
                zoomReset();
            }
            else if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
                e.preventDefault();
                focusNextMarker();
            } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
                e.preventDefault();
                focusPrevMarker();
            }
            else if (e.key === 'Tab') {
                e.preventDefault();
                if (e.shiftKey) {
                    focusPrevMarker();
                } else {
                    focusNextMarker();
                }
            }
            else if (e.key === 'Enter' && state.activeEditMarker) {
                e.preventDefault();
                editActiveMarker();
            }
            else if (e.key === 'Escape' && state.activeEditMarker) {
                e.preventDefault();
                clearActiveMarker();
            }
        }
    });
}
