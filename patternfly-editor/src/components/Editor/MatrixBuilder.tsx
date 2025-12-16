/**
 * Matrix Builder Modal Component
 * 
 * Allows users to create matrices by selecting size and delimiter type
 */

import { useState, useEffect, useCallback } from 'react';
import { Button } from '@patternfly/react-core';
import type { EditorNode } from '../../types/ast';

interface MatrixBuilderProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (ast: EditorNode) => void;
}

type DelimiterType = 'bmatrix' | 'pmatrix' | 'vmatrix' | 'Bmatrix';

const delimiterOptions: Array<{ value: DelimiterType; label: string }> = [
  { value: 'bmatrix', label: '[ ]' },
  { value: 'pmatrix', label: '( )' },
  { value: 'vmatrix', label: '| |' },
  { value: 'Bmatrix', label: '{ }' },
];

export function MatrixBuilder({ isOpen, onClose, onCreate }: MatrixBuilderProps) {
  const [rows, setRows] = useState(2);
  const [cols, setCols] = useState(2);
  const [delimiter, setDelimiter] = useState<DelimiterType>('bmatrix');
  const [hoveredSize, setHoveredSize] = useState<{ rows: number; cols: number } | null>(null);

  // Reset to defaults when modal opens
  useEffect(() => {
    if (isOpen) {
      setRows(2);
      setCols(2);
      setDelimiter('bmatrix');
      setHoveredSize(null);
    }
  }, [isOpen]);

  const handleCreate = useCallback(() => {
    const ast = createMatrixAST(rows, cols, delimiter);
    onCreate(ast);
    onClose();
  }, [rows, cols, delimiter, onCreate, onClose]);

  // Handle Enter key press
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

  const handleGridHover = (r: number, c: number) => {
    setHoveredSize({ rows: r + 1, cols: c + 1 });
  };

  const handleGridClick = (r: number, c: number) => {
    setRows(r + 1);
    setCols(c + 1);
    setHoveredSize(null);
  };

  const displaySize = hoveredSize || { rows, cols };

  if (!isOpen) return null;

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div style={styles.header}>
          <h3 style={styles.title}>Matrix Builder</h3>
          <button style={styles.closeBtn} onClick={onClose}>×</button>
        </div>

        {/* Grid Selector */}
        <div style={styles.gridContainer}>
          <div style={styles.grid}>
            {Array.from({ length: 36 }, (_, i) => {
              const r = Math.floor(i / 6);
              const c = i % 6;
              const isInSelectedRegion = r < rows && c < cols;
              const isInHoveredRegion = hoveredSize && r < hoveredSize.rows && c < hoveredSize.cols;

              return (
                <div
                  key={i}
                  style={{
                    ...styles.cell,
                    backgroundColor: isInSelectedRegion ? '#667eea' : isInHoveredRegion ? '#c5cae9' : 'white',
                    borderColor: isInSelectedRegion ? '#667eea' : '#ddd',
                  }}
                  onMouseEnter={() => handleGridHover(r, c)}
                  onMouseLeave={() => setHoveredSize(null)}
                  onClick={() => handleGridClick(r, c)}
                />
              );
            })}
          </div>
          <div style={styles.sizeDisplay}>
            {displaySize.rows} × {displaySize.cols}
          </div>
        </div>

        {/* Size Inputs */}
        <div style={styles.inputRow}>
          <div style={styles.inputGroup}>
            <label style={styles.label}>Rows</label>
            <input
              type="number"
              value={rows}
              min={1}
              max={10}
              onChange={(e) => setRows(Math.max(1, Math.min(10, parseInt(e.target.value) || 1)))}
              style={styles.input}
            />
          </div>
          <div style={styles.inputGroup}>
            <label style={styles.label}>Cols</label>
            <input
              type="number"
              value={cols}
              min={1}
              max={10}
              onChange={(e) => setCols(Math.max(1, Math.min(10, parseInt(e.target.value) || 1)))}
              style={styles.input}
            />
          </div>
        </div>

        {/* Delimiter Selector */}
        <div style={styles.delimiterRow}>
          <label style={styles.label}>Brackets</label>
          <div style={styles.delimiterBtns}>
            {delimiterOptions.map((opt) => (
              <button
                key={opt.value}
                style={{
                  ...styles.delimiterBtn,
                  backgroundColor: delimiter === opt.value ? '#667eea' : 'white',
                  color: delimiter === opt.value ? 'white' : '#333',
                  borderColor: delimiter === opt.value ? '#667eea' : '#ccc',
                }}
                onClick={() => setDelimiter(opt.value)}
              >
                {opt.label}
              </button>
            ))}
          </div>
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
    minWidth: '320px',
    maxWidth: '400px',
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
  gridContainer: {
    textAlign: 'center',
    marginBottom: '16px',
  },
  grid: {
    display: 'inline-grid',
    gridTemplateColumns: 'repeat(6, 28px)',
    gridTemplateRows: 'repeat(6, 28px)',
    gap: '2px',
    padding: '8px',
    background: '#f5f5f5',
    borderRadius: '4px',
  },
  cell: {
    width: '28px',
    height: '28px',
    border: '1px solid #ddd',
    cursor: 'pointer',
    transition: 'all 0.1s',
    borderRadius: '2px',
  },
  sizeDisplay: {
    marginTop: '8px',
    color: '#667eea',
    fontWeight: 600,
    fontSize: '14px',
  },
  inputRow: {
    display: 'flex',
    gap: '12px',
    marginBottom: '16px',
  },
  inputGroup: {
    flex: 1,
  },
  label: {
    display: 'block',
    fontSize: '12px',
    fontWeight: 500,
    color: '#666',
    marginBottom: '4px',
  },
  input: {
    width: '100%',
    padding: '6px 10px',
    border: '1px solid #ccc',
    borderRadius: '4px',
    fontSize: '14px',
    boxSizing: 'border-box',
  },
  delimiterRow: {
    marginBottom: '20px',
  },
  delimiterBtns: {
    display: 'flex',
    gap: '6px',
  },
  delimiterBtn: {
    flex: 1,
    padding: '8px 12px',
    border: '1px solid #ccc',
    borderRadius: '4px',
    fontSize: '16px',
    fontWeight: 600,
    cursor: 'pointer',
    transition: 'all 0.15s',
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

/**
 * Create a matrix AST node
 */
function createMatrixAST(rows: number, cols: number, delimiter: DelimiterType): EditorNode {
  const totalElements = rows * cols;
  const args: EditorNode[] = [];

  let placeholderId = 0;  // Use sequential IDs like the static version
  for (let i = 0; i < totalElements; i++) {
    const row = Math.floor(i / cols) + 1;
    const col = (i % cols) + 1;
    args.push({
      Placeholder: {
        id: placeholderId++,
        hint: `a${row}${col}`,
      },
    });
  }

  let opName: string;
  if (delimiter === 'pmatrix') {
    opName = 'PMatrix';
  } else if (delimiter === 'vmatrix') {
    opName = 'VMatrix';
  } else if (delimiter === 'Bmatrix') {
    opName = 'BMatrix';
  } else {
    opName = 'Matrix';
  }

  return {
    Operation: {
      name: opName,
      args: [
        { Const: String(rows) },
        { Const: String(cols) },
        { List: args },
      ],
    },
  };
}
