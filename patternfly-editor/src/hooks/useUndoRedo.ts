/**
 * Undo/Redo hook for AST editing
 */

import { useState, useCallback } from 'react';
import type { EditorNode } from '../types/ast';

const MAX_HISTORY = 50;

export function useUndoRedo(initialAST: EditorNode | null) {
  const [undoStack, setUndoStack] = useState<EditorNode[]>([]);
  const [redoStack, setRedoStack] = useState<EditorNode[]>([]);
  const [currentAST, setCurrentAST] = useState<EditorNode | null>(initialAST);

  const saveState = useCallback((ast: EditorNode | null) => {
    if (ast === null) return;

    setUndoStack((prev) => {
      const newStack = [...prev, JSON.parse(JSON.stringify(ast))];
      // Limit history size
      if (newStack.length > MAX_HISTORY) {
        return newStack.slice(-MAX_HISTORY);
      }
      return newStack;
    });

    // Clear redo stack when new action is performed
    setRedoStack([]);
  }, []);

  const undo = useCallback(() => {
    if (undoStack.length === 0) return false;

    setRedoStack((prev) => {
      if (currentAST) {
        return [...prev, JSON.parse(JSON.stringify(currentAST))];
      }
      return prev;
    });

    const previousAST = undoStack[undoStack.length - 1];
    setUndoStack((prev) => prev.slice(0, -1));
    setCurrentAST(previousAST);

    return true;
  }, [undoStack, currentAST]);

  const redo = useCallback(() => {
    if (redoStack.length === 0) return false;

    setUndoStack((prev) => {
      if (currentAST) {
        return [...prev, JSON.parse(JSON.stringify(currentAST))];
      }
      return prev;
    });

    const nextAST = redoStack[redoStack.length - 1];
    setRedoStack((prev) => prev.slice(0, -1));
    setCurrentAST(nextAST);

    return true;
  }, [redoStack, currentAST]);

  const updateAST = useCallback((ast: EditorNode | null) => {
    saveState(currentAST);
    setCurrentAST(ast);
  }, [currentAST, saveState]);

  return {
    currentAST,
    updateAST,
    undo,
    redo,
    canUndo: undoStack.length > 0,
    canRedo: redoStack.length > 0,
    undoCount: undoStack.length,
    redoCount: redoStack.length,
  };
}





