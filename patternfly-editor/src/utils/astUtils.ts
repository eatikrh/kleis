/**
 * AST manipulation utilities
 * 
 * Functions for navigating and modifying the AST tree structure
 */

import type { EditorNode } from '../types/ast';

/**
 * Get node at a given path in the AST
 * Path is an array of indices: [0] = first arg, [0, 1] = second arg of first arg, etc.
 */
export function getNodeAtPath(ast: EditorNode, path: number[]): EditorNode | null {
  if (path.length === 0) {
    return ast;
  }

  const [first, ...rest] = path;

  if ('Operation' in ast) {
    if (first >= 0 && first < ast.Operation.args.length) {
      return getNodeAtPath(ast.Operation.args[first], rest);
    }
  } else if ('List' in ast) {
    if (first >= 0 && first < ast.List.length) {
      return getNodeAtPath(ast.List[first], rest);
    }
  }

  return null;
}

/**
 * Find the first placeholder in an AST node
 * Returns { id: number, path: number[] } or null if no placeholder found
 */
export function findFirstPlaceholder(ast: EditorNode, currentPath: number[] = []): { id: number; path: number[] } | null {
  if ('Placeholder' in ast) {
    return { id: ast.Placeholder.id, path: currentPath };
  }
  
  if ('Operation' in ast) {
    for (let i = 0; i < ast.Operation.args.length; i++) {
      const result = findFirstPlaceholder(ast.Operation.args[i], [...currentPath, i]);
      if (result) return result;
    }
  }
  
  if ('List' in ast) {
    for (let i = 0; i < ast.List.length; i++) {
      const result = findFirstPlaceholder(ast.List[i], [...currentPath, i]);
      if (result) return result;
    }
  }
  
  return null;
}

/**
 * Set node at a given path in the AST (immutably)
 * Returns a new AST with the node replaced
 */
export function setNodeAtPath(ast: EditorNode, path: number[], newNode: EditorNode): EditorNode {
  if (path.length === 0) {
    return newNode;
  }

  const [first, ...rest] = path;

  if ('Operation' in ast) {
    const newArgs = [...ast.Operation.args];
    if (first >= 0 && first < newArgs.length) {
      newArgs[first] = setNodeAtPath(newArgs[first], rest, newNode);
      return {
        Operation: {
          ...ast.Operation,
          args: newArgs,
        },
      };
    }
  } else if ('List' in ast) {
    const newList = [...ast.List];
    if (first >= 0 && first < newList.length) {
      newList[first] = setNodeAtPath(newList[first], rest, newNode);
      return { List: newList };
    }
  }

  // Path doesn't exist, return original
  return ast;
}

/**
 * Get path to a placeholder by its ID
 * Returns null if placeholder not found
 */
export function getPathFromPlaceholderId(placeholderId: string, ast: EditorNode | null): number[] | null {
  if (!ast) return null;
  
  // Extract numeric ID from "ph{number}" format
  const numericId = placeholderId.startsWith('ph') 
    ? parseInt(placeholderId.substring(2), 10)
    : parseInt(placeholderId, 10);
  
  if (isNaN(numericId)) return null;
  
  const findPath = (node: EditorNode, path: number[]): number[] | null => {
    if ('Placeholder' in node && node.Placeholder.id === numericId) {
      return path;
    }
    
    if ('Operation' in node) {
      for (let i = 0; i < node.Operation.args.length; i++) {
        const result = findPath(node.Operation.args[i], [...path, i]);
        if (result) return result;
      }
    }
    
    if ('List' in node) {
      for (let i = 0; i < node.List.length; i++) {
        const result = findPath(node.List[i], [...path, i]);
        if (result) return result;
      }
    }
    
    return null;
  };
  
  return findPath(ast, []);
}

/**
 * Parse simple input (number, variable, or placeholder)
 * Used for inline editing
 */
export function parseSimpleInput(input: string): EditorNode {
  const trimmed = input.trim();
  
  if (trimmed === '') {
    // Empty input creates a placeholder
    return { Placeholder: { id: Date.now(), hint: '' } };
  }

  // Try to parse as number
  const num = Number(trimmed);
  if (!isNaN(num) && isFinite(num)) {
    return { Const: trimmed };  // Backend expects { Const: "2" } format
  }

  // Otherwise treat as variable/object
  return { Object: trimmed };
}

/**
 * Get all placeholder paths in the AST
 * Returns array of { path, id, hint }
 */
export function getAllPlaceholderPaths(
  ast: EditorNode,
  basePath: number[] = []
): Array<{ path: number[]; id: number; hint: string }> {
  const results: Array<{ path: number[]; id: number; hint: string }> = [];

  if ('Placeholder' in ast) {
    results.push({
      path: basePath,
      id: ast.Placeholder.id,
      hint: ast.Placeholder.hint,
    });
  } else if ('Operation' in ast) {
    ast.Operation.args.forEach((arg, index) => {
      results.push(...getAllPlaceholderPaths(arg, [...basePath, index]));
    });
  } else if ('List' in ast) {
    ast.List.forEach((item, index) => {
      results.push(...getAllPlaceholderPaths(item, [...basePath, index]));
    });
  }

  return results;
}

