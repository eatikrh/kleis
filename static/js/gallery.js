import { state, API_BASE } from './state.js';
import { showStatus } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { renderEquation } from './verify.js';

export async function loadGallery() {
    const galleryDiv = document.getElementById('gallery');
    galleryDiv.innerHTML = 'Loading...';
    try {
        const res = await fetch(`${API_BASE}/gallery`);
        const data = await res.json();
        galleryDiv.innerHTML = '';
        data.examples.forEach(ex => {
            const el = document.createElement('div');
            el.className = 'gallery-item';
            el.innerHTML = `<div class="gallery-title">${ex.title}</div><div>\\[${ex.latex}\\]</div>`;
            el.onclick = () => window.loadExample(ex.latex);
            galleryDiv.appendChild(el);
        });
        if (window.MathJax) MathJax.typesetPromise([galleryDiv]);
    } catch (e) {
        galleryDiv.innerHTML = 'Error loading gallery';
    }
}

export async function loadExample(latex) {
    const input = document.getElementById('latexInput');
    if (input) input.value = latex;

    if (state.editorMode === 'structural') {
        try {
            const container = document.getElementById('structuralEditor');
            container.innerHTML = '<div style="text-align:center">🔄 Parsing LaTeX...</div>';

            const response = await fetch(`${API_BASE}/parse`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ latex })
            });

            const data = await response.json();

            if (data.success && data.ast) {
                state.currentAST = data.ast;
                state.nextPlaceholderId = 10000;

                renderStructuralEditor();
                showStatus('✅ Loaded into Structural Editor', 'success');
            } else {
                showStatus('❌ Parse failed: ' + (data.error || 'Unknown error'), 'error');
                setTimeout(() => {
                    window.setEditorMode('text');
                    renderEquation();
                }, 1000);
            }
        } catch (e) {
            showStatus('❌ Network error: ' + e.message, 'error');
        }
    } else {
        renderEquation();
    }
}

export function clearInput() {
    const input = document.getElementById('latexInput');
    if (input) input.value = '';
    const preview = document.getElementById('preview');
    if (preview) preview.innerHTML = '';
}
