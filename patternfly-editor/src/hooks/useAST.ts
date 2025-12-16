/**
 * useAST Hook
 * 
 * Manages the AST state, rendering, and type checking.
 * Connects to the Rust backend for SVG rendering.
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import type { EditorNode } from '../types/ast';
import { renderToSvg, typeCheck, checkHealth, type RenderResult, type TypeCheckResult, type PlaceholderInfo } from '../api/kleis';

interface UseASTState {
  currentAST: EditorNode | null;
  svgContent: string;
  placeholders: PlaceholderInfo[];
  typeInfo: TypeCheckResult | null;
  isRendering: boolean;
  renderError: string | null;
  backendAvailable: boolean;
  history: EditorNode[];
  historyIndex: number;
}

interface UseASTActions {
  setAST: (ast: EditorNode) => void;
  updatePlaceholder: (id: number, value: EditorNode) => void;
  undo: () => void;
  redo: () => void;
  canUndo: boolean;
  canRedo: boolean;
  refreshRender: () => void;
}

export function useAST(): UseASTState & UseASTActions {
  const [currentAST, setCurrentAST] = useState<EditorNode | null>(null);
  const [svgContent, setSvgContent] = useState<string>('');
  const [placeholders, setPlaceholders] = useState<PlaceholderInfo[]>([]);
  const [typeInfo, setTypeInfo] = useState<TypeCheckResult | null>(null);
  const [isRendering, setIsRendering] = useState(false);
  const [renderError, setRenderError] = useState<string | null>(null);
  const [backendAvailable, setBackendAvailable] = useState(false);
  
  // History for undo/redo
  const [history, setHistory] = useState<EditorNode[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  
  // Debounce render requests
  const renderTimeout = useRef<NodeJS.Timeout>();

  // Check backend availability on mount
  useEffect(() => {
    const checkBackend = async () => {
      const available = await checkHealth();
      setBackendAvailable(available);
      if (!available) {
        console.warn('Kleis backend not available at http://localhost:3030');
      }
    };
    checkBackend();
    
    // Recheck every 5 seconds
    const interval = setInterval(checkBackend, 5000);
    return () => clearInterval(interval);
  }, []);

  // Render AST when it changes
  useEffect(() => {
    if (!currentAST || !backendAvailable) return;

    // Debounce to avoid too many requests
    if (renderTimeout.current) {
      clearTimeout(renderTimeout.current);
    }

    renderTimeout.current = setTimeout(async () => {
      setIsRendering(true);
      setRenderError(null);

      try {
        // Render to SVG
        const result: RenderResult = await renderToSvg(currentAST);
        setSvgContent(result.svg);
        setPlaceholders(result.placeholders || []);

        // Type check
        const typeResult = await typeCheck(currentAST);
        setTypeInfo(typeResult);
      } catch (error) {
        console.error('Render failed:', error);
        setRenderError(error instanceof Error ? error.message : 'Render failed');
        setSvgContent('');
        setPlaceholders([]);
      } finally {
        setIsRendering(false);
      }
    }, 100);

    return () => {
      if (renderTimeout.current) {
        clearTimeout(renderTimeout.current);
      }
    };
  }, [currentAST, backendAvailable]);

  // Set AST with history tracking
  const setAST = useCallback((ast: EditorNode) => {
    setCurrentAST(ast);
    
    // Add to history
    setHistory(prev => {
      const newHistory = prev.slice(0, historyIndex + 1);
      return [...newHistory, ast];
    });
    setHistoryIndex(prev => prev + 1);
  }, [historyIndex]);

  // Update a placeholder value in the AST
  const updatePlaceholder = useCallback((id: number, value: EditorNode) => {
    if (!currentAST) return;

    const updateNode = (node: EditorNode): EditorNode => {
      if ('Placeholder' in node && node.Placeholder.id === id) {
        return { ...node, Placeholder: { ...node.Placeholder, value } };
      }
      if ('Operation' in node) {
        return {
          Operation: {
            ...node.Operation,
            args: node.Operation.args.map(updateNode),
          },
        };
      }
      if ('List' in node) {
        return { List: node.List.map(updateNode) };
      }
      return node;
    };

    const updated = updateNode(currentAST);
    setAST(updated);
  }, [currentAST, setAST]);

  // Undo
  const undo = useCallback(() => {
    if (historyIndex > 0) {
      setHistoryIndex(prev => prev - 1);
      setCurrentAST(history[historyIndex - 1]);
    }
  }, [history, historyIndex]);

  // Redo
  const redo = useCallback(() => {
    if (historyIndex < history.length - 1) {
      setHistoryIndex(prev => prev + 1);
      setCurrentAST(history[historyIndex + 1]);
    }
  }, [history, historyIndex]);

  // Force re-render
  const refreshRender = useCallback(() => {
    if (currentAST && backendAvailable) {
      // Trigger re-render by setting the same AST
      setCurrentAST({ ...currentAST });
    }
  }, [currentAST, backendAvailable]);

  return {
    // State
    currentAST,
    svgContent,
    placeholders,
    typeInfo,
    isRendering,
    renderError,
    backendAvailable,
    history,
    historyIndex,
    
    // Actions
    setAST,
    updatePlaceholder,
    undo,
    redo,
    canUndo: historyIndex > 0,
    canRedo: historyIndex < history.length - 1,
    refreshRender,
  };
}

