/**
 * Palette Button Configurations
 * 
 * Defines which buttons appear in each palette tab.
 * To move a button between tabs, just move its config line.
 */

export interface ButtonConfig {
  /** Template name (key in astTemplates) */
  template: string;
  /** Display label (shown on button) */
  label: string;
  /** Tooltip/aria-label */
  tooltip: string;
  /** Optional Unicode symbol to display */
  symbol?: string;
}

export interface TabConfig {
  id: string;
  title: string;
  buttons: ButtonConfig[];
}

/**
 * Palette tab definitions
 */
export const paletteTabs: TabConfig[] = [
  {
    id: 'basic',
    title: 'Basic',
    buttons: [
      { template: 'fraction', label: 'a/b', tooltip: 'Fraction', symbol: '⁄' },
      { template: 'power', label: 'xⁿ', tooltip: 'Power/Exponent', symbol: '^' },
      { template: 'subscript', label: 'x₀', tooltip: 'Subscript', symbol: '_' },
      { template: 'sqrt', label: '√', tooltip: 'Square Root', symbol: '√' },
      { template: 'nthroot', label: 'ⁿ√', tooltip: 'Nth Root', symbol: '∜' },
    ],
  },
  {
    id: 'trig',
    title: 'Trig',
    buttons: [
      { template: 'sin', label: 'sin', tooltip: 'Sine' },
      { template: 'cos', label: 'cos', tooltip: 'Cosine' },
      { template: 'tan', label: 'tan', tooltip: 'Tangent' },
    ],
  },
  {
    id: 'calculus',
    title: 'Calculus',
    buttons: [
      { template: 'integral', label: '∫', tooltip: 'Definite Integral', symbol: '∫' },
      { template: 'derivative', label: 'd/dx', tooltip: 'Derivative' },
      { template: 'partial', label: '∂', tooltip: 'Partial Derivative', symbol: '∂' },
      { template: 'sum', label: 'Σ', tooltip: 'Summation', symbol: 'Σ' },
      { template: 'product', label: 'Π', tooltip: 'Product', symbol: 'Π' },
      { template: 'limit', label: 'lim', tooltip: 'Limit' },
    ],
  },
];

/**
 * Get all unique template names used in the palette
 */
export function getAllTemplateNames(): string[] {
  return paletteTabs.flatMap(tab => tab.buttons.map(btn => btn.template));
}

