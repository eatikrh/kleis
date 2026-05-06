import { state, API_BASE, COORDINATE_PREFERENCE } from './state.js';
import { showStatus, nodeIdFromPath } from './astUtils.js';
import { updateDebugPanel } from './debug.js';
import { checkTypesDebounced } from './verify.js';

export async function renderStructuralEditor() {
    const container = document.getElementById('structuralEditor');
    const preview = document.getElementById('preview');

    if (!state.currentAST) {
        container.innerHTML = '<span style="color: #999;">Click a template button to start building...</span>';
        preview.innerHTML = 'Structural mode active';
        return;
    }

    container.innerHTML = '<div style="text-align:center">🔄 Rendering...</div>';

    try {
        console.log('renderStructuralEditor: sending AST to /api/render_typst', state.currentAST);
        const response = await fetch(`${API_BASE}/render_typst`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ast: state.currentAST })
        });
        console.log('renderStructuralEditor: response status', response.status);

        const data = await response.json();
        console.log('renderStructuralEditor: response payload', data);

        state.lastRenderResponse = data;

        if (data.success) {
            let svg = data.svg;

            if (data.argument_slots && data.argument_slots.length > 0) {
                let overlayElements = [];

                let globalOffsetX = 0;
                let globalOffsetY = 0;
                const svgMatch = /<g[^>]*transform="translate\(([\d.-]+)[ ,]+([\d.-]+)\)"/.exec(svg);
                if (svgMatch) {
                    globalOffsetX = parseFloat(svgMatch[1]);
                    globalOffsetY = parseFloat(svgMatch[2]);
                }

                const isMatrix = state.currentAST.Operation &&
                               (state.currentAST.Operation.name.startsWith('matrix') ||
                                state.currentAST.Operation.name.startsWith('pmatrix') ||
                                state.currentAST.Operation.name.startsWith('vmatrix'));

                const parentNodeIds = new Set();
                data.argument_slots.forEach(slot => {
                    const nid = nodeIdFromPath(slot.path || []);
                    data.argument_slots.forEach(otherSlot => {
                        const otherNid = nodeIdFromPath(otherSlot.path || []);
                        if (otherNid.startsWith(nid + '.')) {
                            parentNodeIds.add(nid);
                        }
                    });
                });
                console.log(`Identified ${parentNodeIds.size} parent nodes (will hide their markers)`);

                data.argument_slots.forEach((slot, index) => {
                    const nid = nodeIdFromPath(slot.path || []);
                    if (parentNodeIds.has(nid)) {
                        console.log(`  Skipping parent node ${nid} (has children)`);
                        return;
                    }
                    let rectX, rectY, rectWidth, rectHeight;
                    let foundPosition = false;
                    let currentNodeId = null;
                    const role = slot.role || null;

                    if (COORDINATE_PREFERENCE === 'semantic') {
                        const nodePathId = nodeIdFromPath(slot.path || []);
                        const bbox = data.argument_bounding_boxes &&
                                    data.argument_bounding_boxes.find(b => b.node_id === nodePathId);

                        if (bbox) {
                            rectX = bbox.x - 3;
                            rectY = bbox.y - 3;
                            rectWidth = bbox.width + 6;
                            rectHeight = bbox.height + 6;
                            foundPosition = true;
                            currentNodeId = bbox.node_id;
                            console.log(`✅ Slot ${slot.id}: Using semantic bbox (x=${bbox.x.toFixed(1)}, y=${bbox.y.toFixed(1)}) node=${nodePathId}`);
                        } else {
                            const ph = data.placeholders && data.placeholders.find(p => p.id === slot.id);
                            if (ph) {
                                rectX = ph.x - 3;
                                rectY = ph.y - 3;
                                rectWidth = ph.width + 6;
                                rectHeight = ph.height + 6;
                                foundPosition = true;
                                console.log(`⚠️ Slot ${slot.id}: Fallback to placeholder (x=${ph.x.toFixed(1)}, y=${ph.y.toFixed(1)})`);
                            } else {
                                console.error(`❌ Slot ${slot.id}: No position found!`);
                            }
                        }
                    } else {
                        let searchId = null;
                        if (slot.is_placeholder && typeof slot.id === 'string' && slot.id.startsWith('ph')) {
                            searchId = parseInt(slot.id.substring(2));
                        } else if (typeof slot.id === 'number') {
                            searchId = slot.id;
                        }

                        const ph = searchId !== null && data.placeholders && data.placeholders.find(p => p.id === searchId);
                        if (ph) {
                            rectX = ph.x - 3;
                            rectY = ph.y - 3;
                            rectWidth = ph.width + 6;
                            rectHeight = ph.height + 6;
                            foundPosition = true;
                            console.log(`✅ Slot ${slot.id}: Using placeholder (x=${ph.x.toFixed(1)}, y=${ph.y.toFixed(1)})`);
                        } else {
                            const nodePathId = nodeIdFromPath(slot.path || []);
                            const bbox = data.argument_bounding_boxes &&
                                        data.argument_bounding_boxes.find(b => b.node_id === nodePathId);

                            if (bbox) {
                                const padding = 5;
                                rectX = bbox.x - padding;
                                rectY = bbox.y - padding;
                                rectWidth = bbox.width + (padding * 2);
                                rectHeight = bbox.height + (padding * 2);
                                foundPosition = true;
                                currentNodeId = bbox.node_id;
                                console.log(`⚠️ Slot ${slot.id}: Using semantic bbox (x=${bbox.x.toFixed(1)}, y=${bbox.y.toFixed(1)}) node=${nodePathId}`);
                            } else {
                                if (data.placeholders && data.placeholders.length >= 2) {
                                    const xCoords = [...new Set(data.placeholders.map(p => Math.round(p.x / 10) * 10))].sort((a,b) => a-b);
                                    const yCoords = [...new Set(data.placeholders.map(p => Math.round(p.y / 10) * 10))].sort((a,b) => a-b);

                                    if (xCoords.length > 0 && yCoords.length > 0) {
                                        const pathIdx = slot.path && slot.path.length > 0 ? slot.path[slot.path.length - 1] : index;
                                        const cols = xCoords.length;
                                        const rowIdx = Math.floor(pathIdx / cols);
                                        const colIdx = pathIdx % cols;

                                        if (colIdx < xCoords.length && rowIdx < yCoords.length) {
                                            const nearestPh = data.placeholders.find(p =>
                                                Math.abs(Math.round(p.x / 10) * 10 - xCoords[colIdx]) < 5 &&
                                                Math.abs(Math.round(p.y / 10) * 10 - yCoords[rowIdx]) < 5
                                            );

                                            if (nearestPh) {
                                                rectX = nearestPh.x - 3;
                                                rectY = nearestPh.y - 3;
                                            } else {
                                                rectX = xCoords[colIdx] - 3;
                                                rectY = yCoords[rowIdx] - 3;
                                            }
                                            rectWidth = 24;
                                            rectHeight = 24;
                                            foundPosition = true;
                                            console.log(`🔧 Slot ${slot.id}: Inferred from grid (row=${rowIdx}, col=${colIdx})`);
                                        }
                                    }
                                }

                                if (!foundPosition) {
                                    console.error(`❌ Slot ${slot.id}: No position found!`);
                                }
                            }
                        }
                    }

                    if (isMatrix && (!foundPosition || (foundPosition && index >= 2))) {
                        const isMatrix2x2 = state.currentAST.Operation.name.includes('2x2');
                        const isMatrix3x3 = state.currentAST.Operation.name.includes('3x3');
                        const cols = isMatrix2x2 ? 2 : (isMatrix3x3 ? 3 : 2);

                        const firstArg = data.argument_slots[0];
                        const hasNestedOps = firstArg && firstArg.path && firstArg.path.length > 1;

                        if (hasNestedOps) {
                            const knownGoodBoxes = data.argument_bounding_boxes
                                .filter(b => b.node_id.match(/^0\.\d+\.0$/));

                            if (knownGoodBoxes.length >= 2) {
                                const cellIndex = slot.path[0];
                                const row = Math.floor(cellIndex / cols);
                                const col = cellIndex % cols;

                                const anchor = knownGoodBoxes[0];
                                const colSpacing = knownGoodBoxes.length > 1 ?
                                                 knownGoodBoxes[1].x - knownGoodBoxes[0].x : 43;
                                const rowSpacing = 28.7;

                                rectX = anchor.x + (col * colSpacing);
                                rectY = anchor.y + (row * rowSpacing);
                                rectWidth = anchor.width;
                                rectHeight = anchor.height;

                                if (role === 'subscript') {
                                    rectX += 13;
                                    rectY += 6;
                                    rectWidth *= 0.5;
                                    rectHeight *= 0.5;
                                }

                                foundPosition = true;
                                console.log(`🔧 Slot ${slot.id}: Matrix grid inference (row=${row}, col=${col}, x=${rectX.toFixed(1)}, y=${rectY.toFixed(1)})`);
                            }
                        }
                    }

                    if (foundPosition) {
                        const widthFactor = role === 'base' ? 0.6 : 0.5;
                        const heightFactor = role === 'base' ? 0.65 : 0.5;

                        const originalWidth = rectWidth;
                        const originalHeight = rectHeight;

                        rectWidth = Math.max(6, rectWidth * widthFactor);
                        rectHeight = Math.max(6, rectHeight * heightFactor);

                        const centerShiftX = (originalWidth - rectWidth) / 2;
                        let centerShiftY = (originalHeight - rectHeight) / 2;

                        rectX += centerShiftX;
                        rectY += centerShiftY;

                        if (role === 'superscript') {
                            const shift = Math.max(4, rectHeight * 0.4);
                            rectY -= shift;
                            rectHeight = Math.max(6, rectHeight * 0.8);
                        } else if (role === 'subscript') {
                            const shift = Math.max(4, rectHeight * 0.4);
                            rectY += shift;
                            rectHeight = Math.max(6, rectHeight * 0.8);
                        } else if (role === 'base') {
                            const superscriptShift = Math.max(4, rectHeight * 0.4);
                            rectY -= superscriptShift;
                            rectHeight = Math.max(6, rectHeight + superscriptShift);
                        }
                        const color = slot.is_placeholder ? '#2c3e50' : '#28a745';
                        const fillColor = slot.is_placeholder ? 'rgba(240, 244, 255, 0.3)' : 'rgba(40, 167, 69, 0.2)';

                        const pathStr = JSON.stringify(slot.path).replace(/"/g, '&quot;');
                        const nodeId = currentNodeId || nodeIdFromPath(slot.path || []);
                        const rect = `<rect x="${rectX}" y="${rectY}" width="${rectWidth}" height="${rectHeight}"
                            fill="${fillColor}" stroke="${color}" stroke-width="2" stroke-dasharray="6,3" rx="3"
                            class="arg-overlay" data-slot-id="${slot.id}" data-path="${pathStr}" data-node-id="${nodeId}"
                            style="cursor: pointer;"
                            tabindex="0" focusable="true"
                            onclick="handleSlotClick(event, '${slot.id}', ${pathStr}, '${nodeId}')"
                            onkeydown="handleSlotKeydown(event, '${slot.id}', ${pathStr}, '${nodeId}')" />`;
                        overlayElements.push(rect);
                    }
                });

                if (overlayElements.length > 0) {
                    console.log(`Creating ${overlayElements.length} overlay elements`);
                    svg = svg.replace('</svg>', `<g id="arg-overlays" visibility="visible">${overlayElements.join('')}</g></svg>`);
                    console.log('Overlays injected into SVG');
                } else {
                    console.warn('No overlay elements created!');
                }
            }

            console.log('Setting container.innerHTML with SVG');

            const existingForeignObject = container.querySelector('#inline-editor-foreign');
            console.log('Existing foreignObject before render:', existingForeignObject);

            container.innerHTML = svg;
            preview.innerHTML = svg;
            console.log('SVG rendered to DOM');

            if (existingForeignObject) {
                const newSvg = container.querySelector('svg');
                if (newSvg) {
                    newSvg.appendChild(existingForeignObject);
                    console.log('ForeignObject restored after render');
                }
            }

            state.currentZoom = 1.0;
            const svgElement = container.querySelector('svg');
            if (svgElement) {
                svgElement.style.transform = 'scale(1.0)';
                const viewBox = svgElement.getAttribute('viewBox');
                if (viewBox) {
                    const parts = viewBox.split(/\s+/).map(parseFloat);
                    if (parts.length === 4) {
                        console.log(`📏 Rendered at natural size: viewBox ${parts[2].toFixed(0)}×${parts[3].toFixed(0)}pt`);
                    }
                }
            }

            setTimeout(() => {
                const overlayGroup = document.querySelector('#arg-overlays');
                if (overlayGroup) {
                    console.log(`✅ Overlay group found with ${overlayGroup.children.length} children`);
                } else {
                    console.error('❌ Overlay group not found in DOM!');
                }
            }, 100);

            const debugPanel = document.getElementById('debugPanel');
            if (debugPanel && debugPanel.style.display !== 'none') {
                updateDebugPanel();
            }

            checkTypesDebounced();
        } else {
            showStatus('Render failed: ' + data.error, 'error');
            console.error('renderStructuralEditor: backend reported failure', data);
        }
    } catch (e) {
        showStatus('Network error: ' + e.message, 'error');
        console.error('renderStructuralEditor: fetch threw error', e);
    }
}

export function applyZoom() {
    const svg = document.querySelector('#structuralEditor svg');
    if (svg) {
        svg.style.transform = `scale(${state.currentZoom})`;
    }
    showStatus(`🔍 Zoom: ${(state.currentZoom * 100).toFixed(0)}%`, 'info');
}
