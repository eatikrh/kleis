import { state, API_BASE, templateMap, astTemplates } from './state.js';
import { showStatus, setNodeAtPath, renumberPlaceholders, findFirstPlaceholderPath, getNodeAtPath } from './astUtils.js';
import { renderStructuralEditor } from './render.js';
import { saveToUndoStack } from './undoRedo.js';
import { isInlineEditorActive, hideInlineEditor } from './inlineEdit.js';

const SKIP_DATA_ATTR_KEYS = new Set(['mode', 'sound']);

const DOMAIN_CONFIGS = {
    egyptian: {
        displayName: 'Egyptian',
        itemLabel: 'glyphs',
        compositionLabels: {
            'quadrat_h': {
                tip: 'Horizontal pair (left + right)',
                label: (() => {
                    const sq = (x,y,w,h) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="2" fill="#f0f0f0" stroke="#999" stroke-width="1.5"/>`;
                    return `<svg width="40" height="40" viewBox="0 0 40 40">${sq(1,10,17,20)}${sq(22,10,17,20)}</svg>`;
                })()
            },
            'quadrat_v': {
                tip: 'Vertical stack (top / bottom)',
                label: (() => {
                    const sq = (x,y,w,h) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="2" fill="#f0f0f0" stroke="#999" stroke-width="1.5"/>`;
                    return `<svg width="40" height="40" viewBox="0 0 40 40">${sq(8,1,24,17)}${sq(8,22,24,17)}</svg>`;
                })()
            },
        },
        filterLabels: {
            sign_shape: 'Shape',
            sign_type: 'Type',
        },
        filterValueLabels: {
            sign_type: { 'Uni': 'Uniliteral', 'Bi': 'Biliteral', 'Tri': 'Triliteral', 'Det': 'Determinative' },
        },
        buildTooltip(t) {
            const shape = (t.metadata && t.metadata.sign_shape) || '';
            const signType = (t.metadata && t.metadata.sign_type) || '';
            const sound = (t.metadata && t.metadata.sound) || '';
            return `${t.name}` +
                (shape ? ` [${shape}]` : '') +
                (signType ? ` ${signType}` : '') +
                (sound ? ` — ${sound}` : '');
        },
        validation: validateQuadratPlacement,
    },
    electronics: {
        displayName: 'Electronics',
        compositionLabels: {
            'series': {
                tip: 'Series connection (left — right)',
                label: (() => {
                    const r = (x,y,w,h) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="1" fill="#f0f0f0" stroke="#999" stroke-width="1.5"/>`;
                    return `<svg width="40" height="40" viewBox="0 0 40 40">${r(1,14,14,12)}<line x1="15" y1="20" x2="25" y2="20" stroke="#999" stroke-width="1.5"/>${r(25,14,14,12)}</svg>`;
                })()
            },
            'parallel': {
                tip: 'Parallel connection (top ‖ bottom)',
                label: (() => {
                    const r = (x,y,w,h) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="1" fill="#f0f0f0" stroke="#999" stroke-width="1.5"/>`;
                    return `<svg width="40" height="40" viewBox="0 0 40 40">${r(8,1,24,15)}${r(8,24,24,15)}<line x1="4" y1="8" x2="4" y2="32" stroke="#999" stroke-width="1.5"/><line x1="36" y1="8" x2="36" y2="32" stroke="#999" stroke-width="1.5"/></svg>`;
                })()
            },
        },
        filterLabels: {
            component_type: 'Type',
            package: 'Package',
        },
    },
};

function discoverFilterKeys(templates) {
    const keyCounts = {};
    for (const t of templates) {
        if (!t.metadata) continue;
        for (const [key, val] of Object.entries(t.metadata)) {
            if (SKIP_DATA_ATTR_KEYS.has(key)) continue;
            if (!keyCounts[key]) keyCounts[key] = new Set();
            keyCounts[key].add(val);
        }
    }
    return Object.entries(keyCounts)
        .filter(([_, vals]) => vals.size >= 2)
        .map(([key, vals]) => ({ key, values: [...vals].sort() }))
        .sort((a, b) => a.key.localeCompare(b.key));
}

