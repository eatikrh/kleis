/**
 * AST Templates for Palette Buttons
 * 
 * These templates define the AST structure generated when a user
 * clicks a palette button. They match the astTemplates in static/index.html.
 * 
 * IMPORTANT: This is the source of truth for template definitions.
 */

import type { EditorNode } from '../../types/ast';
import { placeholder, operation } from '../../types/ast';

// Helper for creating Const nodes
function constNode(value: string): EditorNode {
  return { Const: value };  // Backend expects { Const: "2" } format, not { Const: { value: "2" } }
}

// Helper for creating Object nodes
function objectNode(value: string): EditorNode {
  return { Object: value };
}

// Helper for creating List nodes
function listNode(items: EditorNode[]): EditorNode {
  return { List: items };
}

// Helper for creating tensor operations with metadata
function tensor(
  symbol: string | EditorNode,
  indices: EditorNode[],
  indexStructure: ('up' | 'down')[]
): EditorNode {
  const symbolArg = typeof symbol === 'string' ? objectNode(symbol) : symbol;
  return {
    Operation: {
      name: 'tensor',
      args: [symbolArg, ...indices],
      kind: 'tensor',
      metadata: { indexStructure },
    },
  };
}

/**
 * AST template definitions - matches static/index.html exactly
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
  
  sqrt: operation('sqrt', [placeholder(0, 'radicand')]),
  
  nthroot: operation('nth_root', [
    placeholder(0, 'index'),
    placeholder(1, 'radicand'),
  ]),
  
  binomial: operation('binomial', [
    placeholder(0, 'n'),
    placeholder(1, 'k'),
  ]),
  
  factorial: operation('factorial', [placeholder(0, 'n')]),
  floor: operation('floor', [placeholder(0, 'x')]),
  ceiling: operation('ceiling', [placeholder(0, 'x')]),
  
  // Arithmetic
  equals: operation('equals', [placeholder(0, 'left'), placeholder(1, 'right')]),
  plus: operation('plus', [placeholder(0, 'left'), placeholder(1, 'right')]),
  minus: operation('minus', [placeholder(0, 'left'), placeholder(1, 'right')]),
  times: operation('scalar_multiply', [placeholder(0, 'left'), placeholder(1, 'right')]),
  matrix_multiply: operation('multiply', [placeholder(0, 'left'), placeholder(1, 'right')]),
  
  // Comparisons
  less_than: operation('less_than', [placeholder(0, 'left'), placeholder(1, 'right')]),
  greater_than: operation('greater_than', [placeholder(0, 'left'), placeholder(1, 'right')]),
  leq: operation('leq', [placeholder(0, 'left'), placeholder(1, 'right')]),
  geq: operation('geq', [placeholder(0, 'left'), placeholder(1, 'right')]),
  neq: operation('neq', [placeholder(0, 'left'), placeholder(1, 'right')]),
  approx: operation('approx', [placeholder(0, 'left'), placeholder(1, 'right')]),
  
  // Logic
  logical_and: operation('logical_and', [placeholder(0, 'left'), placeholder(1, 'right')]),
  logical_or: operation('logical_or', [placeholder(0, 'left'), placeholder(1, 'right')]),
  logical_not: operation('logical_not', [placeholder(0, 'arg')]),
  
  // ─────────────────────────────────────────────────────────────
  // Calculus
  // ─────────────────────────────────────────────────────────────
  
  integral: operation('int_bounds', [
    placeholder(0, 'integrand'),
    placeholder(1, 'lower'),
    placeholder(2, 'upper'),
    placeholder(3, 'variable'),
  ]),
  
  sum: operation('sum_bounds', [
    placeholder(0, 'body'),
    placeholder(1, 'from'),
    placeholder(2, 'to'),
  ]),
  
  product: operation('prod_bounds', [
    placeholder(0, 'body'),
    placeholder(1, 'from'),
    placeholder(2, 'to'),
  ]),
  
  limit: operation('lim', [
    placeholder(0, 'body'),
    placeholder(1, 'var'),
    placeholder(2, 'target'),
  ]),
  
  partial: operation('d_part', [
    placeholder(0, 'function'),
    placeholder(1, 'variable'),
  ]),
  
  derivative: operation('d_dt', [
    placeholder(0, 'function'),
    placeholder(1, 'variable'),
  ]),
  
  gradient: operation('grad', [placeholder(0, 'function')]),
  
  // ─────────────────────────────────────────────────────────────
  // Trigonometric Functions
  // ─────────────────────────────────────────────────────────────
  
  sin: operation('sin', [placeholder(0, 'argument')]),
  cos: operation('cos', [placeholder(0, 'argument')]),
  tan: operation('tan', [placeholder(0, 'argument')]),
  arcsin: operation('arcsin', [placeholder(0, 'argument')]),
  arccos: operation('arccos', [placeholder(0, 'argument')]),
  arctan: operation('arctan', [placeholder(0, 'argument')]),
  ln: operation('ln', [placeholder(0, 'argument')]),
  log: operation('log', [placeholder(0, 'argument')]),
  exp: operation('exp', [placeholder(0, 'argument')]),
  
  // ─────────────────────────────────────────────────────────────
  // Matrices
  // ─────────────────────────────────────────────────────────────
  
  matrix2x2: operation('Matrix', [
    constNode('2'),
    constNode('2'),
    listNode([
      placeholder(0, 'a11'), placeholder(1, 'a12'),
      placeholder(2, 'a21'), placeholder(3, 'a22'),
    ]),
  ]),
  
  matrix3x3: operation('Matrix', [
    constNode('3'),
    constNode('3'),
    listNode([
      placeholder(0, 'a11'), placeholder(1, 'a12'), placeholder(2, 'a13'),
      placeholder(3, 'a21'), placeholder(4, 'a22'), placeholder(5, 'a23'),
      placeholder(6, 'a31'), placeholder(7, 'a32'), placeholder(8, 'a33'),
    ]),
  ]),
  
  pmatrix2x2: operation('PMatrix', [
    constNode('2'),
    constNode('2'),
    listNode([
      placeholder(0, 'a11'), placeholder(1, 'a12'),
      placeholder(2, 'a21'), placeholder(3, 'a22'),
    ]),
  ]),
  
  vmatrix2x2: operation('VMatrix', [
    constNode('2'),
    constNode('2'),
    listNode([
      placeholder(0, 'a11'), placeholder(1, 'a12'),
      placeholder(2, 'a21'), placeholder(3, 'a22'),
    ]),
  ]),
  
  // ─────────────────────────────────────────────────────────────
  // Vectors
  // ─────────────────────────────────────────────────────────────
  
  vector_bold: operation('vector_bold', [placeholder(0, 'vector')]),
  vector_arrow: operation('vector_arrow', [placeholder(0, 'vector')]),
  dot: operation('dot', [placeholder(0, 'left'), placeholder(1, 'right')]),
  cross: operation('cross', [placeholder(0, 'left'), placeholder(1, 'right')]),
  norm: operation('norm', [placeholder(0, 'vector')]),
  abs: operation('abs', [placeholder(0, 'value')]),
  
  // ─────────────────────────────────────────────────────────────
  // Brackets & Grouping
  // ─────────────────────────────────────────────────────────────
  
  parens: operation('parens', [placeholder(0, 'content')]),
  brackets: operation('brackets', [placeholder(0, 'content')]),
  braces: operation('braces', [placeholder(0, 'content')]),
  angle_brackets: operation('angle_brackets', [placeholder(0, 'content')]),
  
  // ─────────────────────────────────────────────────────────────
  // Accents
  // ─────────────────────────────────────────────────────────────
  
  dot_accent: operation('dot_accent', [placeholder(0, 'variable')]),
  ddot_accent: operation('ddot_accent', [placeholder(0, 'variable')]),
  hat: operation('hat', [placeholder(0, 'variable')]),
  bar: operation('bar', [placeholder(0, 'variable')]),
  tilde: operation('tilde', [placeholder(0, 'variable')]),
  
  // ─────────────────────────────────────────────────────────────
  // Quantum
  // ─────────────────────────────────────────────────────────────
  
  ket: operation('ket', [placeholder(0, 'state')]),
  bra: operation('bra', [placeholder(0, 'state')]),
  inner: operation('inner', [placeholder(0, 'bra'), placeholder(1, 'ket')]),
  outer: operation('outer', [placeholder(0, 'ket'), placeholder(1, 'bra')]),
  commutator: operation('commutator', [placeholder(0, 'A'), placeholder(1, 'B')]),
  expectation: operation('expectation', [placeholder(0, 'operator')]),
  
  // ─────────────────────────────────────────────────────────────
  // Tensors
  // ─────────────────────────────────────────────────────────────
  
  tensor_mixed: operation('index_mixed', [
    placeholder(0, 'base'),
    placeholder(1, 'upper'),
    placeholder(2, 'lower'),
  ]),
  
  subsup: operation('subsup', [
    placeholder(0, 'base'),
    placeholder(1, 'subscript'),
    placeholder(2, 'superscript'),
  ]),
  
  // Metric tensor g_μν
  metric: tensor('g', [placeholder(0, 'idx1'), placeholder(1, 'idx2')], ['down', 'down']),
  
  // Christoffel symbol Γ^λ_μν
  christoffel: tensor('Γ', [
    placeholder(0, 'upper'),
    placeholder(1, 'lower1'),
    placeholder(2, 'lower2'),
  ], ['up', 'down', 'down']),
  
  // Riemann tensor R^ρ_σμν
  riemann: tensor('R', [
    placeholder(0, 'upper'),
    placeholder(1, 'lower1'),
    placeholder(2, 'lower2'),
    placeholder(3, 'lower3'),
  ], ['up', 'down', 'down', 'down']),
  
  tensor_1up_3down: tensor(placeholder(0, 'symbol'), [
    placeholder(1, 'upper'),
    placeholder(2, 'lower1'),
    placeholder(3, 'lower2'),
    placeholder(4, 'lower3'),
  ], ['up', 'down', 'down', 'down']),
  
  tensor_2up_2down: tensor(placeholder(0, 'symbol'), [
    placeholder(1, 'upper1'),
    placeholder(2, 'upper2'),
    placeholder(3, 'lower1'),
    placeholder(4, 'lower2'),
  ], ['up', 'up', 'down', 'down']),
  
  // ─────────────────────────────────────────────────────────────
  // Integral Transforms
  // ─────────────────────────────────────────────────────────────
  
  fourier_transform: operation('fourier_transform', [
    placeholder(0, 'function'),
    placeholder(1, 'variable'),
  ]),
  
  inverse_fourier: operation('inverse_fourier', [
    placeholder(0, 'function'),
    placeholder(1, 'variable'),
  ]),
  
  laplace_transform: operation('laplace_transform', [
    placeholder(0, 'function'),
    placeholder(1, 'variable'),
  ]),
  
  convolution: operation('convolution', [
    placeholder(0, 'f'),
    placeholder(1, 'g'),
    placeholder(2, 'variable'),
  ]),
  
  // ─────────────────────────────────────────────────────────────
  // Physics Constants (as Objects)
  // ─────────────────────────────────────────────────────────────
  
  hbar: objectNode('ℏ'),
  nabla: objectNode('∇'),
  infinity: objectNode('∞'),
  pi_const: objectNode('π'),
  euler_e: objectNode('e'),
  
  // ─────────────────────────────────────────────────────────────
  // Greek Letters (lowercase)
  // ─────────────────────────────────────────────────────────────
  alpha: objectNode('α'),
  beta: objectNode('β'),
  gamma: objectNode('γ'),
  delta: objectNode('δ'),
  epsilon: objectNode('ε'),
  zeta: objectNode('ζ'),
  eta: objectNode('η'),
  theta: objectNode('θ'),
  iota: objectNode('ι'),
  kappa: objectNode('κ'),
  lambda: objectNode('λ'),
  mu: objectNode('μ'),
  nu: objectNode('ν'),
  xi: objectNode('ξ'),
  omicron: objectNode('ο'),
  pi: objectNode('π'),
  rho: objectNode('ρ'),
  sigma: objectNode('σ'),
  tau: objectNode('τ'),
  upsilon: objectNode('υ'),
  phi: objectNode('φ'),
  chi: objectNode('χ'),
  psi: objectNode('ψ'),
  omega: objectNode('ω'),
  
  // ─────────────────────────────────────────────────────────────
  // Greek Letters (uppercase)
  // ─────────────────────────────────────────────────────────────
  Gamma: objectNode('Γ'),
  Delta: objectNode('Δ'),
  Theta: objectNode('Θ'),
  Lambda: objectNode('Λ'),
  Xi: objectNode('Ξ'),
  Pi: objectNode('Π'),
  Sigma: objectNode('Σ'),
  Upsilon: objectNode('Υ'),
  Phi: objectNode('Φ'),
  Psi: objectNode('Ψ'),
  Omega: objectNode('Ω'),
};

/**
 * Get a template by name, cloning it to avoid mutation
 */
export function getTemplate(name: string): EditorNode | undefined {
  const template = astTemplates[name];
  if (!template) return undefined;
  return JSON.parse(JSON.stringify(template));
}

/**
 * Get the number of templates
 */
export function getTemplateCount(): number {
  return Object.keys(astTemplates).length;
}
