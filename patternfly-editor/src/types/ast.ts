/**
 * Kleis Editor AST Types
 * 
 * These types match the AST format used by static/index.html
 * and expected by the Kleis Rust backend.
 */

export interface PlaceholderData {
  id: number;
  hint: string;
  value?: EditorNode;
}

export interface OperationData {
  name: string;
  args: EditorNode[];
  kind?: string;
  metadata?: Record<string, unknown>;
}

/**
 * EditorNode - The AST node type used in the Equation Editor
 * 
 * This matches the structure from static/index.html and corresponds
 * to the Rust EditorNode type in src/editor_ast.rs
 */
export type EditorNode = 
  | { Object: string }
  | { Const: { value: string } }
  | { Placeholder: PlaceholderData }
  | { Operation: OperationData }
  | { List: EditorNode[] };

/**
 * Create a placeholder node
 */
export function placeholder(id: number, hint: string): EditorNode {
  return { Placeholder: { id, hint } };
}

/**
 * Create an operation node
 */
export function operation(name: string, args: EditorNode[], kind?: string, metadata?: Record<string, unknown>): EditorNode {
  return { Operation: { name, args, kind, metadata } };
}

/**
 * Deep clone an AST node and renumber placeholders
 */
let globalPlaceholderId = 0;

export function cloneAndRenumber(node: EditorNode): EditorNode {
  const cloned = JSON.parse(JSON.stringify(node)) as EditorNode;
  renumberPlaceholders(cloned);
  return cloned;
}

function renumberPlaceholders(node: EditorNode): void {
  if ('Placeholder' in node) {
    node.Placeholder.id = globalPlaceholderId++;
  } else if ('Operation' in node) {
    node.Operation.args.forEach(renumberPlaceholders);
  } else if ('List' in node) {
    node.List.forEach(renumberPlaceholders);
  }
}

/**
 * Reset placeholder ID counter (for testing)
 */
export function resetPlaceholderCounter(): void {
  globalPlaceholderId = 0;
}