function formatFilterLabel(key, config) {
    if (config && config.filterLabels && config.filterLabels[key]) {
        return config.filterLabels[key];
    }
    return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

function formatFilterValue(key, val, config) {
    if (config && config.filterValueLabels && config.filterValueLabels[key] && config.filterValueLabels[key][val]) {
        return config.filterValueLabels[key][val];
    }
    return val;
}

function buildDefaultTooltip(t) {
    let tooltip = t.name;
    if (t.metadata) {
        for (const [key, val] of Object.entries(t.metadata)) {
            if (key === 'mode') continue;
            tooltip += ` [${val}]`;
        }
    }
    return tooltip;
}

export async function showDomainPalette(domain, btn) {
    document.querySelectorAll('[id^="palette-"]').forEach(p => p.style.display = 'none');
    const palette = document.getElementById(`palette-${domain}`);
    if (!palette) return;
    palette.style.display = 'grid';

    document.querySelectorAll('.palette-tab').forEach(t => t.classList.remove('active'));
    if (btn) btn.classList.add('active');

    if (state.domainTemplatesCache[domain]) return;

    const config = DOMAIN_CONFIGS[domain] || {};
    const prefix = domain + '_';

    try {
        const res = await fetch(`${API_BASE}/templates`);
        const allTemplates = await res.json();

        const domainTemplates = allTemplates.filter(t =>
            t.category && t.category.startsWith(prefix)
        );

        if (domainTemplates.length === 0) {
            palette.innerHTML = `<div style="grid-column:1/-1;text-align:center;color:#888;padding:12px;">No ${config.displayName || domain} templates found. Start the server with std_template_lib/ available.</div>`;
            return;
        }

        const compositions = [];
        const atoms = [];
        for (const t of domainTemplates) {
            if (t.category === `${domain}_composition`) {
                compositions.push(t);
            } else {
                atoms.push(t);
            }
        }

        let html = '';

        if (compositions.length > 0) {
            html += `<div style="grid-column:1/-1;font-weight:600;font-size:12px;color:#666;padding:6px 4px 2px;border-bottom:1px solid #eee;margin-bottom:4px;">Composition</div>`;
            const compLabels = config.compositionLabels || {};
            for (const t of compositions) {
                const info = compLabels[t.name] || { label: t.glyph || t.name, tip: t.name };
                templateMap[`${domain}:${t.name}`] = t.name;
                html += `<button class="math-btn" onclick="insertTemplate('${domain}:${t.name}')" data-tooltip="${info.tip}">${info.label}</button>`;
            }
        }

        const filterKeys = discoverFilterKeys(atoms);
        if (filterKeys.length > 0) {
            html += `<div style="grid-column:1/-1;display:flex;gap:8px;align-items:center;padding:6px 4px;flex-wrap:wrap;">`;
            html += `<label style="font-size:12px;color:#666;">Filter:</label>`;
            for (const fk of filterKeys) {
                const label = formatFilterLabel(fk.key, config);
                html += `<select id="${domain}-filter-${fk.key}" onchange="filterDomainGlyphs('${domain}')" style="font-size:12px;padding:2px 6px;border:1px solid #e0e0e0;border-radius:4px;">`;
                html += `<option value="">All ${label.toLowerCase()}s</option>`;
                for (const val of fk.values) {
                    html += `<option value="${val}">${formatFilterValue(fk.key, val, config)}</option>`;
                }
                html += `</select>`;
            }
            html += `<span id="${domain}-filter-count" style="font-size:11px;color:#999;margin-left:auto;"></span>`;
            html += `</div>`;
        }

        const groups = {};
        for (const t of atoms) {
            const cat = t.category.replace(prefix, '').replace(/_/g, ' ');
            const label = cat.charAt(0).toUpperCase() + cat.slice(1);
            if (!groups[label]) groups[label] = [];
            groups[label].push(t);
        }

        const sortedLabels = Object.keys(groups).sort();
        for (const label of sortedLabels) {
            html += `<div class="domain-group-header" data-domain="${domain}" data-group="${label}" style="grid-column:1/-1;font-weight:600;font-size:12px;color:#666;padding:6px 4px 2px;border-bottom:1px solid #eee;margin-top:4px;">${label}</div>`;
            for (const t of groups[label]) {
                const dataAttrs = [];
                dataAttrs.push(`data-domain="${domain}"`);
                dataAttrs.push(`data-group="${label}"`);

                if (t.metadata) {
                    for (const [key, val] of Object.entries(t.metadata)) {
                        if (!SKIP_DATA_ATTR_KEYS.has(key)) {
                            dataAttrs.push(`data-${key.replace(/_/g, '-')}="${val}"`);
                        }
                    }
                }

                const tooltip = config.buildTooltip
                    ? config.buildTooltip(t)
                    : buildDefaultTooltip(t);

                if (t.svg) {
                    html += `<button class="math-btn domain-glyph" ${dataAttrs.join(' ')} onclick="insertDomainGlyph('${domain}', '${t.name}')" data-tooltip="${tooltip}"><img src="/${t.svg}" alt="${t.name}" style="height:32px;width:32px;object-fit:contain;" onerror="this.outerHTML='<span>${t.name}</span>'"></button>`;
                } else {
                    html += `<button class="math-btn domain-glyph" ${dataAttrs.join(' ')} onclick="insertDomainGlyph('${domain}', '${t.name}')" data-tooltip="${tooltip}">${t.glyph || t.name}</button>`;
                }
            }
        }

        palette.innerHTML = html;
        state.domainTemplatesCache[domain] = { templates: domainTemplates, filterKeys };

        for (const t of atoms) {
            const key = `${domain}:${t.name}`;
            if (astTemplates[t.name]) {
                templateMap[key] = t.name;
            } else {
                templateMap[key] = `${domain}_${t.name}`;
                astTemplates[`${domain}_${t.name}`] = { Operation: { name: t.name, args: [] } };
            }
        }

        filterDomainGlyphs(domain);
    } catch (e) {
        palette.innerHTML = `<div style="grid-column:1/-1;text-align:center;color:#c00;padding:12px;">Failed to load templates: ${e.message}</div>`;
    }
}

export function filterDomainGlyphs(domain) {
    const cached = state.domainTemplatesCache[domain];
    if (!cached) return;

    const palette = document.getElementById(`palette-${domain}`);
    if (!palette) return;

    const activeFilters = {};
    for (const fk of cached.filterKeys) {
        const el = document.getElementById(`${domain}-filter-${fk.key}`);
        if (el && el.value) {
            activeFilters[fk.key] = el.value;
        }
    }

    const glyphBtns = palette.querySelectorAll('.domain-glyph');
    const groupHeaders = palette.querySelectorAll('.domain-group-header');
    const visibleGroups = new Set();
    let visibleCount = 0;
    const hasActiveFilter = Object.keys(activeFilters).length > 0;

    glyphBtns.forEach(btn => {
        let match = true;
        for (const [key, val] of Object.entries(activeFilters)) {
            const attrName = `data-${key.replace(/_/g, '-')}`;
            const btnVal = btn.getAttribute(attrName) || '';
            if (btnVal !== val) {
                match = false;
                break;
            }
        }
        if (match) {
            btn.style.display = '';
            visibleGroups.add(btn.getAttribute('data-group'));
            visibleCount++;
        } else {
            btn.style.display = 'none';
        }
    });

    groupHeaders.forEach(hdr => {
        if (hdr.getAttribute('data-domain') === domain) {
            hdr.style.display = visibleGroups.has(hdr.getAttribute('data-group')) ? '' : 'none';
        }
    });

    const countEl = document.getElementById(`${domain}-filter-count`);
    if (countEl) {
        const itemWord = (DOMAIN_CONFIGS[domain] && DOMAIN_CONFIGS[domain].itemLabel) || 'items';
        countEl.textContent = hasActiveFilter
            ? `${visibleCount} of ${glyphBtns.length} ${itemWord}`
            : `${glyphBtns.length} ${itemWord}`;
    }
}

export function insertDomainGlyph(domain, name) {
    const config = DOMAIN_CONFIGS[domain] || {};

    if (state.editorMode === 'structural') {
        if (isInlineEditorActive()) {
            hideInlineEditor(false);
        }

        const templateKey = `${domain}_${name}`;
        const compositionTemplate = astTemplates[templateKey] || astTemplates[name];
        if (compositionTemplate && compositionTemplate.Operation && compositionTemplate.Operation.args.length > 0) {
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
            if (config.validation) {
                const warning = config.validation(state.currentAST, state.activeEditMarker.path, name);
                if (warning) {
                    showStatus(`Blocked: ${warning}`, 'error');
                    return;
                }
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

function validateQuadratPlacement(ast, path, newGlyphName) {
    if (path.length < 1) return null;
    const parentPath = path.slice(0, -1);
    const slotIndex = path[path.length - 1];

    const parent = parentPath.length === 0 ? ast : getNodeAtPath(ast, parentPath);
    if (!parent || !parent.Operation) return null;

    const quadratName = parent.Operation.name;
    if (!quadratName.startsWith('quadrat_')) return null;

    const cached = state.domainTemplatesCache['egyptian'];
    if (!cached) return null;
    const tmpl = cached.templates.find(t => t.name === newGlyphName);
    const newShape = (tmpl && tmpl.metadata && tmpl.metadata.sign_shape) || null;
    if (!newShape) return null;

    const args = parent.Operation.args;

    if (quadratName === 'quadrat_h') {
        if (newShape === 'Tall') {
            return `${newGlyphName} is Tall — horizontal pairs need Flat or Small signs`;
        }
    } else if (quadratName === 'quadrat_v') {
        const otherIndex = slotIndex === 0 ? 1 : 0;
        const otherNode = args[otherIndex];
        const otherName = (otherNode && otherNode.Operation && otherNode.Operation.args && otherNode.Operation.args.length === 0)
            ? otherNode.Operation.name : null;
        if (otherName) {
            const otherTmpl = cached.templates.find(t => t.name === otherName);
            const otherShape = (otherTmpl && otherTmpl.metadata && otherTmpl.metadata.sign_shape) || null;
            if (newShape === 'Tall' && otherShape === 'Tall') {
                return `Both signs are Tall — vertical stack would be illegible`;
            }
        }
    }

    return null;
}

export function showPalette(name, btn) {
    document.querySelectorAll('[id^="palette-"]').forEach(p => p.style.display = 'none');
    const palette = document.getElementById(`palette-${name}`);
    if (palette) palette.style.display = 'grid';

    document.querySelectorAll('.palette-tab').forEach(t => t.classList.remove('active'));
    if (btn) btn.classList.add('active');
}

export async function initDomainPalettes() {
    try {
        const res = await fetch(`${API_BASE}/templates`);
        const templates = await res.json();

        const domainCategories = {};
        for (const t of templates) {
            if (t.category) {
                const idx = t.category.indexOf('_');
                if (idx > 0) {
                    const prefix = t.category.substring(0, idx);
                    if (!domainCategories[prefix]) domainCategories[prefix] = new Set();
                    domainCategories[prefix].add(t.category);
                }
            }
        }
        const domains = new Set(
            Object.entries(domainCategories)
                .filter(([_, cats]) => cats.size >= 2)
                .map(([prefix]) => prefix)
        );

        const tabsContainer = document.querySelector('.palette-tabs');
        if (!tabsContainer) return;

        const paletteContainer = document.querySelector('.symbol-palette');
        if (!paletteContainer) return;

        for (const domain of [...domains].sort()) {
            const config = DOMAIN_CONFIGS[domain] || {};
            const displayName = config.displayName || domain.charAt(0).toUpperCase() + domain.slice(1);

            const tab = document.createElement('button');
            tab.className = 'palette-tab';
            tab.textContent = displayName;
            tab.onclick = function() { showDomainPalette(domain, this); };
            tabsContainer.appendChild(tab);

            const paletteDiv = document.createElement('div');
            paletteDiv.id = `palette-${domain}`;
            paletteDiv.className = 'symbol-grid';
            paletteDiv.style.display = 'none';
            paletteDiv.innerHTML = '<div style="grid-column: 1 / -1; text-align: center; color: #888; padding: 12px;">Loading...</div>';
            paletteContainer.appendChild(paletteDiv);
        }
    } catch (e) {
        console.error('Failed to discover domain palettes:', e);
    }
}
