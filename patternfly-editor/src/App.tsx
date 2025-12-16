import { useState, useEffect, useCallback } from 'react';
import {
  Page,
  PageSection,
  Masthead,
  MastheadMain,
  MastheadBrand,
  MastheadContent,
  Title,
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

import { PaletteTabs, getTemplateCount } from './components/Palette';
import type { EditorNode } from './types/ast';
import { useServerStatus, useRenderTypst, useTypeCheck, useKleisOutput } from './hooks/useKleisAPI';

import './App.css';

type EditorMode = 'structural' | 'text';

function App() {
  const [mode, setMode] = useState<EditorMode>('structural');
  const [currentAST, setCurrentAST] = useState<EditorNode | null>(null);
  const [zoom, setZoom] = useState(100);
  const [latexInput, setLatexInput] = useState('');
  
  // API hooks
  const { isConnected, checking: checkingServer } = useServerStatus();
  const { svg, placeholders, loading: renderLoading, error: renderError, render: renderSvg } = useRenderTypst();
  const { type: typeResult, loading: typeLoading, error: typeError, check: checkType } = useTypeCheck();
  const { output: kleisOutput, loading: kleisLoading, render: renderKleisOutput } = useKleisOutput();

  // Auto-render when AST changes
  useEffect(() => {
    if (currentAST && isConnected) {
      renderSvg(currentAST);
      checkType(currentAST);
      renderKleisOutput(currentAST);
    }
  }, [currentAST, isConnected, renderSvg, checkType, renderKleisOutput]);

  const handleInsert = useCallback((ast: EditorNode) => {
    setCurrentAST(ast);
    console.log('Generated AST:', JSON.stringify(ast, null, 2));
  }, []);

  const handleZoomIn = () => setZoom(z => Math.min(z + 25, 200));
  const handleZoomOut = () => setZoom(z => Math.max(z - 25, 50));
  const handleZoomReset = () => setZoom(100);

  // Header with logo and mode toggle
  const masthead = (
    <Masthead className="kleis-header">
      <MastheadMain>
        <MastheadBrand className="kleis-brand">
          <img src="/kleis-icon.svg" alt="Kleis" className="kleis-logo" />
          <div className="kleis-title">
            <Title headingLevel="h1" size="2xl">Kleis Equation Editor</Title>
            <span className="kleis-subtitle">Structural Math Editor with Type Verification</span>
          </div>
        </MastheadBrand>
      </MastheadMain>
      <MastheadContent>
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
      </MastheadContent>
    </Masthead>
  );

  return (
    <Page masthead={masthead} className="kleis-page">
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
                      <Button variant="plain" onClick={handleZoomReset} aria-label="Reset zoom">
                        <SyncIcon />
                      </Button>
                      <Button variant="plain" aria-label="Undo">
                        <UndoIcon />
                      </Button>
                      <Button variant="plain" aria-label="Redo">
                        <RedoIcon />
                      </Button>
                    </div>
                  )}
                </div>
              </CardTitle>
              <CardBody>
                {mode === 'structural' ? (
                  <div 
                    className="structural-editor"
                    style={{ transform: `scale(${zoom / 100})`, transformOrigin: 'top left' }}
                  >
                    {renderLoading ? (
                      <div className="loading-state">
                        <Spinner size="lg" />
                        <p>Rendering equation...</p>
                      </div>
                    ) : svg ? (
                      <div 
                        className="svg-container"
                        dangerouslySetInnerHTML={{ __html: svg }} 
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
                    
                    {/* Placeholder overlays would go here */}
                    {placeholders.length > 0 && (
                      <div className="placeholder-overlays">
                        {placeholders.map((pos) => (
                          <div
                            key={pos.id}
                            className="placeholder-overlay"
                            style={{
                              left: pos.x,
                              top: pos.y,
                              width: pos.width,
                              height: pos.height,
                            }}
                            onClick={() => console.log('Clicked placeholder:', pos.id)}
                          />
                        ))}
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
                <PaletteTabs onInsert={handleInsert} />
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
              <CardTitle>📤 Kleis Output</CardTitle>
              <CardBody>
                {kleisLoading ? (
                  <Spinner size="sm" />
                ) : kleisOutput ? (
                  <CodeBlock>
                    <CodeBlockCode>{kleisOutput}</CodeBlockCode>
                  </CodeBlock>
                ) : currentAST ? (
                  <CodeBlock>
                    <CodeBlockCode>
                      {'Operation' in currentAST 
                        ? `${currentAST.Operation.name}(□, □)` 
                        : 'N/A'}
                    </CodeBlockCode>
                  </CodeBlock>
                ) : (
                  <p className="muted">Kleis notation will appear here</p>
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
        </Grid>
      </PageSection>
    </Page>
  );
}

export default App;
