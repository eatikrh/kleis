/**
 * SVG Editor component with placeholder click handling and inline editing
 */

import { useRef, useEffect, useMemo } from 'react';
import type { PlaceholderPosition, ArgumentSlot, ArgumentBoundingBox } from '../../api/kleis';
import { InlineEditor } from './InlineEditor';

const normalizeSlotId = (id: string) => (id.startsWith('ph') ? id : `ph${id}`);

interface SVGEditorProps {
  svg: string;
  placeholders: PlaceholderPosition[];
  argumentSlots: ArgumentSlot[];
  argumentBoundingBoxes: ArgumentBoundingBox[];
  zoom: number;
  activeMarkerId: string | null;
  inlineEditing: {
    active: boolean;
    placeholderId: string;
    x: number;
    y: number;
    width: number;
    height: number;
    value: string;
  } | null;
  onPlaceholderClick: (id: string, path: number[], nodeId: string, event: MouseEvent) => void;
  onInlineCommit: (value: string) => void;
  onInlineCancel: () => void;
  onInlineAppend: (text: string) => void;
}

export function SVGEditor({
  svg,
  placeholders,
  argumentSlots,
  argumentBoundingBoxes,
  zoom,
  activeMarkerId,
  inlineEditing,
  onPlaceholderClick: _onPlaceholderClick, // Not used directly - handlers are in window
  onInlineCommit,
  onInlineCancel,
  onInlineAppend,
}: SVGEditorProps) {
  const svgContainerRef = useRef<HTMLDivElement>(null);
  // Track previous SVG for inline editing preservation
  const prevSvgRef = useRef<string>('');
  
  // Helper to get SVG element from DOM (no state, no re-renders)
  const getSvgElement = () => svgContainerRef.current?.querySelector('svg') as SVGSVGElement | null;

  // Compute SVG with overlays using useMemo (no setState = no infinite loops)
  // This is a direct port of static/index.html overlay generation logic
  const svgWithOverlays = useMemo(() => {
    // If inline editing is active, preserve previous SVG (don't regenerate)
    if (inlineEditing && inlineEditing.active) {
      return prevSvgRef.current;
    }
    
    if (!svg || argumentSlots.length === 0) {
      prevSvgRef.current = svg;
      return svg;
    }

    // Helper to compute node_id from path (EXACTLY like static/index.html)
    const nodeIdFromPath = (path: number[]): string => {
      if (path.length === 0) return '0';
      return '0.' + path.join('.');
    };

    const overlayElements: string[] = [];

    // OPTION B: Only show markers for leaf nodes (not parent operations)
    // Build a set of parent node IDs (nodes that have children)
    const parentNodeIds = new Set<string>();
    argumentSlots.forEach(slot => {
      const nodeId = nodeIdFromPath(slot.path || []);
      // If any slot's path is longer and starts with this nodeId, it's a parent
      argumentSlots.forEach(otherSlot => {
        const otherNodeId = nodeIdFromPath(otherSlot.path || []);
        if (otherNodeId.startsWith(nodeId + '.')) {
          parentNodeIds.add(nodeId);
        }
      });
    });
    console.log(`Identified ${parentNodeIds.size} parent nodes (will hide their markers)`);

    argumentSlots.forEach((slot, index) => {
      // FILTER: Skip parent operations, only show leaf nodes
      const nodeId = nodeIdFromPath(slot.path || []);
      if (parentNodeIds.has(nodeId)) {
        console.log(`  Skipping parent node ${nodeId} (has children)`);
        return; // Skip this slot
      }
      
      const slotId = normalizeSlotId(slot.id);
      const role = slot.role || null;
      
      let rectX = 0, rectY = 0, rectWidth = 0, rectHeight = 0;
      let foundPosition = false;
      let currentNodeId: string | null = null;
      
      // COORDINATE_PREFERENCE = 'placeholder' (like static/index.html)
      // Try placeholder position FIRST
      // For placeholder slots, extract numeric ID from "ph{number}" format
      // For filled slots (UUID), try matching by position in placeholder array
      let searchId: number | null = null;
      if (slot.is_placeholder && typeof slot.id === 'string' && slot.id.startsWith('ph')) {
        searchId = parseInt(slot.id.substring(2), 10);
      } else if (typeof slot.id === 'number') {
        searchId = slot.id;
      }
      
      const ph = searchId !== null && placeholders.length > 0 
        ? placeholders.find(p => p.id === searchId) 
        : null;
        
      if (ph) {
        rectX = ph.x - 3;
        rectY = ph.y - 3;
        rectWidth = ph.width + 6;
        rectHeight = ph.height + 6;
        foundPosition = true;
        console.log(`✅ Slot ${slot.id}: Using placeholder (x=${ph.x.toFixed(1)}, y=${ph.y.toFixed(1)})`);
      } else {
        // Fallback to semantic bounding box
        const nodePathId = nodeIdFromPath(slot.path || []);
        const bbox = argumentBoundingBoxes.find(b => b.node_id === nodePathId);
        
        if (bbox) {
          // Add more padding to prevent bunching
          const padding = 5;
          rectX = bbox.x - padding;
          rectY = bbox.y - padding;
          rectWidth = bbox.width + (padding * 2);
          rectHeight = bbox.height + (padding * 2);
          foundPosition = true;
          currentNodeId = bbox.node_id;
          console.log(`⚠️ Slot ${slot.id}: Using semantic bbox (x=${bbox.x.toFixed(1)}, y=${bbox.y.toFixed(1)}) node=${nodePathId}`);
        } else {
          // Last resort: infer position from grid structure
          // For matrix cells, use existing placeholder positions to infer grid
          if (placeholders.length >= 2) {
            const xCoords = [...new Set(placeholders.map(p => Math.round(p.x / 10) * 10))].sort((a,b) => a-b);
            const yCoords = [...new Set(placeholders.map(p => Math.round(p.y / 10) * 10))].sort((a,b) => a-b);
            
            if (xCoords.length > 0 && yCoords.length > 0) {
              // Assume row-major order based on slot path
              const pathIdx = slot.path && slot.path.length > 0 ? slot.path[slot.path.length - 1] : index;
              const cols = xCoords.length;
              const rowIdx = Math.floor(pathIdx / cols);
              const colIdx = pathIdx % cols;
              
              if (colIdx < xCoords.length && rowIdx < yCoords.length) {
                // Find actual x/y from nearest placeholder in that grid position
                const nearestPh = placeholders.find(p => 
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
            return; // Skip this slot
          }
        }
      }
      
      if (!foundPosition) return;
      
      // Scale down boxes but keep them visible (like static/index.html)
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
      
      // Role-based adjustments (like static/index.html)
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
        // align top edge with superscript top
        rectY -= superscriptShift;
        rectHeight = Math.max(6, rectHeight + superscriptShift);
      }

      // Color based on placeholder vs filled
      const color = slot.is_placeholder ? '#667eea' : '#28a745';
      const fillColor = slot.is_placeholder ? 'rgba(240, 244, 255, 0.3)' : 'rgba(40, 167, 69, 0.2)';
      
      // Create path string for data attribute (HTML-escaped)
      const pathStr = JSON.stringify(slot.path).replace(/"/g, '&quot;');
      // For onclick JavaScript, use the raw JSON array (like static/index.html)
      const pathJs = JSON.stringify(slot.path);
      
      // Use stored node_id or compute from path
      const finalNodeId = currentNodeId || nodeIdFromPath(slot.path || []);
      
      // Create rect overlay (like static/index.html)
      const rect = `<rect x="${rectX}" y="${rectY}" width="${rectWidth}" height="${rectHeight}"
        fill="${fillColor}" stroke="${color}" stroke-width="2" stroke-dasharray="6,3" rx="3"
        class="arg-overlay" data-slot-id="${slotId}" data-path="${pathStr}" data-node-id="${finalNodeId}"
        data-default-stroke="${color}" data-default-fill="${fillColor}"
        style="cursor: pointer;"
        tabindex="0" focusable="true"
        onclick="if(window.handleSlotClick){window.handleSlotClick(event,'${slotId}',${pathJs},'${finalNodeId}')}"
        onkeydown="if(window.handleSlotKeydown){window.handleSlotKeydown(event,'${slotId}',${pathJs},'${finalNodeId}')}" />`;
      overlayElements.push(rect);
    });

    // Inject overlays into SVG (like static/index.html)
    let result = svg;
    if (overlayElements.length > 0) {
      console.log(`Creating ${overlayElements.length} overlay elements`);
      result = svg.replace('</svg>', `<g id="arg-overlays" visibility="visible">${overlayElements.join('')}</g></svg>`);
      console.log('Overlays injected into SVG');
    } else {
      console.warn('No overlay elements created!');
    }
    prevSvgRef.current = result;
    return result;
  }, [svg, placeholders, argumentSlots, argumentBoundingBoxes, inlineEditing]);

  // Update active marker visual state when activeMarkerId changes
  // NOTE: Only depend on activeMarkerId, NOT svgWithOverlays - that was causing infinite loops
  useEffect(() => {
    if (!activeMarkerId) return;
    
    // Small delay to ensure DOM is ready after render
    const timeoutId = setTimeout(() => {
      const svgEl = getSvgElement();
      if (!svgEl) return;
      
      const normalizedId = normalizeSlotId(activeMarkerId);
      const activeOverlay = svgEl.querySelector(`[data-slot-id="${normalizedId}"]`) as SVGRectElement | null;
      
      if (activeOverlay) {
        // Remove active-marker class and reset inline attributes for all overlays
        svgEl.querySelectorAll('.arg-overlay, .placeholder-overlay').forEach(el => {
          el.classList.remove('active-marker');
          if (el instanceof SVGRectElement) {
            const defaultStroke = el.getAttribute('data-default-stroke') || '#667eea';
            const defaultFill = el.getAttribute('data-default-fill') || 'rgba(240, 244, 255, 0.3)';
            el.setAttribute('stroke-width', '2');
            el.setAttribute('stroke', defaultStroke);
            el.setAttribute('fill', defaultFill);
          }
        });
        // Add active-marker class and set inline attributes on the active overlay
        activeOverlay.classList.add('active-marker');
        activeOverlay.setAttribute('stroke-width', '4');
        activeOverlay.setAttribute('stroke', '#ff6b6b');
        activeOverlay.setAttribute('fill', 'rgba(255, 107, 107, 0.3)');
      }
    }, 50);
    
    return () => clearTimeout(timeoutId);
  }, [activeMarkerId]); // Only run when activeMarkerId changes - NOT svgWithOverlays!

  // Inject inline editor as foreignObject into SVG (like static/index.html)
  // Query DOM directly (like static/index.html) - don't rely on state references
  useEffect(() => {
    if (!svgContainerRef.current) {
      return;
    }

    // Query SVG directly from DOM (like static/index.html) - ensures we get the live element
    const container = svgContainerRef.current;
    const svgEl = container.querySelector('svg') as SVGElement | null;
    
    if (!svgEl) {
      return;
    }


    // Find or create foreignObject for inline editor (query DOM directly)
    let foreignObject = svgEl.querySelector('#inline-editor-foreign') as SVGForeignObjectElement;
    
    if (inlineEditing && inlineEditing.active) {
      
      if (!foreignObject) {
        // Create foreignObject (like static/index.html)
        foreignObject = document.createElementNS('http://www.w3.org/2000/svg', 'foreignObject');
        foreignObject.id = 'inline-editor-foreign';
        
        // Append to live SVG element from DOM
        svgEl.appendChild(foreignObject);
        
        // Verify it's actually in the DOM
        const inDOM = document.contains(foreignObject);
        if (!inDOM) {
          console.error('❌ SVGEditor: CRITICAL - foreignObject not in DOM after append!');
          return;
        }
      } else {
      }

      // Position at marker location (like static/index.html)
      // Extract viewBox offset to account for SVG coordinate system
      let viewBoxOffsetX = 0;
      let viewBoxOffsetY = 0;
      const viewBox = svgEl.getAttribute('viewBox');
      if (viewBox) {
        const parts = viewBox.split(/\s+/).map(parseFloat);
        if (parts.length === 4) {
          // viewBox format: "minX minY width height"
          viewBoxOffsetX = parts[0] || 0;
          viewBoxOffsetY = parts[1] || 0;
        }
      }
      
      // Position relative to SVG coordinate system (accounting for viewBox offset)
      // The bounding box coordinates are already in SVG space, so we use them directly
      const posX = inlineEditing.x - 10;
      const posY = inlineEditing.y - 5;
      const width = Math.max(200, inlineEditing.width + 40);
      const height = Math.max(40, inlineEditing.height + 10);

      foreignObject.setAttribute('x', String(posX));
      foreignObject.setAttribute('y', String(posY));
      foreignObject.setAttribute('width', String(width));
      foreignObject.setAttribute('height', String(height));
      foreignObject.setAttribute('visibility', 'visible');
      // CSS styles for foreignObject (like static/index.html)
      foreignObject.style.display = 'block';
      foreignObject.style.overflow = 'visible';
      foreignObject.style.pointerEvents = 'all';
      // Also set CSS width/height to ensure visibility
      foreignObject.style.width = `${width}px`;
      foreignObject.style.height = `${height}px`;
      
      // Force a reflow to ensure foreignObject is rendered before focusing input
      void foreignObject.offsetHeight;
      

      // Create or update input element
      let input = foreignObject.querySelector('input') as HTMLInputElement;
      if (!input) {
        // Create wrapper div (foreignObject needs HTML content in proper namespace)
        const wrapper = document.createElementNS('http://www.w3.org/1999/xhtml', 'div');
        wrapper.style.width = '100%';
        wrapper.style.height = '100%';

        // Create input inside wrapper
        input = document.createElementNS('http://www.w3.org/1999/xhtml', 'input') as HTMLInputElement;
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

        // Setup keyboard handlers
        input.addEventListener('keydown', (e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            onInlineCommit(input.value);
          } else if (e.key === 'Escape') {
            e.preventDefault();
            onInlineCancel();
          } else if (e.key === 'Tab') {
            e.preventDefault();
            onInlineCommit(input.value);
          }
        });

        // DISABLED: blur auto-commit causes race conditions
        // User must press Enter or Tab to commit, or click elsewhere
        // The blur handler was causing the highlight to disappear because:
        // 1. Input opens and gets focus
        // 2. Something causes blur (maybe React re-render stealing focus?)
        // 3. Blur handler commits empty value → AST updates → SVG re-renders → highlight lost
        // 
        // For now, only explicit actions (Enter/Tab) will commit.
        // TODO: Re-enable blur commit once race conditions are fixed

        // Expose append function to window for palette buttons
        (window as any).appendToInlineEditor = (text: string) => {
          if (input) {
            const start = input.selectionStart || 0;
            const end = input.selectionEnd || 0;
            const currentValue = input.value;
            input.value = currentValue.substring(0, start) + text + currentValue.substring(end);
            const newPos = start + text.length;
            input.setSelectionRange(newPos, newPos);
            input.focus();
          }
        };
      }

      // Set current value
      input.value = inlineEditing.value || '';
      
      // Focus and select immediately (like static/index.html)
      // Use a very short delay to ensure DOM is ready
      setTimeout(() => {
        input.focus();
        input.select();
      }, 10);

      // Add visual feedback class to active marker
      const activeOverlay = svgEl.querySelector(`[data-slot-id="${inlineEditing.placeholderId}"]`);
      if (activeOverlay) {
        activeOverlay.classList.add('editing-inline');
      } else {
        console.warn('🟡 SVGEditor: Could not find overlay for placeholderId:', inlineEditing.placeholderId);
      }
      document.body.classList.add('inline-editing');
    } else {
      // REMOVE inline editor completely (not just hide) to prevent stale event handlers
      if (svgContainerRef.current) {
        const svgEl = svgContainerRef.current.querySelector('svg');
        if (svgEl) {
          const foreignObject = svgEl.querySelector('#inline-editor-foreign') as SVGForeignObjectElement;
          if (foreignObject) {
            // Remove completely so it gets recreated fresh with new handlers next time
            foreignObject.remove();
          }
          
          // Remove visual feedback
          svgEl.querySelectorAll('.editing-inline').forEach(el => {
            el.classList.remove('editing-inline');
          });
        }
      }
      document.body.classList.remove('inline-editing');
      
      // Clean up window function
      delete (window as any).appendToInlineEditor;
    }
  }, [inlineEditing, onInlineCommit, onInlineCancel]);

  return (
    <div className="svg-editor-container" ref={svgContainerRef} style={{ position: 'relative' }}>
      <div
        className="svg-container"
        dangerouslySetInnerHTML={{ __html: svgWithOverlays }}
        style={{ position: 'relative' }}
      />
    </div>
  );
}

