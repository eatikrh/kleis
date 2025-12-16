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
    // Check every 10 seconds
    const interval = setInterval(checkConnection, 10000);
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
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const render = useCallback(async (ast: EditorNode) => {
    setLoading(true);
    setError(null);
    
    const result: RenderTypstResponse = await renderTypst(ast);
    
    if (result.success && result.svg) {
      setSvg(result.svg);
      setPlaceholders(result.placeholder_positions || []);
    } else {
      setError(result.error || 'Render failed');
      setSvg('');
      setPlaceholders([]);
    }
    
    setLoading(false);
    return result;
  }, []);

  const clear = useCallback(() => {
    setSvg('');
    setPlaceholders([]);
    setError(null);
  }, []);

  return { svg, placeholders, loading, error, render, clear };
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
    
    if (result.success && result.type) {
      setType(result.type);
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

