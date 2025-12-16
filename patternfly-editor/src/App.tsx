import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Page,
  PageSection,
  Card,
  CardBody,
  CardTitle,
  Grid,
  GridItem,
  ToggleGroup,
  ToggleGroupItem,
  Button,
  CodeBlock,
  CodeBlockCode,
  Panel,
  PanelMain,
  PanelMainBody,
  Alert,
  Spinner,
} from '@patternfly/react-core';
import { 
  SearchPlusIcon, 
  SearchMinusIcon, 
  UndoIcon, 
  RedoIcon,
  SyncIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
} from '@patternfly/react-icons';

import { PaletteTabs, getTemplateCount, astTemplates } from './components/Palette';
import type { EditorNode } from './types/ast';
import { useServerStatus, useRenderTypst, useTypeCheck } from './hooks/useKleisAPI';
import { useUndoRedo } from './hooks/useUndoRedo';
import { useVerify } from './hooks/useVerify';
import { SVGEditor } from './components/Editor/SVGEditor';
import { MatrixBuilder } from './components/Editor/MatrixBuilder';
import { PiecewiseBuilder } from './components/Editor/PiecewiseBuilder';
import { setNodeAtPath, getNodeAtPath, getAllPlaceholderPaths, parseSimpleInput, getPathFromPlaceholderId } from './utils/astUtils';
import type { PlaceholderPosition } from './api/kleis';

import './App.css';

const normalizeSlotId = (id: string | number) => {
  const str = String(id);
  return str.startsWith('ph') ? str : `ph${str}`;
};

type EditorMode = 'structural' | 'text';

