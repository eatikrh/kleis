/**
 * Piecewise Function Builder Modal Component
 * 
 * Allows users to create piecewise functions by selecting number of cases
 */

import { useState, useEffect, useCallback } from 'react';
import type { EditorNode } from '../../types/ast';

interface PiecewiseBuilderProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (ast: EditorNode) => void;
}

export function PiecewiseBuilder({ isOpen, onClose, onCreate }: PiecewiseBuilderProps) {
  const [cases, setCases] = useState(2);

  // Reset to defaults when modal opens
  useEffect(() => {
    if (isOpen) {
      setCases(2);
    }
  }, [isOpen]);

  const handleCreate = useCallback(() => {
    const ast = createPiecewiseAST(cases);
    onCreate(ast);
    onClose();
  }, [cases, onCreate, onClose]);

  // Handle Enter/Escape key press
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        e.stopPropagation();
        handleCreate();
      } else if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, handleCreate, onClose]);

  if (!isOpen) return null;

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div style={styles.header}>
          <h3 style={styles.title}>Piecewise Function</h3>
          <button style={styles.closeBtn} onClick={onClose}>×</button>
        </div>

        {/* Cases Input */}
        <div style={styles.inputRow}>
          <label style={styles.label}>Number of cases</label>
          <div style={styles.numberInput}>
            <button 
              style={styles.numBtn}
              onClick={() => setCases(Math.max(2, cases - 1))}
            >−</button>
            <input
              type="number"
              value={cases}
              min={2}
              max={10}
              onChange={(e) => setCases(Math.max(2, Math.min(10, parseInt(e.target.value) || 2)))}
              style={styles.input}
            />
            <button 
              style={styles.numBtn}
              onClick={() => setCases(Math.min(10, cases + 1))}
            >+</button>
          </div>
        </div>

        {/* Preview */}
        <div style={styles.preview}>
          <div style={styles.previewLabel}>Preview</div>
          <pre style={styles.previewContent}>{generatePreview(cases)}</pre>
        </div>

        {/* Buttons */}
        <div style={styles.buttons}>
          <button style={styles.cancelBtn} onClick={onClose}>Cancel</button>
          <button style={styles.createBtn} onClick={handleCreate}>Create</button>
        </div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 10000,
  },
  modal: {
    background: 'white',
    padding: '20px',
    borderRadius: '8px',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.3)',
    minWidth: '300px',
    maxWidth: '380px',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  title: {
    margin: 0,
    fontSize: '18px',
    fontWeight: 600,
    color: '#333',
  },
  closeBtn: {
    background: 'none',
    border: 'none',
    fontSize: '24px',
    cursor: 'pointer',
    color: '#666',
    padding: '0 4px',
  },
  inputRow: {
    marginBottom: '16px',
  },
  label: {
    display: 'block',
    fontSize: '12px',
    fontWeight: 500,
    color: '#666',
    marginBottom: '6px',
  },
  numberInput: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
  },
  numBtn: {
    width: '32px',
    height: '32px',
    border: '1px solid #ccc',
    borderRadius: '4px',
    background: '#f5f5f5',
    fontSize: '18px',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  input: {
    width: '60px',
    padding: '6px 10px',
    border: '1px solid #ccc',
    borderRadius: '4px',
    fontSize: '14px',
    textAlign: 'center',
  },
  preview: {
    marginBottom: '20px',
  },
  previewLabel: {
    fontSize: '12px',
    fontWeight: 500,
    color: '#666',
    marginBottom: '6px',
  },
  previewContent: {
    margin: 0,
    padding: '12px',
    background: '#f5f5f5',
    borderRadius: '4px',
    fontFamily: 'monospace',
    fontSize: '13px',
    whiteSpace: 'pre-wrap',
    color: '#333',
  },
  buttons: {
    display: 'flex',
    gap: '10px',
    justifyContent: 'flex-end',
  },
  cancelBtn: {
    padding: '8px 16px',
    background: '#f0f0f0',
    border: 'none',
    borderRadius: '4px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
  },
  createBtn: {
    padding: '8px 16px',
    background: '#4CAF50',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
  },
};

function generatePreview(cases: number): string {
  let text = 'f(x) = {\n';
  for (let i = 1; i <= cases; i++) {
    text += `  expr${i}  if cond${i}`;
    if (i < cases) text += '\n';
  }
  return text;
}

/**
 * Create a piecewise function AST node
 * Uses sequential IDs starting from 0, like the static version
 */
function createPiecewiseAST(cases: number): EditorNode {
  let nextId = 0;
  
  const exprs: EditorNode[] = [];
  const conds: EditorNode[] = [];
  
  for (let i = 0; i < cases; i++) {
    exprs.push({
      Placeholder: {
        id: nextId++,
        hint: `expr${i + 1}`,
      },
    });
    conds.push({
      Placeholder: {
        id: nextId++,
        hint: `cond${i + 1}`,
      },
    });
  }
  
  return {
    Operation: {
      name: 'Piecewise',
      args: [
        { Const: String(cases) },
        { List: exprs },
        { List: conds },
      ],
    },
  };
}



