/**
 * Palette Button Configurations
 * 
 * Defines which buttons appear in each palette tab.
 * To move a button between tabs, just move its config line.
 * 
 * Matches the structure in static/index.html.
 */

export interface ButtonConfig {
  /** Template name (key in astTemplates) */
  template: string;
  /** Display label (shown on button) */
  label: string;
  /** Tooltip/aria-label */
  tooltip: string;
  /** Optional Unicode symbol to display instead of label */
  symbol?: string;
}

export interface TabConfig {
  id: string;
  title: string;
  buttons: ButtonConfig[];
}

/**
 * Palette tab definitions - organized to match static/index.html
 */
export const paletteTabs: TabConfig[] = [
  {
    id: 'basics',
    title: 'Basics',
    buttons: [
      { template: 'plus', label: '+', tooltip: 'Plus' },
      { template: 'minus', label: '−', tooltip: 'Minus' },
      { template: 'times', label: '×', tooltip: 'Times' },
      { template: 'fraction', label: '⁄', tooltip: 'Fraction', symbol: 'a/b' },
      { template: 'power', label: 'xⁿ', tooltip: 'Power/Exponent' },
      { template: 'subscript', label: 'x₀', tooltip: 'Subscript' },
      { template: 'sqrt', label: '√', tooltip: 'Square Root' },
      { template: 'nthroot', label: 'ⁿ√', tooltip: 'Nth Root' },
      { template: 'factorial', label: 'n!', tooltip: 'Factorial' },
      { template: 'abs', label: '|x|', tooltip: 'Absolute Value' },
      { template: 'binomial', label: 'C(n,k)', tooltip: 'Binomial Coefficient' },
      { template: 'floor', label: '⌊x⌋', tooltip: 'Floor' },
      { template: 'ceiling', label: '⌈x⌉', tooltip: 'Ceiling' },
    ],
  },
  {
    id: 'fences',
    title: 'Fences',
    buttons: [
      { template: 'parens', label: '(x)', tooltip: 'Parentheses' },
      { template: 'brackets', label: '[x]', tooltip: 'Square Brackets' },
      { template: 'braces', label: '{x}', tooltip: 'Curly Braces' },
      { template: 'angle_brackets', label: '⟨x⟩', tooltip: 'Angle Brackets' },
      { template: 'norm', label: '‖x‖', tooltip: 'Norm' },
    ],
  },
  {
    id: 'accents',
    title: 'Accents',
    buttons: [
      { template: 'dot_accent', label: 'ẋ', tooltip: 'Dot Accent' },
      { template: 'ddot_accent', label: 'ẍ', tooltip: 'Double Dot' },
      { template: 'hat', label: 'x̂', tooltip: 'Hat' },
      { template: 'bar', label: 'x̄', tooltip: 'Bar' },
      { template: 'tilde', label: 'x̃', tooltip: 'Tilde' },
      { template: 'vector_arrow', label: 'x⃗', tooltip: 'Vector Arrow' },
      { template: 'vector_bold', label: '𝐱', tooltip: 'Bold Vector' },
    ],
  },
  {
    id: 'calculus',
    title: 'Calculus',
    buttons: [
      { template: 'integral', label: '∫', tooltip: 'Definite Integral' },
      { template: 'derivative', label: 'd/dx', tooltip: 'Derivative' },
      { template: 'partial', label: '∂', tooltip: 'Partial Derivative' },
      { template: 'sum', label: 'Σ', tooltip: 'Summation' },
      { template: 'product', label: 'Π', tooltip: 'Product' },
      { template: 'limit', label: 'lim', tooltip: 'Limit' },
      { template: 'gradient', label: '∇', tooltip: 'Gradient' },
      { template: 'nabla', label: '∇', tooltip: 'Nabla Symbol' },
      { template: 'infinity', label: '∞', tooltip: 'Infinity' },
    ],
  },
  {
    id: 'linear',
    title: 'Linear Algebra',
    buttons: [
      { template: 'matrix2x2', label: '2×2', tooltip: '2×2 Matrix' },
      { template: 'matrix3x3', label: '3×3', tooltip: '3×3 Matrix' },
      { template: 'pmatrix2x2', label: '(2×2)', tooltip: '2×2 Paren Matrix' },
      { template: 'vmatrix2x2', label: '|2×2|', tooltip: '2×2 Determinant' },
      { template: 'matrix_multiply', label: 'A·B', tooltip: 'Matrix Multiply' },
      { template: 'dot', label: 'a·b', tooltip: 'Dot Product' },
      { template: 'cross', label: 'a×b', tooltip: 'Cross Product' },
    ],
  },
  {
    id: 'functions',
    title: 'Functions',
    buttons: [
      { template: 'sin', label: 'sin', tooltip: 'Sine' },
      { template: 'cos', label: 'cos', tooltip: 'Cosine' },
      { template: 'tan', label: 'tan', tooltip: 'Tangent' },
      { template: 'arcsin', label: 'arcsin', tooltip: 'Inverse Sine' },
      { template: 'arccos', label: 'arccos', tooltip: 'Inverse Cosine' },
      { template: 'arctan', label: 'arctan', tooltip: 'Inverse Tangent' },
      { template: 'ln', label: 'ln', tooltip: 'Natural Log' },
      { template: 'log', label: 'log', tooltip: 'Logarithm' },
      { template: 'exp', label: 'exp', tooltip: 'Exponential' },
      { template: 'euler_e', label: 'e', tooltip: 'Euler\'s Number' },
      { template: 'pi_const', label: 'π', tooltip: 'Pi' },
    ],
  },
  {
    id: 'logic',
    title: 'Logic & Sets',
    buttons: [
      { template: 'equals', label: '=', tooltip: 'Equals' },
      { template: 'neq', label: '≠', tooltip: 'Not Equals' },
      { template: 'less_than', label: '<', tooltip: 'Less Than' },
      { template: 'greater_than', label: '>', tooltip: 'Greater Than' },
      { template: 'leq', label: '≤', tooltip: 'Less or Equal' },
      { template: 'geq', label: '≥', tooltip: 'Greater or Equal' },
      { template: 'approx', label: '≈', tooltip: 'Approximately' },
      { template: 'logical_and', label: '∧', tooltip: 'Logical And' },
      { template: 'logical_or', label: '∨', tooltip: 'Logical Or' },
      { template: 'logical_not', label: '¬', tooltip: 'Logical Not' },
    ],
  },
  {
    id: 'physics',
    title: 'Physics',
    buttons: [
      { template: 'hbar', label: 'ℏ', tooltip: 'Reduced Planck' },
      { template: 'ket', label: '|ψ⟩', tooltip: 'Ket' },
      { template: 'bra', label: '⟨ψ|', tooltip: 'Bra' },
      { template: 'inner', label: '⟨φ|ψ⟩', tooltip: 'Inner Product' },
      { template: 'outer', label: '|ψ⟩⟨φ|', tooltip: 'Outer Product' },
      { template: 'commutator', label: '[A,B]', tooltip: 'Commutator' },
      { template: 'expectation', label: '⟨A⟩', tooltip: 'Expectation' },
    ],
  },
  {
    id: 'tensors',
    title: 'Tensors',
    buttons: [
      { template: 'metric', label: 'gμν', tooltip: 'Metric Tensor' },
      { template: 'christoffel', label: 'Γ', tooltip: 'Christoffel Symbol' },
      { template: 'riemann', label: 'R', tooltip: 'Riemann Tensor' },
      { template: 'tensor_mixed', label: 'Tᵘᵥ', tooltip: 'Mixed Tensor' },
      { template: 'subsup', label: 'T^a_b', tooltip: 'Sub-Superscript' },
      { template: 'tensor_1up_3down', label: 'T¹₃', tooltip: '1 Up 3 Down' },
      { template: 'tensor_2up_2down', label: 'T²₂', tooltip: '2 Up 2 Down' },
    ],
  },
  {
    id: 'transforms',
    title: 'Transforms',
    buttons: [
      { template: 'fourier_transform', label: 'ℱ', tooltip: 'Fourier Transform' },
      { template: 'inverse_fourier', label: 'ℱ⁻¹', tooltip: 'Inverse Fourier' },
      { template: 'laplace_transform', label: 'ℒ', tooltip: 'Laplace Transform' },
      { template: 'convolution', label: 'f∗g', tooltip: 'Convolution' },
    ],
  },
];

/**
 * Get all unique template names used in the palette
 */
export function getAllTemplateNames(): string[] {
  return paletteTabs.flatMap(tab => tab.buttons.map(btn => btn.template));
}

/**
 * Get total button count
 */
export function getTotalButtonCount(): number {
  return paletteTabs.reduce((sum, tab) => sum + tab.buttons.length, 0);
}
