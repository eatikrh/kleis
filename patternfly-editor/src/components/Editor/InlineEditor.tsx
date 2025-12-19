/**
 * Inline editor component for editing placeholders directly in SVG
 */

import { useEffect, useRef } from 'react';

interface InlineEditorProps {
  active: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
  value: string;
  onCommit: (value: string) => void;
  onCancel: () => void;
  onAppend: (text: string) => void;
}

export function InlineEditor({
  active,
  x,
  y,
  width,
  height,
  value,
  onCommit,
  onCancel,
  onAppend,
}: InlineEditorProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (active && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [active]);

  useEffect(() => {
    // Expose append function to window for palette buttons
    if (active) {
      (window as any).appendToInlineEditor = onAppend;
    } else {
      delete (window as any).appendToInlineEditor;
    }
    return () => {
      delete (window as any).appendToInlineEditor;
    };
  }, [active, onAppend]);

  if (!active) return null;

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      onCommit(inputRef.current?.value || '');
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onCancel();
    } else if (e.key === 'Tab') {
      e.preventDefault();
      onCommit(inputRef.current?.value || '');
      // TODO: Focus next placeholder
    }
  };

  return (
    <foreignObject
      x={x}
      y={y}
      width={Math.max(width, 200)}
      height={Math.max(height, 40)}
      style={{ overflow: 'visible' }}
    >
      <input
        ref={inputRef}
        type="text"
        className="inline-edit-input"
        defaultValue={value}
        autoComplete="off"
        spellCheck={false}
        placeholder="Type or click symbols..."
        onKeyDown={handleKeyDown}
        onBlur={() => {
          // Commit on blur (with small delay to allow button clicks)
          setTimeout(() => {
            if (inputRef.current) {
              onCommit(inputRef.current.value);
            }
          }, 200);
        }}
        style={{
          width: '100%',
          height: '100%',
          padding: '4px 8px',
          border: '2px solid #667eea',
          borderRadius: '4px',
          fontSize: '16px',
          fontFamily: 'inherit',
          outline: 'none',
          boxSizing: 'border-box',
        }}
      />
    </foreignObject>
  );
}







