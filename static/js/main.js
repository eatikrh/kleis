import { state, isJupyterMode, astTemplates } from './state.js';
import { showStatus } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { undo, redo } from './undoRedo.js';
import { initializeInlineEditing } from './inlineEdit.js';
import { handleSlotClick, handleSlotKeydown, toggleBoundingBoxes, resetStructuralEditor,
         zoomIn, zoomOut, zoomReset } from './slotHandlers.js';
import { insertSymbol, insertTemplate } from './palette.js';
import { showEgyptianPalette, filterEgyptianGlyphs, insertEgyptianGlyph, showPalette } from './egyptian.js';
import { initializeMatrixBuilder, showMatrixBuilder, closeMatrixBuilder,
         createMatrixFromBuilder, insertMatrixFromPalette } from './matrixBuilder.js';
import { showPiecewiseBuilder, closePiecewiseBuilder, updatePiecewisePreview,
         createPiecewiseFromBuilder, initPiecewiseBuilder } from './piecewiseBuilder.js';
import { toggleDebugPanel, copyASTToClipboard, copyTypstToClipboard, downloadAST } from './debug.js';
import { renderEquation, verifyWithZ3, checkSatisfiable } from './verify.js';
import { setEditorMode } from './modeConvert.js';
import { sendToJupyter, initJupyter } from './jupyter.js';
import { initKeyboardShortcuts } from './keyboard.js';
import { loadGallery, loadExample, clearInput } from './gallery.js';

// Wire all functions that HTML onclick/onchange/oninput attributes reference
window.insertSymbol = insertSymbol;
window.insertTemplate = insertTemplate;
window.undo = undo;
window.redo = redo;
window.zoomIn = zoomIn;
window.zoomOut = zoomOut;
window.zoomReset = zoomReset;
window.handleSlotClick = handleSlotClick;
window.handleSlotKeydown = handleSlotKeydown;
window.toggleBoundingBoxes = toggleBoundingBoxes;
window.resetStructuralEditor = resetStructuralEditor;
window.setEditorMode = setEditorMode;
window.showPalette = showPalette;
window.showEgyptianPalette = showEgyptianPalette;
window.filterEgyptianGlyphs = filterEgyptianGlyphs;
window.insertEgyptianGlyph = insertEgyptianGlyph;
window.loadExample = loadExample;
window.loadGallery = loadGallery;
window.clearInput = clearInput;
window.sendToJupyter = sendToJupyter;
window.renderEquation = renderEquation;
window.verifyWithZ3 = verifyWithZ3;
window.checkSatisfiable = checkSatisfiable;
window.toggleDebugPanel = toggleDebugPanel;
window.copyASTToClipboard = copyASTToClipboard;
window.copyTypstToClipboard = copyTypstToClipboard;
window.downloadAST = downloadAST;
window.showMatrixBuilder = showMatrixBuilder;
window.closeMatrixBuilder = closeMatrixBuilder;
window.createMatrixFromBuilder = createMatrixFromBuilder;
window.insertMatrixFromPalette = insertMatrixFromPalette;
window.showPiecewiseBuilder = showPiecewiseBuilder;
window.closePiecewiseBuilder = closePiecewiseBuilder;
window.updatePiecewisePreview = updatePiecewisePreview;
window.createPiecewiseFromBuilder = createPiecewiseFromBuilder;

// Initialize subsystems
initJupyter();
initKeyboardShortcuts();
initPiecewiseBuilder();

window.addEventListener('load', () => {
    initializeMatrixBuilder();
    loadGallery();

    const templateCount = Object.keys(astTemplates).length;
    console.log(`✅ Kleis Editor v2.1 (ES modules) loaded with ${templateCount} AST templates`);

    if (templateCount < 50) {
        console.error(`⚠️ WARNING: Only ${templateCount} templates loaded! Expected 55. Browser cache issue!`);
        alert(`⚠️ OLD VERSION LOADED!\n\nOnly ${templateCount} templates found.\nExpected: 55 templates\n\nPlease:\n1. Close this tab\n2. Open in incognito mode (Cmd+Shift+N)\n3. Or clear browser cache completely`);
    } else {
        console.log('✅ All templates loaded correctly');
        console.log('sqrt template:', astTemplates.sqrt);
    }

    if (isJupyterMode) {
        const jupyterBtn = document.getElementById('jupyterSendBtn');
        if (jupyterBtn) {
            jupyterBtn.style.display = 'inline-block';
        }
        setEditorMode('structural');
        console.log('🟢 Jupyter mode: Send button enabled, structural mode activated');
    }
});

if (window.MathJax) {
    MathJax.typesetPromise(document.querySelectorAll('.math-btn'))
        .then(() => {
            console.log('✓ Palette buttons rendered with MathJax');
            initializeInlineEditing();
        })
        .catch(err => console.error('MathJax rendering failed:', err));
} else {
    initializeInlineEditing();
}
