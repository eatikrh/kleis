import { Button, Tooltip } from '@patternfly/react-core';
import type { ButtonConfig } from './buttonConfigs';
import type { EditorNode } from '../../types/ast';
import { cloneAndRenumber } from '../../types/ast';
import { getTemplate } from './astTemplates';

interface PaletteButtonProps {
  config: ButtonConfig;
  onInsert: (ast: EditorNode) => void;
}

/**
 * A single palette button that generates an AST when clicked
 */
export function PaletteButton({ config, onInsert }: PaletteButtonProps) {
  const handleClick = () => {
    const template = getTemplate(config.template);
    if (template) {
      // Clone and renumber placeholders
      const ast = cloneAndRenumber(template);
      onInsert(ast);
    } else {
      console.warn(`No template found for: ${config.template}`);
    }
  };

  return (
    <Tooltip content={config.tooltip}>
      <Button
        variant="secondary"
        onClick={handleClick}
        style={{
          fontFamily: 'serif',
          fontSize: '1.1rem',
          minWidth: '48px',
          padding: '8px 12px',
        }}
      >
        {config.symbol || config.label}
      </Button>
    </Tooltip>
  );
}

