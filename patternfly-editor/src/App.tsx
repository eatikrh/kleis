import { useState } from 'react';
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
} from '@patternfly/react-core';
import { 
  SearchPlusIcon, 
  SearchMinusIcon, 
  UndoIcon, 
  RedoIcon,
  SyncIcon 
} from '@patternfly/react-icons';

import { PaletteTabs } from './components/Palette';
import type { EditorNode } from './types/ast';

import './App.css';

type EditorMode = 'structural' | 'text';

function App() {
  const [mode, setMode] = useState<EditorMode>('structural');
  const [currentAST, setCurrentAST] = useState<EditorNode | null>(null);
  const [zoom, setZoom] = useState(100);
  const [svgContent, setSvgContent] = useState<string>('');
  const [latexInput, setLatexInput] = useState('');

  const handleInsert = (ast: EditorNode) => {
    setCurrentAST(ast);
    console.log('Generated AST:', JSON.stringify(ast, null, 2));
    // TODO: Send to backend for rendering
  };

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
      </MastheadContent>
    </Masthead>
  );

  return (
    <Page masthead={masthead} className="kleis-page">
      <PageSection className="kleis-main">
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
                    style={{ transform: `scale(${zoom / 100})` }}
                  >
                    {svgContent ? (
                      <div dangerouslySetInnerHTML={{ __html: svgContent }} />
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
                          Click palette buttons to build your equation.
                          <br />
                          <small>SVG rendering will connect to Rust backend.</small>
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
              <CardTitle>🎨 Symbol Palette</CardTitle>
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
                {currentAST ? (
                  <Panel>
                    <PanelMain>
                      <PanelMainBody>
                        <p><strong>Operation:</strong> {'Operation' in currentAST ? currentAST.Operation.name : 'N/A'}</p>
                        <p><strong>Type:</strong> <span className="type-badge">Checking...</span></p>
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
                {currentAST ? (
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
