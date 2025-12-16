import { useState } from 'react';
import {
  Page,
  PageSection,
  Masthead,
  MastheadMain,
  MastheadBrand,
  MastheadContent,
  Title,
  Split,
  SplitItem,
  Brand,
} from '@patternfly/react-core';

import { PaletteTabs } from './components/Palette';
import { ASTPreview } from './components/Preview';
import type { EditorNode } from './types/ast';

import './App.css';

function App() {
  const [currentAST, setCurrentAST] = useState<EditorNode | null>(null);
  const [astHistory, setAstHistory] = useState<EditorNode[]>([]);

  const handleInsert = (ast: EditorNode) => {
    setCurrentAST(ast);
    setAstHistory((prev) => [...prev, ast]);
    
    // Log for verification against reference implementation
    console.log('Generated AST:', JSON.stringify(ast, null, 2));
  };

  const masthead = (
    <Masthead>
      <MastheadMain>
        <MastheadBrand>
          <Brand
            src="/kleis-icon.svg"
            alt="Kleis"
            style={{ height: '32px', width: 'auto' }}
          >
            <source srcSet="/kleis-icon.svg" />
          </Brand>
          <Title headingLevel="h1" size="xl" style={{ marginLeft: '12px', color: 'white' }}>
            Kleis Equation Editor
          </Title>
        </MastheadBrand>
      </MastheadMain>
      <MastheadContent>
        <span style={{ color: 'var(--pf-v5-global--Color--light-200)', fontSize: '0.875rem' }}>
          PatternFly Edition
        </span>
      </MastheadContent>
    </Masthead>
  );

  return (
    <Page masthead={masthead}>
      <PageSection>
        <Split hasGutter>
          <SplitItem isFilled>
            <div className="editor-main">
              <Title headingLevel="h2" size="lg" style={{ marginBottom: '16px' }}>
                Symbol Palette
              </Title>
              <PaletteTabs onInsert={handleInsert} />
            </div>
          </SplitItem>
          <SplitItem style={{ width: '400px' }}>
            <ASTPreview ast={currentAST} />
            
            {astHistory.length > 0 && (
              <div style={{ marginTop: '16px' }}>
                <Title headingLevel="h3" size="md">
                  History ({astHistory.length} items)
                </Title>
                <ul style={{ fontSize: '0.875rem', color: 'var(--pf-v5-global--Color--200)' }}>
                  {astHistory.slice(-5).map((ast, i) => (
                    <li key={i}>
                      {'Operation' in ast ? ast.Operation.name : 'unknown'}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </SplitItem>
        </Split>
      </PageSection>
    </Page>
  );
}

export default App;
