import { Card, CardBody, CardTitle, CodeBlock, CodeBlockCode } from '@patternfly/react-core';
import type { EditorNode } from '../../types/ast';

interface ASTPreviewProps {
  ast: EditorNode | null;
}

/**
 * Displays the current AST as formatted JSON
 * 
 * This is for development/verification - helps compare output
 * with the reference implementation (static/index.html)
 */
export function ASTPreview({ ast }: ASTPreviewProps) {
  return (
    <Card>
      <CardTitle>AST Output</CardTitle>
      <CardBody>
        {ast ? (
          <CodeBlock>
            <CodeBlockCode>
              {JSON.stringify(ast, null, 2)}
            </CodeBlockCode>
          </CodeBlock>
        ) : (
          <p style={{ color: 'var(--pf-v5-global--Color--200)' }}>
            Click a palette button to generate an AST
          </p>
        )}
      </CardBody>
    </Card>
  );
}