function App() {
  const [mode, setMode] = useState<EditorMode>('structural');
  const [zoom, setZoom] = useState(200);  // Default to 200% for better readability
  const [latexInput, setLatexInput] = useState('');
  const [activeMarkerId, setActiveMarkerId] = useState<string | null>(null);
  const [activeMarkerPath, setActiveMarkerPath] = useState<number[] | null>(null);
  // Backup ref for active path - survives blur handler race conditions
  const activeMarkerPathRef = useRef<number[] | null>(null);
  // Guard to prevent inline commit during palette insertion
  const isInsertingRef = useRef<boolean>(false);
  const [inlineEditing, setInlineEditing] = useState<{
    active: boolean;
    placeholderId: string;
    x: number;
    y: number;
    width: number;
    height: number;
    value: string;
  } | null>(null);
  const [matrixBuilderOpen, setMatrixBuilderOpen] = useState(false);
  const [piecewiseBuilderOpen, setPiecewiseBuilderOpen] = useState(false);
  
  // API hooks
  const { isConnected, checking: checkingServer } = useServerStatus();
  const { svg, placeholders, argumentSlots, argumentBoundingBoxes, loading: renderLoading, error: renderError, render: renderSvg } = useRenderTypst();
  const { type: typeResult, loading: typeLoading, error: typeError, check: checkType } = useTypeCheck();
  const { verify, checkSat, verifying, verifyResult, verifyError, checkingSat, satResult, satError } = useVerify();

  // Undo/Redo hook
  const { currentAST, updateAST, undo, redo, canUndo, canRedo } = useUndoRedo(null);

  // Define marker navigation functions BEFORE keyboard handler
  const focusNextMarker = useCallback(() => {
    if (!currentAST) return;
    const allPaths = getAllPlaceholderPaths(currentAST);
    if (allPaths.length === 0) return;

    let currentIndex = -1;
    if (activeMarkerPath) {
      currentIndex = allPaths.findIndex(p => 
        JSON.stringify(p.path) === JSON.stringify(activeMarkerPath)
      );
    }

    const nextIndex = (currentIndex + 1) % allPaths.length;
    const nextPath = allPaths[nextIndex];
    const placeholder = placeholders.find(p => p.id === String(nextPath.id));
    
    if (placeholder) {
      setActiveMarkerId(String(nextPath.id));
      setActiveMarkerPath(nextPath.path);
    }
  }, [currentAST, activeMarkerPath, placeholders]);

  const focusPrevMarker = useCallback(() => {
    if (!currentAST) return;
    const allPaths = getAllPlaceholderPaths(currentAST);
    if (allPaths.length === 0) return;

    let currentIndex = -1;
    if (activeMarkerPath) {
      currentIndex = allPaths.findIndex(p => 
        JSON.stringify(p.path) === JSON.stringify(activeMarkerPath)
      );
    }

    const prevIndex = currentIndex <= 0 ? allPaths.length - 1 : currentIndex - 1;
    const prevPath = allPaths[prevIndex];
    const placeholder = placeholders.find(p => p.id === String(prevPath.id));
    
    if (placeholder) {
      setActiveMarkerId(String(prevPath.id));
      setActiveMarkerPath(prevPath.path);
    }
  }, [currentAST, activeMarkerPath, placeholders]);

  const handlePlaceholderClick = useCallback((id: string, path: number[], nodeId: string, event: MouseEvent) => {
    const normalizedId = normalizeSlotId(id);
    if (!currentAST) {
      console.warn('handlePlaceholderClick: No currentAST');
      return;
    }


    // Extract bbox from clicked element (like static/index.html)
    let bbox = { x: 0, y: 0, width: 0, height: 0 };
    if (event && event.target) {
      const rect = event.target as SVGRectElement;
      bbox = {
        x: parseFloat(rect.getAttribute('x') || '0'),
        y: parseFloat(rect.getAttribute('y') || '0'),
        width: parseFloat(rect.getAttribute('width') || '0'),
        height: parseFloat(rect.getAttribute('height') || '0'),
      };
    }

    // Use the path directly from the click handler (like static/index.html)
    setActiveMarkerId(normalizedId);
    setActiveMarkerPath(path);

    // Get current value at this path
    const node = getNodeAtPath(currentAST, path);
    let currentValue = '';
    if (node) {
      if ('Const' in node) currentValue = typeof node.Const === 'string' ? node.Const : node.Const.value;
      else if ('Object' in node) currentValue = node.Object;
    }


    // Show inline editor
    setInlineEditing({
      active: true,
      placeholderId: normalizedId,
      x: bbox.x,
      y: bbox.y,
      width: bbox.width,
      height: bbox.height,
      value: currentValue,
    });
  }, [currentAST]);

  // Expose handler to window for SVG onclick handlers (like static/index.html)
  useEffect(() => {
    (window as any).handleSlotClick = (event: MouseEvent, id: string, path: number[] | string, nodeId: string) => {
      console.log('handleSlotClick called!', { id, path, nodeId });
      const normalizedId = normalizeSlotId(id);
      
      // Parse path if it's a string (from SVG onclick attribute)
      let parsedPath: number[] = [];
      if (typeof path === 'string') {
        try {
          parsedPath = JSON.parse(path.replace(/&quot;/g, '"'));
        } catch (e) {
          console.error('Failed to parse path:', path, e);
          return;
        }
      } else if (Array.isArray(path)) {
        parsedPath = path;
      } else {
        console.error('Invalid path format:', path);
        return;
      }
      
      if (event) {
        event.stopPropagation();
        event.preventDefault();
      }
      
      // Apply active-marker class immediately (like static/index.html)
      // Remove from all overlays first
      document.querySelectorAll('.arg-overlay, .placeholder-overlay').forEach(el => {
        el.classList.remove('active-marker');
        if (el instanceof SVGRectElement) {
          const defaultStroke = el.getAttribute('data-default-stroke') || '#667eea';
          const defaultFill = el.getAttribute('data-default-fill') || 'rgba(240, 244, 255, 0.3)';
          el.setAttribute('stroke-width', '2');
          el.setAttribute('stroke', defaultStroke);
          el.setAttribute('fill', defaultFill);
        }
      });
      
      // Highlight the clicked element - use querySelector as fallback
      let clickedOverlay = event?.target as SVGRectElement | null;
      // If event target isn't the rect itself, find it by data-slot-id
      if (!clickedOverlay || !(clickedOverlay instanceof SVGRectElement)) {
        clickedOverlay = document.querySelector(`[data-slot-id="${normalizedId}"]`) as SVGRectElement | null;
      }
      console.log('clickedOverlay:', clickedOverlay, 'normalizedId:', normalizedId);
      if (clickedOverlay && clickedOverlay instanceof SVGRectElement) {
        clickedOverlay.classList.add('active-marker');
        clickedOverlay.setAttribute('stroke-width', '4');
        clickedOverlay.setAttribute('stroke', '#ff6b6b');
        clickedOverlay.setAttribute('fill', 'rgba(255, 107, 107, 0.3)');
        console.log('Applied highlight to:', clickedOverlay.getAttribute('data-slot-id'));
      } else {
        console.warn('Could not find overlay for:', normalizedId);
      }
      
      // Extract bbox for inline editor positioning
      let bbox = { x: 0, y: 0, width: 0, height: 0 };
      if (clickedOverlay) {
        bbox = {
          x: parseFloat(clickedOverlay.getAttribute('x') || '0'),
          y: parseFloat(clickedOverlay.getAttribute('y') || '0'),
          width: parseFloat(clickedOverlay.getAttribute('width') || '0'),
          height: parseFloat(clickedOverlay.getAttribute('height') || '0'),
        };
      }
      
      // Get current value at this path
      let currentValue = '';
      if (currentAST) {
        const node = getNodeAtPath(currentAST, parsedPath);
        if (node) {
          if ('Const' in node) currentValue = typeof node.Const === 'string' ? node.Const : node.Const.value;
          else if ('Object' in node) currentValue = node.Object;
        }
      }
      
      // Set state all at once to minimize re-renders
      console.log('Setting activeMarkerPath to:', parsedPath);
      setActiveMarkerId(normalizedId);
      setActiveMarkerPath(parsedPath);
      activeMarkerPathRef.current = parsedPath; // Backup in ref for race conditions
      setInlineEditing({
        active: true,
        placeholderId: normalizedId,
        x: bbox.x,
        y: bbox.y,
        width: bbox.width,
        height: bbox.height,
        value: currentValue,
      });
      
    };

    (window as any).handleSlotKeydown = (event: KeyboardEvent, id: string, path: number[], nodeId: string) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        const syntheticEvent = new MouseEvent('click', { bubbles: true, cancelable: true });
        (window as any).handleSlotClick(syntheticEvent, id, path, nodeId);
      }
    };

    return () => {
      delete (window as any).handleSlotClick;
      delete (window as any).handleSlotKeydown;
    };
  }, [currentAST]); // Only depend on currentAST, not on callback functions

  // Auto-render when AST changes
  useEffect(() => {
    if (currentAST && isConnected) {
      renderSvg(currentAST);
      checkType(currentAST);
    }
  }, [currentAST, isConnected, renderSvg, checkType]);

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Only handle keys when in structural mode and editor is focused
      if (mode !== 'structural' || !currentAST) return;

      // Cmd/Ctrl+Z for undo
      if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        if (canUndo) undo();
        return;
      }

      // Cmd/Ctrl+Shift+Z for redo
      if ((e.metaKey || e.ctrlKey) && e.key === 'z' && e.shiftKey) {
        e.preventDefault();
        if (canRedo) redo();
        return;
      }

      // Arrow keys for marker navigation
      if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
        e.preventDefault();
        focusNextMarker();
      } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
        e.preventDefault();
        focusPrevMarker();
      } 
      // Tab key navigation (like static/index.html)
      else if (e.key === 'Tab') {
        e.preventDefault();
        if (e.shiftKey) {
          focusPrevMarker();
        } else {
          focusNextMarker();
        }
      } 
      else if (e.key === 'Enter' && activeMarkerId && activeMarkerPath) {
        e.preventDefault();
        // Focus the inline editor input if it exists
        const input = document.querySelector('#inline-input') as HTMLInputElement;
        if (input) {
          input.focus();
          input.select();
        }
      } else if (e.key === 'Escape') {
        e.preventDefault();
        setActiveMarkerId(null);
        setActiveMarkerPath(null);
        setInlineEditing(null);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [mode, currentAST, activeMarkerId, activeMarkerPath, canUndo, canRedo, undo, redo, focusNextMarker, focusPrevMarker]);

  const handleInlineCommit = useCallback((value: string) => {
    // Guard: If we're in the middle of a palette insertion, don't commit
    // This prevents stale closures from overwriting the inserted value
    if (isInsertingRef.current) {
      console.log('handleInlineCommit: blocked by isInsertingRef guard');
      return;
    }
    
    if (!currentAST || !activeMarkerPath) {
      console.log('handleInlineCommit: no currentAST or activeMarkerPath');
      return;
    }

    console.log('handleInlineCommit: committing value:', value);
    const newNode = parseSimpleInput(value);
    const updatedAST = setNodeAtPath(currentAST, activeMarkerPath, newNode);
    updateAST(updatedAST);
    
    setInlineEditing(null);
    setActiveMarkerId(null);
    setActiveMarkerPath(null);
  }, [currentAST, activeMarkerPath, updateAST]);

  const handleInlineCancel = useCallback(() => {
    setInlineEditing(null);
  }, []);

  const handleInlineAppend = useCallback((text: string) => {
    if (!inlineEditing) return;
    setInlineEditing({
      ...inlineEditing,
      value: inlineEditing.value + text,
    });
  }, [inlineEditing]);

  const handleInsert = useCallback((ast: EditorNode) => {
    console.log('handleInsert called, activeMarkerPath:', activeMarkerPath, 'pathRef:', activeMarkerPathRef.current, 'currentAST:', !!currentAST);
    
    // Set guard to prevent inline commit from firing during insertion
    isInsertingRef.current = true;
    
    // Check for active marker OR ref (survives blur race) OR inline editing location
    const insertPath = activeMarkerPath 
      || activeMarkerPathRef.current 
      || (inlineEditing ? getPathFromPlaceholderId(inlineEditing.placeholderId, currentAST) : null);
    console.log('insertPath:', insertPath);
    
    // IMPORTANT: Clear state FIRST before updating AST
    // This prevents race conditions where old event handlers fire during re-render
    setInlineEditing(null);
    setActiveMarkerId(null);
    setActiveMarkerPath(null);
    activeMarkerPathRef.current = null;
    
    // Clear visual highlighting immediately
    document.querySelectorAll('.arg-overlay, .placeholder-overlay').forEach(el => {
      el.classList.remove('active-marker');
      if (el instanceof SVGRectElement) {
        const defaultStroke = el.getAttribute('data-default-stroke') || '#667eea';
        const defaultFill = el.getAttribute('data-default-fill') || 'rgba(240, 244, 255, 0.3)';
        el.setAttribute('stroke-width', '2');
        el.setAttribute('stroke', defaultStroke);
        el.setAttribute('fill', defaultFill);
      }
    });
    
    // NOW update the AST (triggers re-render with inline editor already closed)
    if (insertPath && currentAST) {
      console.log('Inserting at path:', insertPath);
      const updatedAST = setNodeAtPath(currentAST, insertPath, ast);
      updateAST(updatedAST);
    } else {
      console.log('Replacing whole AST');
      updateAST(ast);
    }
    
    // Clear the guard after a short delay to ensure all stale handlers have been blocked
    setTimeout(() => {
      isInsertingRef.current = false;
    }, 100);
    
  }, [activeMarkerPath, inlineEditing, currentAST, updateAST]);

  const handleMatrixCreate = useCallback((ast: EditorNode) => {
    handleInsert(ast);
  }, [handleInsert]);

  const handlePiecewiseCreate = useCallback((ast: EditorNode) => {
    handleInsert(ast);
  }, [handleInsert]);

  const handleVerify = useCallback(async () => {
    if (!currentAST) return;
    await verify(currentAST);
  }, [currentAST, verify]);

  const handleCheckSat = useCallback(async () => {
    if (!currentAST) return;
    await checkSat(currentAST);
  }, [currentAST, checkSat]);

  const handleZoomIn = () => setZoom(z => Math.min(z + 25, 400));
  const handleZoomOut = () => setZoom(z => Math.max(z - 25, 100));
  const handleZoomReset = () => setZoom(200);
  
  // Reset/clear AST to default equation
  const handleClearAST = useCallback(() => {
    // Clear active marker state
    setActiveMarkerId(null);
    setActiveMarkerPath(null);
    activeMarkerPathRef.current = null;
    setInlineEditing(null);
    // Reset zoom
    setZoom(200);
    // Reset AST to default equals template
    updateAST(astTemplates.equals);
  }, [updateAST]);

  return (
    <Page className="kleis-page">
      <div className="kleis-custom-header">
        <div className="kleis-brand">
          <img src="/kleis-icon.svg" alt="Kleis" className="kleis-logo" />
          <div className="kleis-title">
            <h1>Kleis Equation Editor</h1>
            <span className="kleis-subtitle">Structural Math Editor with Type Verification</span>
          </div>
        </div>
        <div className="header-controls">
          <ToggleGroup aria-label="Editor mode">
            <ToggleGroupItem
              text="Structural"
              buttonId="structural"
              isSelected={mode === 'structural'}
              onChange={() => setMode('structural')}
            />
            <ToggleGroupItem
              text="Text"
              buttonId="text"
              isSelected={mode === 'text'}
              onChange={() => setMode('text')}
            />
          </ToggleGroup>
          <div className="server-status">
            {checkingServer ? (
              <Spinner size="sm" />
            ) : isConnected ? (
              <span className="status-connected">
                <CheckCircleIcon /> Server
              </span>
            ) : (
              <span className="status-disconnected">
                <ExclamationCircleIcon /> Offline
              </span>
            )}
          </div>
        </div>
      </div>
      <PageSection className="kleis-main">
        {!isConnected && !checkingServer && (
          <Alert 
            variant="warning" 
            isInline 
            title="Backend server not connected"
            className="server-alert"
          >
            Run <code>cargo run --bin server</code> to enable rendering and type checking.
          </Alert>
        )}
        
        <Grid hasGutter>
          {/* Main Editor Panel */}
          <GridItem span={12}>
            <Card className="editor-card">
              <CardTitle>
                <div className="editor-card-header">
                  <span>📝 {mode === 'structural' ? 'Structural Editor' : 'LaTeX Input'}</span>
                  {mode === 'structural' && (
                    <div className="zoom-controls">
                      <Button variant="plain" onClick={handleZoomOut} aria-label="Zoom out">
                        <SearchMinusIcon />
                      </Button>
                      <span className="zoom-level">{zoom}%</span>
                      <Button variant="plain" onClick={handleZoomIn} aria-label="Zoom in">
                        <SearchPlusIcon />
                      </Button>
                      <Button variant="plain" onClick={handleClearAST} aria-label="Clear equation" title="Reset to empty equation">
                        <SyncIcon />
                      </Button>
                      <Button 
                        variant="plain" 
                        aria-label="Undo" 
                        onClick={undo}
                        isDisabled={!canUndo}
                        title={`Undo (${canUndo ? 'available' : 'unavailable'})`}
                      >
                        <UndoIcon />
                      </Button>
                      <Button 
                        variant="plain" 
                        aria-label="Redo" 
                        onClick={redo}
                        isDisabled={!canRedo}
                        title={`Redo (${canRedo ? 'available' : 'unavailable'})`}
                      >
                        <RedoIcon />
                      </Button>
                      <Button 
                        variant="primary" 
                        onClick={handleVerify}
                        isDisabled={!currentAST || verifying}
                        title="Verify validity"
                      >
                        ✓ Verify
                      </Button>
                      <Button 
                        variant="primary" 
                        onClick={handleCheckSat}
                        isDisabled={!currentAST || checkingSat}
                        title="Check satisfiability"
                      >
                        ∃ Sat?
                      </Button>
                    </div>
                  )}
                </div>
              </CardTitle>
              <CardBody>
                {mode === 'structural' ? (
                  <div 
                    className="structural-editor"
                    style={{ transform: `scale(${zoom / 100})`, transformOrigin: 'center center' }}
                  >
                    {renderLoading ? (
                      <div className="loading-state">
                        <Spinner size="lg" />
                        <p>Rendering equation...</p>
                      </div>
                    ) : svg ? (
                      <SVGEditor
                        svg={svg}
                        placeholders={placeholders}
                        argumentSlots={argumentSlots}
                        argumentBoundingBoxes={argumentBoundingBoxes}
                        zoom={zoom}
                        activeMarkerId={activeMarkerId}
                        inlineEditing={inlineEditing}
                        onPlaceholderClick={handlePlaceholderClick}
                        onInlineCommit={handleInlineCommit}
                        onInlineCancel={handleInlineCancel}
                        onInlineAppend={handleInlineAppend}
                      />
                    ) : renderError ? (
                      <div className="error-state">
                        <ExclamationCircleIcon />
                        <p>Render error: {renderError}</p>
                      </div>
                    ) : currentAST ? (
                      <div className="equation-placeholder">
                        <div className="placeholder-box">
                          {'Operation' in currentAST && (
                            <span className="operation-preview">
                              {currentAST.Operation.name}(
                              {currentAST.Operation.args.map((arg, i) => (
                                <span key={i} className="arg-box">
                                  {'Placeholder' in arg ? `□${arg.Placeholder.hint}` : '...'}
                                  {i < currentAST.Operation.args.length - 1 && ', '}
                                </span>
                              ))}
                              )
                            </span>
                          )}
                        </div>
                        <p className="placeholder-hint">
                          {isConnected 
                            ? 'Equation will render shortly...'
                            : 'Start server for SVG rendering'}
                        </p>
                      </div>
                    ) : (
                      <div className="empty-editor">
                        <p>Click a symbol in the palette below to start building your equation.</p>
                        <p className="hint">The equation will appear here with clickable placeholders.</p>
                      </div>
                    )}
                    
                  </div>
                ) : (
                  <textarea
                    className="latex-input"
                    value={latexInput}
                    onChange={(e) => setLatexInput(e.target.value)}
                    placeholder="Enter LaTeX equation here...
Example: \frac{1}{2} \int_{0}^{\infty} e^{-x^2} \, dx"
                  />
                )}
              </CardBody>
            </Card>
          </GridItem>

          {/* Symbol Palette */}
          <GridItem span={12}>
            <Card className="palette-card">
              <CardTitle>
                🎨 Symbol Palette
                <span className="template-count">{getTemplateCount()} templates</span>
              </CardTitle>
              <CardBody>
                <PaletteTabs 
                  onInsert={handleInsert}
                  onOpenMatrixBuilder={() => setMatrixBuilderOpen(true)}
                  onOpenPiecewiseBuilder={() => setPiecewiseBuilderOpen(true)}
                />
              </CardBody>
            </Card>
          </GridItem>

          {/* Output Panels */}
          <GridItem span={4}>
            <Card>
              <CardTitle>🔍 Type Info</CardTitle>
              <CardBody>
                {typeLoading ? (
                  <Spinner size="sm" />
                ) : typeError ? (
                  <Alert variant="danger" isInline isPlain title={typeError} />
                ) : typeResult ? (
                  <Panel>
                    <PanelMain>
                      <PanelMainBody>
                        <p><strong>Type:</strong> <span className="type-badge">{typeResult}</span></p>
                      </PanelMainBody>
                    </PanelMain>
                  </Panel>
                ) : currentAST ? (
                  <Panel>
                    <PanelMain>
                      <PanelMainBody>
                        <p><strong>Operation:</strong> {'Operation' in currentAST ? currentAST.Operation.name : 'N/A'}</p>
                        <p><strong>Type:</strong> <span className="type-badge">{isConnected ? 'Checking...' : 'Start server'}</span></p>
                      </PanelMainBody>
                    </PanelMain>
                  </Panel>
                ) : (
                  <p className="muted">Build an equation to see type information</p>
                )}
              </CardBody>
            </Card>
          </GridItem>


          <GridItem span={4}>
            <Card>
              <CardTitle>🧮 AST Debug</CardTitle>
              <CardBody>
                {currentAST ? (
                  <CodeBlock>
                    <CodeBlockCode className="ast-code">
                      {JSON.stringify(currentAST, null, 2)}
                    </CodeBlockCode>
                  </CodeBlock>
                ) : (
                  <p className="muted">AST will appear here</p>
                )}
              </CardBody>
            </Card>
          </GridItem>

          {/* Verify/SAT Results */}
          {(verifyResult || satResult) && (
            <GridItem span={12}>
              <Card>
                <CardTitle>🔬 Verification Results</CardTitle>
                <CardBody>
                  {verifyResult && (
                    <Alert 
                      variant={verifyResult.result === 'valid' ? 'success' : verifyResult.result === 'invalid' ? 'danger' : 'warning'}
                      isInline
                      title={`Verify: ${verifyResult.result}`}
                    >
                      {verifyResult.kleis_syntax && (
                        <div style={{ marginTop: '8px' }}>
                          <strong>Kleis:</strong> <code>{verifyResult.kleis_syntax}</code>
                        </div>
                      )}
                      {verifyResult.counterexample && (
                        <div style={{ marginTop: '8px' }}>
                          <strong>Counterexample:</strong> <code>{verifyResult.counterexample}</code>
                        </div>
                      )}
                      {verifyError && <div style={{ marginTop: '8px', color: '#c9190b' }}>{verifyError}</div>}
                    </Alert>
                  )}
                  {satResult && (
                    <Alert 
                      variant={satResult.result === 'satisfiable' ? 'success' : satResult.result === 'unsatisfiable' ? 'danger' : 'warning'}
                      isInline
                      title={`Satisfiability: ${satResult.result}`}
                      style={{ marginTop: verifyResult ? '16px' : '0' }}
                    >
                      {satResult.kleis_syntax && (
                        <div style={{ marginTop: '8px' }}>
                          <strong>Kleis:</strong> <code>{satResult.kleis_syntax}</code>
                        </div>
                      )}
                      {satResult.example && (
                        <div style={{ marginTop: '8px' }}>
                          <strong>Example:</strong> <code>{satResult.example}</code>
                        </div>
                      )}
                      {satError && <div style={{ marginTop: '8px', color: '#c9190b' }}>{satError}</div>}
                    </Alert>
                  )}
                </CardBody>
              </Card>
            </GridItem>
          )}
        </Grid>

        {/* Matrix Builder Modal */}
        <MatrixBuilder
          isOpen={matrixBuilderOpen}
          onClose={() => setMatrixBuilderOpen(false)}
          onCreate={handleMatrixCreate}
        />

        {/* Piecewise Builder Modal */}
        <PiecewiseBuilder
          isOpen={piecewiseBuilderOpen}
          onClose={() => setPiecewiseBuilderOpen(false)}
          onCreate={handlePiecewiseCreate}
        />
      </PageSection>
    </Page>
  );
}

export default App;
