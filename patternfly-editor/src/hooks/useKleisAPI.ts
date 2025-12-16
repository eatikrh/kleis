/**
 * React hooks for Kleis API integration
 */

import { useState, useEffect, useCallback } from 'react';
import type { EditorNode } from '../types/ast';
import { 
  renderTypst, 
  typeCheck, 
  renderKleis, 
  healthCheck,
  type RenderTypstResponse,
  type TypeCheckResponse,
  type PlaceholderPosition,
  type ArgumentSlot,
  type ArgumentBoundingBox,
} from '../api/kleis';

// ─────────────────────────────────────────────────────────────
// useServerStatus - Check if backend is available
// ─────────────────────────────────────────────────────────────

export function useServerStatus() {
  const [isConnected, setIsConnected] = useState(false);
  const [checking, setChecking] = useState(true);

  const checkConnection = useCallback(async () => {
    setChecking(true);
    const connected = await healthCheck();
    setIsConnected(connected);
    setChecking(false);
  }, []);

  useEffect(() => {
    checkConnection();
    // Check every 30 seconds (reduced from 10 to lower CPU usage)
    const interval = setInterval(checkConnection, 30000);
    return () => clearInterval(interval);
  }, [checkConnection]);

  return { isConnected, checking, checkConnection };
}

// ─────────────────────────────────────────────────────────────
// useRenderTypst - Render AST to SVG
// ─────────────────────────────────────────────────────────────

export function useRenderTypst() {
  const [svg, setSvg] = useState<string>('');
  const [placeholders, setPlaceholders] = useState<PlaceholderPosition[]>([]);
  const [argumentSlots, setArgumentSlots] = useState<ArgumentSlot[]>([]);
  const [argumentBoundingBoxes, setArgumentBoundingBoxes] = useState<ArgumentBoundingBox[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const render = useCallback(async (ast: EditorNode) => {
    setLoading(true);
    setError(null);
    
    const result: RenderTypstResponse = await renderTypst(ast);
    
    if (result.success && result.svg) {
      setSvg(result.svg);
      setPlaceholders(result.placeholders || []);  // Backend uses "placeholders" not "placeholder_positions"
      setArgumentSlots(result.argument_slots || []);
      setArgumentBoundingBoxes(result.argument_bounding_boxes || []);
    } else {
      setError(result.error || 'Render failed');
      setSvg('');
      setPlaceholders([]);
      setArgumentSlots([]);
      setArgumentBoundingBoxes([]);
    }
    
    setLoading(false);
    return result;
  }, []);

  const clear = useCallback(() => {
    setSvg('');
    setPlaceholders([]);
    setArgumentSlots([]);
    setArgumentBoundingBoxes([]);
    setError(null);
  }, []);

  return { svg, placeholders, argumentSlots, argumentBoundingBoxes, loading, error, render, clear };
}

// ─────────────────────────────────────────────────────────────
// useTypeCheck - Type check AST
// ─────────────────────────────────────────────────────────────

export function useTypeCheck() {
  const [type, setType] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const check = useCallback(async (ast: EditorNode) => {
    setLoading(true);
    setError(null);
    
    const result: TypeCheckResponse = await typeCheck(ast);
    
    if (result.success && result.type_name) {
      setType(result.type_name);
    } else {
      setError(result.error || 'Type check failed');
      setType(null);
    }
    
    setLoading(false);
    return result;
  }, []);

  const clear = useCallback(() => {
    setType(null);
    setError(null);
  }, []);

  return { type, loading, error, check, clear };
}

// ─────────────────────────────────────────────────────────────
// useKleisOutput - Render to Kleis syntax
// ─────────────────────────────────────────────────────────────

export function useKleisOutput() {
  const [output, setOutput] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const render = useCallback(async (ast: EditorNode) => {
    setLoading(true);
    setError(null);
    
    const result = await renderKleis(ast);
    
    if (result.success && result.output) {
      setOutput(result.output);
    } else {
      setError(result.error || 'Render failed');
      setOutput('');
    }
    
    setLoading(false);
    return result;
  }, []);

  const clear = useCallback(() => {
    setOutput('');
    setError(null);
  }, []);

  return { output, loading, error, render, clear };
}

