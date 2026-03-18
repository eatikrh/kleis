/**
 * Palette Button Configurations
 * 
 * Defines which buttons appear in each palette tab.
 * To move a button between tabs, just move its config line.
 * 
 * Matches the structure in static/index.html.
 */

import React from 'react';

export interface ButtonConfig {
  /** Template name (key in astTemplates) - OR special action like 'matrix-builder' or 'piecewise-builder' */
  template: string;
  /** Display label (shown on button) - fallback if LaTeX not available */
  label: string;
  /** Tooltip/aria-label */
  tooltip: string;
  /** Optional Unicode symbol to display instead of label - fallback */
  symbol?: string;
  /** LaTeX string to render with MathJax (e.g., `\\(a = b\\)`) */
  latex?: string;
  /** Custom SVG element (for buttons like Matrix Builder) */
  customSvg?: React.ReactNode;
  /** Custom action handler (for buttons that open modals instead of inserting templates) */
  customAction?: () => void;
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
      { template: 'plus', label: '+', tooltip: 'Plus', latex: '\\(+\\)' },
      { template: 'minus', label: '−', tooltip: 'Minus', latex: '\\(-\\)' },
      { template: 'times', label: '×', tooltip: 'Times', latex: '\\(\\times\\)' },
      { template: 'fraction', label: '⁄', tooltip: 'Fraction', latex: '\\(\\frac{a}{b}\\)' },
      { template: 'power', label: 'xⁿ', tooltip: 'Power/Exponent', latex: '\\(x^n\\)' },
      { template: 'subscript', label: 'x₀', tooltip: 'Subscript', latex: '\\(x_i\\)' },
      { template: 'sqrt', label: '√', tooltip: 'Square Root', latex: '\\(\\sqrt{x}\\)' },
      { template: 'nthroot', label: 'ⁿ√', tooltip: 'Nth Root', latex: '\\(\\sqrt[n]{x}\\)' },
      { template: 'factorial', label: 'n!', tooltip: 'Factorial', latex: '\\(n!\\)' },
      { template: 'abs', label: '|x|', tooltip: 'Absolute Value', latex: '\\(|x|\\)' },
      { template: 'binomial', label: 'C(n,k)', tooltip: 'Binomial Coefficient', latex: '\\(\\binom{n}{k}\\)' },
      { template: 'floor', label: '⌊x⌋', tooltip: 'Floor', latex: '\\(\\lfloor x \\rfloor\\)' },
      { template: 'ceiling', label: '⌈x⌉', tooltip: 'Ceiling', latex: '\\(\\lceil x \\rceil\\)' },
      { template: 'equals', label: '=', tooltip: 'Equality', latex: '\\(a = b\\)' },
      { template: 'plus', label: '+', tooltip: 'Addition', latex: '\\(a + b\\)' },
      { template: 'minus', label: '−', tooltip: 'Subtraction', latex: '\\(a - b\\)' },
      { template: 'times', label: '×', tooltip: 'Multiplication', latex: '\\(a \\cdot b\\)' },
      { template: 'negate', label: '-x', tooltip: 'Negate', latex: '\\(-x\\)' },
      { template: 'sin', label: 'sin', tooltip: 'Sine', latex: '\\(\\sin(x)\\)' },
      { template: 'cos', label: 'cos', tooltip: 'Cosine', latex: '\\(\\cos(x)\\)' },
      { template: 'tan', label: 'tan', tooltip: 'Tangent', latex: '\\(\\tan(x)\\)' },
      { template: 'arcsin', label: 'arcsin', tooltip: 'Arcsine', latex: '\\(\\arcsin(x)\\)' },
      { template: 'arccos', label: 'arccos', tooltip: 'Arccosine', latex: '\\(\\arccos(x)\\)' },
      { template: 'arctan', label: 'arctan', tooltip: 'Arctangent', latex: '\\(\\arctan(x)\\)' },
      { template: 'ln', label: 'ln', tooltip: 'Natural Log', latex: '\\(\\ln(x)\\)' },
      { template: 'log', label: 'log', tooltip: 'Logarithm', latex: '\\(\\log(x)\\)' },
      { template: 'exp', label: 'exp', tooltip: 'Exponential', latex: '\\(\\exp(x)\\)' },
      { template: 'exp_e', label: 'e^x', tooltip: 'e to the power', latex: '\\(e^x\\)' },
      { template: 'piecewise-builder', label: '📐', tooltip: 'Piecewise Function Builder', latex: '\\(\\begin{cases}f_1 & c_1\\\\f_2 & c_2\\end{cases}\\)', customAction: () => {
        // This will be set by PaletteTabs component
        if ((window as any).openPiecewiseBuilder) {
          (window as any).openPiecewiseBuilder();
        }
      } },
    ],
  },
  {
    id: 'fences',
    title: 'Fences',
    buttons: [
      { template: 'parens', label: '(x)', tooltip: 'Parentheses', latex: '\\((x)\\)' },
      { template: 'brackets', label: '[x]', tooltip: 'Square Brackets', latex: '\\([x]\\)' },
      { template: 'braces', label: '{x}', tooltip: 'Curly Braces', latex: '\\(\\{x\\}\\)' },
      { template: 'angle_brackets', label: '⟨x⟩', tooltip: 'Angle Brackets', latex: '\\(\\langle x \\rangle\\)' },
      { template: 'abs', label: '|x|', tooltip: 'Absolute Value', latex: '\\(|x|\\)' },
      { template: 'norm', label: '‖x‖', tooltip: 'Norm', latex: '\\(\\|v\\|\\)' },
      { template: 'floor', label: '⌊x⌋', tooltip: 'Floor', latex: '\\(\\lfloor x \\rfloor\\)' },
      { template: 'ceiling', label: '⌈x⌉', tooltip: 'Ceiling', latex: '\\(\\lceil x \\rceil\\)' },
    ],
  },
  {
    id: 'accents',
    title: 'Accents',
    buttons: [
      { template: 'dot_accent', label: 'ẋ', tooltip: 'Dot Accent', latex: '\\(\\dot{x}\\)' },
      { template: 'ddot_accent', label: 'ẍ', tooltip: 'Double Dot', latex: '\\(\\ddot{x}\\)' },
      { template: 'hat', label: 'x̂', tooltip: 'Hat', latex: '\\(\\hat{x}\\)' },
      { template: 'bar', label: 'x̄', tooltip: 'Bar', latex: '\\(\\bar{x}\\)' },
      { template: 'tilde', label: 'x̃', tooltip: 'Tilde', latex: '\\(\\tilde{x}\\)' },
      { template: 'overline', label: 'ABC', tooltip: 'Overline', latex: '\\(\\overline{ABC}\\)' },
      { template: 'underline', label: 'ABC', tooltip: 'Underline', latex: '\\(\\underline{ABC}\\)' },
      { template: 'vector_arrow', label: 'x⃗', tooltip: 'Vector Arrow', latex: '\\(\\vec{v}\\)' },
      { template: 'vector_bold', label: '𝐱', tooltip: 'Bold', latex: '\\(\\mathbf{x}\\)' },
    ],
  },
  {
    id: 'calculus',
    title: 'Calculus',
    buttons: [
      { template: 'integral', label: '∫', tooltip: 'Definite Integral', latex: '\\(\\int_a^b f\\,dx\\)' },
      { template: 'sum', label: 'Σ', tooltip: 'Summation', latex: '\\(\\sum_{i=1}^n a_i\\)' },
      { template: 'product', label: 'Π', tooltip: 'Product', latex: '\\(\\prod_{i=1}^n a_i\\)' },
      { template: 'limit', label: 'lim', tooltip: 'Limit', latex: '\\(\\lim_{x\\to 0} f(x)\\)' },
      { template: 'derivative', label: 'd/dx', tooltip: 'Derivative', latex: '\\(\\frac{df}{dx}\\)' },
      { template: 'partial', label: '∂', tooltip: 'Partial Derivative', latex: '\\(\\frac{\\partial f}{\\partial x}\\)' },
      { template: 'gradient', label: '∇', tooltip: 'Gradient', latex: '\\(\\nabla f\\)' },
      { template: 'fourier_transform', label: 'ℱ', tooltip: 'Fourier Transform', latex: '\\(\\mathcal{F}[f](\\omega)\\)' },
      { template: 'inverse_fourier', label: 'ℱ⁻¹', tooltip: 'Inverse Fourier', latex: '\\(\\mathcal{F}^{-1}[F](t)\\)' },
      { template: 'laplace_transform', label: 'ℒ', tooltip: 'Laplace Transform', latex: '\\(\\mathcal{L}[f](s)\\)' },
      { template: 'inverse_laplace', label: 'ℒ⁻¹', tooltip: 'Inverse Laplace', latex: '\\(\\mathcal{L}^{-1}[F](t)\\)' },
      { template: 'convolution', label: 'f∗g', tooltip: 'Convolution', latex: '\\((f \\ast g)(x)\\)' },
      { template: 'kernel_integral', label: '∫K', tooltip: 'Kernel Integral', latex: '\\(\\int_D K f\\,d\\mu\\)' },
      { template: 'greens_function', label: 'G', tooltip: 'Green\'s Function', latex: '\\(G(x,m)\\)' },
    ],
  },
  {
    id: 'linear',
    title: 'Linear Algebra',
    buttons: [
      { template: 'matrix2x2', label: '2×2', tooltip: 'Matrix 2×2 [brackets]', latex: '\\(\\begin{bmatrix}a&b\\\\c&d\\end{bmatrix}\\)' },
      { template: 'matrix3x3', label: '3×3', tooltip: 'Matrix 3×3 [brackets]', latex: '\\(\\begin{bmatrix}a&b&c\\\\d&e&f\\\\g&h&i\\end{bmatrix}\\)' },
      { template: 'pmatrix2x2', label: '(2×2)', tooltip: 'Matrix 2×2 (parens)', latex: '\\(\\begin{pmatrix}a&b\\\\c&d\\end{pmatrix}\\)' },
      { template: 'pmatrix3x3', label: '(3×3)', tooltip: 'Matrix 3×3 (parens)', latex: '\\(\\begin{pmatrix}a&b&c\\\\d&e&f\\\\g&h&i\\end{pmatrix}\\)' },
      { template: 'vmatrix2x2', label: '|2×2|', tooltip: 'Determinant 2×2', latex: '\\(\\begin{vmatrix}a&b\\\\c&d\\end{vmatrix}\\)' },
      { template: 'vmatrix3x3', label: '|3×3|', tooltip: 'Determinant 3×3', latex: '\\(\\begin{vmatrix}a&b&c\\\\d&e&f\\\\g&h&i\\end{vmatrix}\\)' },
      { template: 'vector_bold', label: '𝐯', tooltip: 'Bold Vector', latex: '\\(\\mathbf{v}\\)' },
      { template: 'vector_arrow', label: 'v⃗', tooltip: 'Vector Arrow', latex: '\\(\\vec{v}\\)' },
      { template: 'matrix_multiply', label: 'A·B', tooltip: 'Matrix Multiplication', latex: '\\(A \\bullet B\\)' },
      { template: 'matrix-builder', label: '', tooltip: 'Custom Matrix Builder', customSvg: React.createElement('svg', {
        xmlns: 'http://www.w3.org/2000/svg',
        width: 36,
        height: 28,
        viewBox: '0 0 64 48'
      },
        React.createElement('g', { stroke: '#333', strokeWidth: 2.5, fill: 'none' },
          React.createElement('path', { d: 'M 12 6 L 8 6 L 8 42 L 12 42' }),
          React.createElement('path', { d: 'M 52 6 L 56 6 L 56 42 L 52 42' })
        ),
        React.createElement('g', { stroke: '#666', strokeWidth: 1.8, fill: 'none', opacity: 0.7 },
          React.createElement('line', { x1: 22, y1: 12, x2: 22, y2: 36 }),
          React.createElement('line', { x1: 30, y1: 12, x2: 30, y2: 36 }),
          React.createElement('line', { x1: 38, y1: 12, x2: 38, y2: 36 }),
          React.createElement('line', { x1: 46, y1: 12, x2: 46, y2: 36 }),
          React.createElement('line', { x1: 14, y1: 18, x2: 50, y2: 18 }),
          React.createElement('line', { x1: 14, y1: 24, x2: 50, y2: 24 }),
          React.createElement('line', { x1: 14, y1: 30, x2: 50, y2: 30 }),
          React.createElement('line', { x1: 14, y1: 36, x2: 50, y2: 36 })
        ),
        React.createElement('g', { transform: 'translate(50, 36)' },
          React.createElement('circle', { cx: 0, cy: 0, r: 7, fill: '#4CAF50', stroke: 'white', strokeWidth: 1.5 }),
          React.createElement('g', { stroke: 'white', strokeWidth: 2.2, strokeLinecap: 'round' },
            React.createElement('line', { x1: -3, y1: 0, x2: 3, y2: 0 }),
            React.createElement('line', { x1: 0, y1: -3, x2: 0, y2: 3 })
          )
        )
      ), customAction: () => {
        // This will be set by PaletteTabs component
        if ((window as any).openMatrixBuilder) {
          (window as any).openMatrixBuilder();
        }
      } },
      { template: 'dot', label: 'a·b', tooltip: 'Dot Product', latex: '\\(a \\cdot b\\)' },
      { template: 'cross', label: 'a×b', tooltip: 'Cross Product', latex: '\\(a \\times b\\)' },
    ],
  },
  {
    id: 'functions',
    title: 'Functions',
    buttons: [
      { template: 'sin', label: 'sin', tooltip: 'Sine', latex: '\\(\\sin(x)\\)' },
      { template: 'cos', label: 'cos', tooltip: 'Cosine', latex: '\\(\\cos(x)\\)' },
      { template: 'tan', label: 'tan', tooltip: 'Tangent', latex: '\\(\\tan(x)\\)' },
      { template: 'arcsin', label: 'arcsin', tooltip: 'Arcsine', latex: '\\(\\arcsin(x)\\)' },
      { template: 'arccos', label: 'arccos', tooltip: 'Arccosine', latex: '\\(\\arccos(x)\\)' },
      { template: 'arctan', label: 'arctan', tooltip: 'Arctangent', latex: '\\(\\arctan(x)\\)' },
      { template: 'ln', label: 'ln', tooltip: 'Natural Log', latex: '\\(\\ln(x)\\)' },
      { template: 'log', label: 'log', tooltip: 'Logarithm', latex: '\\(\\log(x)\\)' },
      { template: 'exp', label: 'exp', tooltip: 'Exponential', latex: '\\(\\exp(x)\\)' },
      { template: 'exp_e', label: 'e^x', tooltip: 'e to the power', latex: '\\(e^x\\)' },
    ],
  },
  {
    id: 'logic',
    title: 'Logic & Sets',
    buttons: [
      { template: 'let_simple', label: 'let', tooltip: 'Let Binding', latex: '\\(\\text{let } x = v\\)' },
      { template: 'equals', label: '=', tooltip: 'Equals', latex: '\\(a = b\\)' },
      { template: 'neq', label: '≠', tooltip: 'Not Equal', latex: '\\(a \\neq b\\)' },
      { template: 'less_than', label: '<', tooltip: 'Less Than', latex: '\\(a < b\\)' },
      { template: 'greater_than', label: '>', tooltip: 'Greater Than', latex: '\\(a > b\\)' },
      { template: 'leq', label: '≤', tooltip: 'Less or Equal', latex: '\\(a \\leq b\\)' },
      { template: 'geq', label: '≥', tooltip: 'Greater or Equal', latex: '\\(a \\geq b\\)' },
      { template: 'approx', label: '≈', tooltip: 'Approximately Equal', latex: '\\(a \\approx b\\)' },
      { template: 'logical_and', label: '∧', tooltip: 'Logical AND', latex: '\\(a \\land b\\)' },
      { template: 'logical_or', label: '∨', tooltip: 'Logical OR', latex: '\\(a \\lor b\\)' },
      { template: 'logical_not', label: '¬', tooltip: 'Logical NOT', latex: '\\(\\lnot p\\)' },
      { template: 'equiv', label: '≡', tooltip: 'Equivalent', latex: '\\(\\equiv\\)' },
      { template: 'in', label: '∈', tooltip: 'Element Of', latex: '\\(\\in\\)' },
      { template: 'notin', label: '∉', tooltip: 'Not Element Of', latex: '\\(\\notin\\)' },
      { template: 'subset', label: '⊂', tooltip: 'Subset', latex: '\\(\\subset\\)' },
      { template: 'subseteq', label: '⊆', tooltip: 'Subset or Equal', latex: '\\(\\subseteq\\)' },
      { template: 'cup', label: '∪', tooltip: 'Union', latex: '\\(\\cup\\)' },
      { template: 'cap', label: '∩', tooltip: 'Intersection', latex: '\\(\\cap\\)' },
      { template: 'emptyset', label: '∅', tooltip: 'Empty Set', latex: '\\(\\emptyset\\)' },
      { template: 'to', label: '→', tooltip: 'Maps To', latex: '\\(\\to\\)' },
      { template: 'Rightarrow', label: '⇒', tooltip: 'Implies', latex: '\\(\\Rightarrow\\)' },
      { template: 'Leftrightarrow', label: '⇔', tooltip: 'If and Only If', latex: '\\(\\Leftrightarrow\\)' },
      { template: 'forall', label: '∀', tooltip: 'For All', latex: '\\(\\forall\\)' },
      { template: 'exists', label: '∃', tooltip: 'There Exists', latex: '\\(\\exists\\)' },
    ],
  },
  {
    id: 'physics',
    title: 'Physics',
    buttons: [
      { template: 'ket', label: '|ψ⟩', tooltip: 'Ket', latex: '\\(|\\psi\\rangle\\)' },
      { template: 'bra', label: '⟨ψ|', tooltip: 'Bra', latex: '\\(\\langle\\phi|\\)' },
      { template: 'inner', label: '⟨φ|ψ⟩', tooltip: 'Inner Product', latex: '\\(\\langle\\phi|\\psi\\rangle\\)' },
      { template: 'outer', label: '|ψ⟩⟨φ|', tooltip: 'Outer Product', latex: '\\(|\\psi\\rangle\\langle\\phi|\\)' },
      { template: 'expectation', label: '⟨A⟩', tooltip: 'Expectation Value', latex: '\\(\\langle A \\rangle\\)' },
      { template: 'commutator', label: '[A,B]', tooltip: 'Commutator', latex: '\\([A, B]\\)' },
      { template: 'metric', label: 'gμν', tooltip: 'Metric Tensor', latex: '\\(g_{\\mu\\nu}\\)' },
      { template: 'christoffel', label: 'Γ', tooltip: 'Christoffel Symbol', latex: '\\(\\Gamma^\\lambda_{\\mu\\nu}\\)' },
      { template: 'riemann', label: 'R', tooltip: 'Riemann Tensor', latex: '\\(R^\\rho_{\\sigma\\mu\\nu}\\)' },
    ],
  },
  {
    id: 'tensors',
    title: 'Tensors',
    buttons: [
      { template: 'metric', label: 'gμν', tooltip: 'Metric Tensor', latex: '\\(g_{\\mu\\nu}\\)' },
      { template: 'christoffel', label: 'Γ', tooltip: 'Christoffel Symbol', latex: '\\(\\Gamma^\\lambda_{\\mu\\nu}\\)' },
      { template: 'riemann', label: 'R', tooltip: 'Riemann Tensor', latex: '\\(R^{\\mu\\nu}_{\\rho\\sigma}\\)' },
      { template: 'tensor_mixed', label: 'Tᵘᵥ', tooltip: 'Mixed Tensor', latex: '\\(T^i_j\\)' },
      { template: 'subsup', label: 'T^a_b', tooltip: 'Sub-Superscript', latex: '\\(T_j^i\\)' },
      { template: 'tensor_covariant', label: 'Tμν', tooltip: 'Covariant Tensor', latex: '\\(g_{\\mu\\nu}\\)' },
      { template: 'tensor_1up_3down', label: 'T¹₃', tooltip: '1 Up 3 Down', latex: '\\(T^\\mu_{\\nu\\rho\\sigma}\\)' },
      { template: 'tensor_2up_2down', label: 'T²₂', tooltip: '2 Up 2 Down', latex: '\\(R^{\\mu\\nu}_{\\rho\\sigma}\\)' },
    ],
  },
  {
    id: 'transforms',
    title: 'Transforms',
    buttons: [
      { template: 'fourier_transform', label: 'ℱ', tooltip: 'Fourier Transform', latex: '\\(\\mathcal{F}[f](\\omega)\\)' },
      { template: 'inverse_fourier', label: 'ℱ⁻¹', tooltip: 'Inverse Fourier', latex: '\\(\\mathcal{F}^{-1}[F](t)\\)' },
      { template: 'laplace_transform', label: 'ℒ', tooltip: 'Laplace Transform', latex: '\\(\\mathcal{L}[f](s)\\)' },
      { template: 'inverse_laplace', label: 'ℒ⁻¹', tooltip: 'Inverse Laplace', latex: '\\(\\mathcal{L}^{-1}[F](t)\\)' },
      { template: 'convolution', label: 'f∗g', tooltip: 'Convolution', latex: '\\((f \\ast g)(x)\\)' },
    ],
  },
  {
    id: 'greek',
    title: 'Greek',
    buttons: [
      // Lowercase
      { template: 'alpha', label: 'α', tooltip: 'Alpha', latex: '\\(\\alpha\\)' },
      { template: 'beta', label: 'β', tooltip: 'Beta', latex: '\\(\\beta\\)' },
      { template: 'gamma', label: 'γ', tooltip: 'Gamma', latex: '\\(\\gamma\\)' },
      { template: 'delta', label: 'δ', tooltip: 'Delta', latex: '\\(\\delta\\)' },
      { template: 'epsilon', label: 'ε', tooltip: 'Epsilon', latex: '\\(\\epsilon\\)' },
      { template: 'zeta', label: 'ζ', tooltip: 'Zeta', latex: '\\(\\zeta\\)' },
      { template: 'eta', label: 'η', tooltip: 'Eta', latex: '\\(\\eta\\)' },
      { template: 'theta', label: 'θ', tooltip: 'Theta', latex: '\\(\\theta\\)' },
      { template: 'iota', label: 'ι', tooltip: 'Iota', latex: '\\(\\iota\\)' },
      { template: 'kappa', label: 'κ', tooltip: 'Kappa', latex: '\\(\\kappa\\)' },
      { template: 'lambda', label: 'λ', tooltip: 'Lambda', latex: '\\(\\lambda\\)' },
      { template: 'mu', label: 'μ', tooltip: 'Mu', latex: '\\(\\mu\\)' },
      { template: 'nu', label: 'ν', tooltip: 'Nu', latex: '\\(\\nu\\)' },
      { template: 'xi', label: 'ξ', tooltip: 'Xi', latex: '\\(\\xi\\)' },
      { template: 'omicron', label: 'ο', tooltip: 'Omicron', latex: '\\(o\\)' },
      { template: 'pi', label: 'π', tooltip: 'Pi', latex: '\\(\\pi\\)' },
      { template: 'rho', label: 'ρ', tooltip: 'Rho', latex: '\\(\\rho\\)' },
      { template: 'sigma', label: 'σ', tooltip: 'Sigma', latex: '\\(\\sigma\\)' },
      { template: 'tau', label: 'τ', tooltip: 'Tau', latex: '\\(\\tau\\)' },
      { template: 'upsilon', label: 'υ', tooltip: 'Upsilon', latex: '\\(\\upsilon\\)' },
      { template: 'phi', label: 'φ', tooltip: 'Phi', latex: '\\(\\phi\\)' },
      { template: 'chi', label: 'χ', tooltip: 'Chi', latex: '\\(\\chi\\)' },
      { template: 'psi', label: 'ψ', tooltip: 'Psi', latex: '\\(\\psi\\)' },
      { template: 'omega', label: 'ω', tooltip: 'Omega', latex: '\\(\\omega\\)' },
      // Uppercase
      { template: 'Gamma', label: 'Γ', tooltip: 'Gamma (upper)', latex: '\\(\\Gamma\\)' },
      { template: 'Delta', label: 'Δ', tooltip: 'Delta (upper)', latex: '\\(\\Delta\\)' },
      { template: 'Theta', label: 'Θ', tooltip: 'Theta (upper)', latex: '\\(\\Theta\\)' },
      { template: 'Lambda', label: 'Λ', tooltip: 'Lambda (upper)', latex: '\\(\\Lambda\\)' },
      { template: 'Xi', label: 'Ξ', tooltip: 'Xi (upper)', latex: '\\(\\Xi\\)' },
      { template: 'Pi', label: 'Π', tooltip: 'Pi (upper)', latex: '\\(\\Pi\\)' },
      { template: 'Sigma', label: 'Σ', tooltip: 'Sigma (upper)', latex: '\\(\\Sigma\\)' },
      { template: 'Upsilon', label: 'Υ', tooltip: 'Upsilon (upper)', latex: '\\(\\Upsilon\\)' },
      { template: 'Phi', label: 'Φ', tooltip: 'Phi (upper)', latex: '\\(\\Phi\\)' },
      { template: 'Psi', label: 'Ψ', tooltip: 'Psi (upper)', latex: '\\(\\Psi\\)' },
      { template: 'Omega', label: 'Ω', tooltip: 'Omega (upper)', latex: '\\(\\Omega\\)' },
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
