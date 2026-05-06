import { state, API_BASE, templateMap, astTemplates } from './state.js';
import { showStatus, setNodeAtPath, renumberPlaceholders, findFirstPlaceholderPath, getNodeAtPath } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { saveToUndoStack } from './undoRedo.js';
import { isInlineEditorActive, hideInlineEditor } from './inlineEdit.js';

export async function showEgyptianPalette(btn) {
    document.querySelectorAll('[id^="palette-"]').forEach(p => p.style.display = 'none');
    const palette = document.getElementById('palette-egyptian');
    palette.style.display = 'grid';

    document.querySelectorAll('.palette-tab').forEach(t => t.classList.remove('active'));
    if (btn) btn.classList.add('active');

    if (state.egyptianTemplatesCache) return;

    try {
        const res = await fetch(`${API_BASE}/templates`);
        const templates = await res.json();

        const egyptian = templates.filter(t =>
            t.category && t.category.startsWith('egyptian_')
        );

        if (egyptian.length === 0) {
            palette.innerHTML = '<div style="grid-column:1/-1;text-align:center;color:#888;padding:12px;">No Egyptian templates found. Start the server with std_template_lib/ available.</div>';
            return;
        }

        const compositions = [];
        const glyphs = [];
        for (const t of egyptian) {
            if (t.category === 'egyptian_composition') {
                compositions.push(t);
            } else {
                glyphs.push(t);
            }
        }

        let html = '';

        if (compositions.length > 0) {
            html += `<div style="grid-column:1/-1;font-weight:600;font-size:12px;color:#666;padding:6px 4px 2px;border-bottom:1px solid #eee;margin-bottom:4px;">Composition (quadrats)</div>`;
            const sq = (x,y,w,h) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="2" fill="#f0f0f0" stroke="#999" stroke-width="1.5"/>`;
            const compLabels = {
                'quadrat_h': {
                    tip: 'Horizontal pair (left + right)',
                    label: `<svg width="40" height="40" viewBox="0 0 40 40">${sq(1,10,17,20)}${sq(22,10,17,20)}</svg>`
                },
                'quadrat_v': {
                    tip: 'Vertical stack (top / bottom)',
                    label: `<svg width="40" height="40" viewBox="0 0 40 40">${sq(8,1,24,17)}${sq(8,22,24,17)}</svg>`
                },
            };
            for (const t of compositions) {
                const info = compLabels[t.name] || { label: t.glyph || t.name, tip: t.name };
                templateMap[`egyptian:${t.name}`] = t.name;
                html += `<button class="math-btn" onclick="insertTemplate('egyptian:${t.name}')" data-tooltip="${info.tip}">${info.label}</button>`;
            }
        }

        html += `<div style="grid-column:1/-1;display:flex;gap:8px;align-items:center;padding:6px 4px;flex-wrap:wrap;">` +
            `<label style="font-size:12px;color:#666;">Filter:</label>` +
            `<select id="egyptian-filter-shape" onchange="filterEgyptianGlyphs()" style="font-size:12px;padding:2px 6px;border:1px solid #e0e0e0;border-radius:4px;">` +
            `<option value="">All shapes</option>` +
            `<option value="Tall">Tall</option>` +
            `<option value="Flat">Flat</option>` +
            `<option value="Small">Small</option>` +
            `</select>` +
            `<select id="egyptian-filter-type" onchange="filterEgyptianGlyphs()" style="font-size:12px;padding:2px 6px;border:1px solid #e0e0e0;border-radius:4px;">` +
            `<option value="">All types</option>` +
            `<option value="Uni">Uniliteral</option>` +
            `<option value="Bi">Biliteral</option>` +
            `<option value="Tri">Triliteral</option>` +
            `<option value="Det">Determinative</option>` +
            `</select>` +
            `<span id="egyptian-filter-count" style="font-size:11px;color:#999;margin-left:auto;"></span>` +
            `</div>`;

        const groups = {};
        for (const t of glyphs) {
            const cat = t.category.replace('egyptian_', '').replace(/_/g, ' ');
            const label = cat.charAt(0).toUpperCase() + cat.slice(1);
            if (!groups[label]) groups[label] = [];
            groups[label].push(t);
        }

        const sortedLabels = Object.keys(groups).sort();
        for (const label of sortedLabels) {
            html += `<div class="egyptian-group-header" data-group="${label}" style="grid-column:1/-1;font-weight:600;font-size:12px;color:#666;padding:6px 4px 2px;border-bottom:1px solid #eee;margin-top:4px;">${label}</div>`;
            for (const t of groups[label]) {
                const shape = (t.metadata && t.metadata.sign_shape) || '';
                const signType = (t.metadata && t.metadata.sign_type) || '';
                const sound = (t.metadata && t.metadata.sound) || '';
                const tooltip = `${t.name}` +
                    (shape ? ` [${shape}]` : '') +
                    (signType ? ` ${signType}` : '') +
                    (sound ? ` — ${sound}` : '');

                if (t.svg) {
                    html += `<button class="math-btn egyptian-glyph" data-group="${label}" onclick="insertTemplate('egyptian:${t.name}')" data-tooltip="${tooltip}" data-sign-shape="${shape}" data-sign-type="${signType}"><img src="/${t.svg}" alt="${t.name}" style="height:32px;width:32px;object-fit:contain;" onerror="this.outerHTML='<span>${t.name}</span>'"></button>`;
                } else {
                    html += `<button class="math-btn egyptian-glyph" data-group="${label}" onclick="insertTemplate('egyptian:${t.name}')" data-tooltip="${tooltip}" data-sign-shape="${shape}" data-sign-type="${signType}">${t.glyph || t.name}</button>`;
                }
            }
        }

        palette.innerHTML = html;
        state.egyptianTemplatesCache = egyptian;

        for (const t of glyphs) {
            const key = `egyptian:${t.name}`;
            templateMap[key] = `eg_${t.name}`;
            astTemplates[`eg_${t.name}`] = { Operation: { name: t.name, args: [] } };
        }

        filterEgyptianGlyphs();
    } catch (e) {
        palette.innerHTML = `<div style="grid-column:1/-1;text-align:center;color:#c00;padding:12px;">Failed to load templates: ${e.message}</div>`;
    }
}

export function filterEgyptianGlyphs() {
    const shapeFilter = document.getElementById('egyptian-filter-shape')?.value || '';
    const typeFilter = document.getElementById('egyptian-filter-type')?.value || '';
    const palette = document.getElementById('palette-egyptian');
    if (!palette) return;

    const glyphBtns = palette.querySelectorAll('.egyptian-glyph');
    const groupHeaders = palette.querySelectorAll('.egyptian-group-header');
    const visibleGroups = new Set();
    let visibleCount = 0;

    glyphBtns.forEach(btn => {
        const shape = btn.getAttribute('data-sign-shape') || '';
        const type = btn.getAttribute('data-sign-type') || '';
        const matchShape = !shapeFilter || shape === shapeFilter;
        const matchType = !typeFilter || type === typeFilter;
        if (matchShape && matchType) {
            btn.style.display = '';
            visibleGroups.add(btn.getAttribute('data-group'));
            visibleCount++;
        } else {
            btn.style.display = 'none';
        }
    });

    groupHeaders.forEach(hdr => {
        hdr.style.display = visibleGroups.has(hdr.getAttribute('data-group')) ? '' : 'none';
    });

    const countEl = document.getElementById('egyptian-filter-count');
    if (countEl) {
        countEl.textContent = shapeFilter || typeFilter
            ? `${visibleCount} of ${glyphBtns.length} glyphs`
            : `${glyphBtns.length} glyphs`;
    }
}

function getGlyphShape(glyphName) {
    if (!state.egyptianTemplatesCache) return null;
    const t = state.egyptianTemplatesCache.find(t => t.name === glyphName);
    return (t && t.metadata && t.metadata.sign_shape) || null;
}

function getNodeGlyphName(node) {
    if (node && node.Operation && node.Operation.args && node.Operation.args.length === 0) {
        return node.Operation.name;
    }
    return null;
}

function validateQuadratPlacement(ast, path, newGlyphName) {
    if (path.length < 1) return null;
    const parentPath = path.slice(0, -1);
    const slotIndex = path[path.length - 1];

    const parent = parentPath.length === 0 ? ast : getNodeAtPath(ast, parentPath);
    if (!parent || !parent.Operation) return null;

    const quadratName = parent.Operation.name;
    if (!quadratName.startsWith('quadrat_')) return null;

    const newShape = getGlyphShape(newGlyphName);
    if (!newShape) return null;

    const args = parent.Operation.args;

    if (quadratName === 'quadrat_h') {
        if (newShape === 'Tall') {
            return `${newGlyphName} is Tall — horizontal pairs need Flat or Small signs`;
        }
    } else if (quadratName === 'quadrat_v') {
        const otherIndex = slotIndex === 0 ? 1 : 0;
        const otherName = getNodeGlyphName(args[otherIndex]);
        const otherShape = otherName ? getGlyphShape(otherName) : null;
        if (newShape === 'Tall' && otherShape === 'Tall') {
            return `Both signs are Tall — vertical stack would be illegible`;
        }
    }

    return null;
}

export function insertEgyptianGlyph(name) {
    console.log('[insertEgyptianGlyph] name:', name, 'mode:', state.editorMode, 'marker:', state.activeEditMarker);

    if (state.editorMode === 'structural') {
        if (isInlineEditorActive()) {
            console.log('[insertEgyptianGlyph] hiding inline editor');
            hideInlineEditor(false);
        }

        const compositionTemplate = astTemplates[name];
        if (compositionTemplate) {
            console.log('[insertEgyptianGlyph] composition template found');
            saveToUndoStack();
            let ast = JSON.parse(JSON.stringify(compositionTemplate));
            renumberPlaceholders(ast);

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
            showStatus(`Inserted ${name} composition`, 'success');
            return;
        }

        const glyphNode = { Operation: { name: name, args: [] } };

        if (state.activeEditMarker) {
            const warning = validateQuadratPlacement(state.currentAST, state.activeEditMarker.path, name);
            if (warning) {
                showStatus(`Blocked: ${warning}`, 'error');
                return;
            }

            saveToUndoStack();
            setNodeAtPath(state.currentAST, state.activeEditMarker.path, glyphNode);
            state.activeEditMarker = null;
            document.querySelectorAll('.arg-overlay').forEach(el => {
                el.classList.remove('active-marker');
            });
            renderStructuralEditor();
            showStatus(`Inserted ${name}`, 'success');
        } else {
            const emptyPath = findFirstPlaceholderPath(state.currentAST, []);
            if (emptyPath) {
                saveToUndoStack();
                setNodeAtPath(state.currentAST, emptyPath, glyphNode);
                renderStructuralEditor();
                showStatus(`Inserted ${name}`, 'success');
            } else {
                saveToUndoStack();
                state.currentAST = glyphNode;
                renderStructuralEditor();
            }
        }
    } else {
        const textarea = document.getElementById('latexInput');
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const text = textarea.value;
        textarea.value = text.substring(0, start) + name + '()' + text.substring(end);
        textarea.setSelectionRange(start + name.length + 2, start + name.length + 2);
        textarea.focus();
    }
}

export function showPalette(name, btn) {
    document.querySelectorAll('[id^="palette-"]').forEach(p => p.style.display = 'none');
    const palette = document.getElementById(`palette-${name}`);
    if (palette) palette.style.display = 'grid';

    document.querySelectorAll('.palette-tab').forEach(t => t.classList.remove('active'));
    if (btn) btn.classList.add('active');
}
