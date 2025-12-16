/**
 * AST Templates for Palette Buttons
 * 
 * These templates define the AST structure generated when a user
 * clicks a palette button. They match the astTemplates in static/index.html.
 * 
 * IMPORTANT: This is the source of truth for template definitions.
 * To add a new palette button:
 * 1. Add the template here
 * 2. Add the button to the appropriate tab in buttonConfigs.ts
 */

import type { EditorNode } from '../../types/ast';
import { placeholder, operation } from '../../types/ast';

/**
 * AST template definitions
 * 
 * Each template defines the AST structure for an operation.
 * Placeholders use incremental IDs starting from 0.
 */
export const astTemplates: Record<string, EditorNode> = {
  // ─────────────────────────────────────────────────────────────
  // Basic Operations
  // ─────────────────────────────────────────────────────────────
  
  fraction: operation('scalar_divide', [
    placeholder(0, 'numerator'),
    placeholder(1, 'denominator'),
  ]),
  
  power: operation('power', [
    placeholder(0, 'base'),
    placeholder(1, 'exponent'),
  ]),
  
  subscript: operation('sub', [
    placeholder(0, 'base'),
    placeholder(1, 'subscript'),
  ]),
  
  sqrt: operation('sqrt', [
    placeholder(0, 'radicand'),
  ]),
  
  nthroot: operation('nth_root', [
    placeholder(0, 'index'),
    placeholder(1, 'radicand'),
  ]),
  
  // ─────────────────────────────────────────────────────────────
  // Trigonometric Functions
  // ─────────────────────────────────────────────────────────────
  
  sin: operation('sin', [placeholder(0, 'x')]),
  cos: operation('cos', [placeholder(0, 'x')]),
  tan: operation('tan', [placeholder(0, 'x')]),
  
  // ─────────────────────────────────────────────────────────────
  // Calculus
  // ─────────────────────────────────────────────────────────────
  
  integral: operation('integral', [
    placeholder(0, 'lower'),
    placeholder(1, 'upper'),
    placeholder(2, 'integrand'),
    placeholder(3, 'var'),
  ]),
  
  derivative: operation('derivative', [
    placeholder(0, 'expr'),
    placeholder(1, 'var'),
  ]),
  
  partial: operation('partial', [
    placeholder(0, 'expr'),
    placeholder(1, 'var'),
  ]),
  
  sum: operation('sum', [
    placeholder(0, 'lower'),
    placeholder(1, 'upper'),
    placeholder(2, 'body'),
  ]),
  
  product: operation('product', [
    placeholder(0, 'lower'),
    placeholder(1, 'upper'),
    placeholder(2, 'body'),
  ]),
  
  limit: operation('limit', [
    placeholder(0, 'var'),
    placeholder(1, 'to'),
    placeholder(2, 'expr'),
  ]),
};

/**
 * Get a template by name, cloning it to avoid mutation
 */
export function getTemplate(name: string): EditorNode | undefined {
  const template = astTemplates[name];
  if (!template) return undefined;
  return JSON.parse(JSON.stringify(template));
}

