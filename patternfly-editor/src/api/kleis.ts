/**
 * Kleis API Client
 * 
 * Connects to the Rust backend server for:
 * - Rendering equations as SVG (via Typst)
 * - Type checking expressions
 * - Verification and satisfiability checking
 */

import type { EditorNode } from '../types/ast';

const API_BASE = 'http://localhost:3000/api';

// ─────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────

export interface ArgumentSlot {
  id: string;
  path: number[];
  hint: string;
  is_placeholder: boolean;
  role?: string;
}

export interface ArgumentBoundingBox {
  arg_index: number;
  node_id: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RenderTypstResponse {
  success: boolean;
  svg?: string;
  placeholders?: PlaceholderPosition[];  // Backend uses "placeholders" not "placeholder_positions"
  argument_slots?: ArgumentSlot[];
  argument_bounding_boxes?: ArgumentBoundingBox[];
  semantic_boxes?: SemanticBox[];
  error?: string;
}

export interface PlaceholderPosition {
  id: number;  // Backend returns numeric IDs
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface SemanticBox {
  uuid: string;
  x: number;
  y: number;
  width: number;
  height: number;
  kind: string;
}

export interface TypeCheckResponse {
  success: boolean;
  type_name?: string;  // Server returns 'type_name', not 'type'
  constraints?: string[];
  error?: string;
  suggestion?: string;
}

export interface RenderASTResponse {
  success: boolean;
  output?: string;
  error?: string;
}

export interface VerifyResponse {
  success: boolean;
  result?: string; // "valid", "invalid", "unknown", "error", "incomplete"
  kleis_syntax?: string;
  counterexample?: string;
  error?: string;
}

export interface CheckSatResponse {
  success: boolean;
  result?: string; // "satisfiable", "unsatisfiable", "unknown", "error", "incomplete"
  kleis_syntax?: string;
  example?: string;
  error?: string;
}

// ─────────────────────────────────────────────────────────────
// API Functions
// ─────────────────────────────────────────────────────────────

/**
 * Render AST to SVG using Typst
 * Returns SVG string and placeholder positions for overlay
 */
export async function renderTypst(ast: EditorNode): Promise<RenderTypstResponse> {
  try {
    const response = await fetch(`${API_BASE}/render_typst`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ast }),
    });
    
    if (!response.ok) {
      return { success: false, error: `HTTP ${response.status}` };
    }
    
    return await response.json();
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Network error' 
    };
  }
}

/**
 * Type check an AST expression
 */
export async function typeCheck(ast: EditorNode): Promise<TypeCheckResponse> {
  try {
    const response = await fetch(`${API_BASE}/type_check`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ast }),
    });
    
    if (!response.ok) {
      return { success: false, error: `HTTP ${response.status}` };
    }
    
    return await response.json();
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Network error' 
    };
  }
}

/**
 * Render AST to various formats (latex, html, unicode, kleis)
 */
export async function renderAST(
  ast: EditorNode, 
  format: 'latex' | 'html' | 'unicode' | 'kleis' = 'latex'
): Promise<RenderASTResponse> {
  try {
    const response = await fetch(`${API_BASE}/render_ast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ast, format }),
    });
    
    if (!response.ok) {
      return { success: false, error: `HTTP ${response.status}` };
    }
    
    return await response.json();
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Network error' 
    };
  }
}

/**
 * Render AST to Kleis syntax
 */
export async function renderKleis(ast: EditorNode): Promise<RenderASTResponse> {
  try {
    const response = await fetch(`${API_BASE}/render_kleis`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ast }),
    });
    
    if (!response.ok) {
      return { success: false, error: `HTTP ${response.status}` };
    }
    
    return await response.json();
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Network error' 
    };
  }
}

/**
 * Verify an expression using Z3
 */
export async function verify(ast: EditorNode): Promise<VerifyResponse> {
  try {
    const response = await fetch(`${API_BASE}/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ast }),
    });
    
    if (!response.ok) {
      return { success: false, error: `HTTP ${response.status}` };
    }
    
    return await response.json();
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Network error' 
    };
  }
}

/**
 * Check satisfiability of an expression using Z3
 */
export async function checkSat(ast: EditorNode): Promise<CheckSatResponse> {
  try {
    const response = await fetch(`${API_BASE}/check_sat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ast }),
    });
    
    if (!response.ok) {
      return { success: false, error: `HTTP ${response.status}` };
    }
    
    return await response.json();
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Network error' 
    };
  }
}

/**
 * Check if the server is available
 */
export async function healthCheck(): Promise<boolean> {
  try {
    const response = await fetch('http://localhost:3000/health');
    return response.ok;
  } catch {
    return false;
  }
}
