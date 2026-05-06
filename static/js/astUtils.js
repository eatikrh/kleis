import { state } from './state.js';

export function log(msg) { console.log(msg); }

export function showStatus(msg, type) {
    const el = document.getElementById('status');
    if (el) {
        el.innerHTML = msg;
        el.className = `status ${type}`;
    }
}

export function getNodeAtPath(ast, path) {
    let current = ast;
    if (path.length === 0) return current;
    if (current.Operation) {
        return getNodeAtPath(current.Operation.args[path[0]], path.slice(1));
    }
    return current;
}

export function setNodeAtPath(ast, path, newValue) {
    if (path.length === 0) return;
    let current = ast;
    for (let i = 0; i < path.length - 1; i++) {
        if (current.Operation) {
            current = current.Operation.args[path[i]];
        } else if (current.List && Array.isArray(current.List)) {
            current = current.List[path[i]];
        } else {
            console.error('setNodeAtPath: cannot traverse node at path', path, 'step', i);
            return;
        }
    }
    const lastIndex = path[path.length - 1];
    if (current.Operation && current.Operation.args) {
        current.Operation.args[lastIndex] = newValue;
    } else if (current.List && Array.isArray(current.List)) {
        current.List[lastIndex] = newValue;
    } else {
        console.error('setNodeAtPath: cannot set value on node at path', path);
    }
}

export function getNodeById(ast, nodeId) {
    const pathParts = nodeId.split('.').map(Number);
    let node = ast;
    for (let i = 1; i < pathParts.length; i++) {
        if (node.Operation && node.Operation.args) {
            node = node.Operation.args[pathParts[i]];
        } else if (node.List && Array.isArray(node.List)) {
            node = node.List[pathParts[i]];
        } else {
            console.warn('Could not navigate to node:', nodeId, 'at step', i, 'node:', node);
            return ast;
        }
    }
    return node;
}

export function nodeIdFromPath(pathArray) {
    const segments = [0];
    if (Array.isArray(pathArray)) {
        pathArray.forEach((idx) => segments.push(idx));
    }
    return segments.join('.');
}

export function parseSimpleInput(input) {
    if (!input) return { Placeholder: { id: state.nextPlaceholderId++, hint: 'val' } };
    if (/^-?\d+(\.\d+)?$/.test(input)) return { Const: input };
    return { Object: input };
}

export function renumberPlaceholders(node) {
    if (node.Placeholder) {
        node.Placeholder.id = state.nextPlaceholderId++;
    } else if (node.Operation) {
        node.Operation.args.forEach(renumberPlaceholders);
    }
}

export function findFirstPlaceholderPath(node, path) {
    if (!node) return null;
    if (node.Placeholder) return path;
    if (node.Operation && node.Operation.args) {
        for (let i = 0; i < node.Operation.args.length; i++) {
            const result = findFirstPlaceholderPath(node.Operation.args[i], [...path, i]);
            if (result) return result;
        }
    }
    if (node.List && Array.isArray(node.List)) {
        for (let i = 0; i < node.List.length; i++) {
            const result = findFirstPlaceholderPath(node.List[i], [...path, i]);
            if (result) return result;
        }
    }
    return null;
}

export function getAllMarkers() {
    return Array.from(document.querySelectorAll('.arg-overlay, .placeholder-overlay'));
}

export function formatASTAsTree(node, depth) {
    const indent = '  '.repeat(depth);
    if (node.Placeholder) {
        return `${indent}□ Placeholder(id=${node.Placeholder.id}, hint="${node.Placeholder.hint}")`;
    } else if (node.Const) {
        return `${indent}# Const("${node.Const}")`;
    } else if (node.Object) {
        return `${indent}○ Object("${node.Object}")`;
    } else if (node.Operation) {
        let result = `${indent}⊕ Operation("${node.Operation.name}")\n`;
        node.Operation.args.forEach((arg, i) => {
            result += `${indent}├─[${i}] ${formatASTAsTree(arg, depth + 1).trim()}\n`;
        });
        return result.trimEnd();
    }
    return `${indent}? Unknown`;
}

export function countPlaceholdersInAST(node) {
    if (node.Placeholder) return 1;
    if (node.Operation) {
        return node.Operation.args.reduce((sum, arg) => sum + countPlaceholdersInAST(arg), 0);
    }
    return 0;
}

export function countOperationsInAST(node) {
    if (node.Operation) {
        return 1 + node.Operation.args.reduce((sum, arg) => sum + countOperationsInAST(arg), 0);
    }
    return 0;
}

export function getASTDepth(node) {
    if (node.Operation) {
        return 1 + Math.max(...node.Operation.args.map(arg => getASTDepth(arg)), 0);
    }
    return 1;
}

export function countNodesInAST(node) {
    if (node.Operation) {
        return 1 + node.Operation.args.reduce((sum, arg) => sum + countNodesInAST(arg), 0);
    }
    return 1;
}

export function getNodeValueAtPath(ast, path) {
    if (!ast || path.length === 0) return '';
    let current = ast;
    for (let i = 0; i < path.length; i++) {
        if (current.Operation) {
            current = current.Operation.args[path[i]];
        } else if (current.List && Array.isArray(current.List)) {
            current = current.List[path[i]];
        } else {
            console.warn('getNodeValueAtPath: cannot traverse at path', path, 'step', i);
            return '';
        }
    }
    if (current.Const) return current.Const;
    if (current.Object) return current.Object;
    return '';
}

export function latexToUnicode(latex) {
    const map = {
        '\\alpha': 'α', '\\beta': 'β', '\\gamma': 'γ', '\\delta': 'δ',
        '\\epsilon': 'ε', '\\zeta': 'ζ', '\\eta': 'η', '\\theta': 'θ',
        '\\iota': 'ι', '\\kappa': 'κ', '\\lambda': 'λ', '\\mu': 'μ',
        '\\nu': 'ν', '\\xi': 'ξ', '\\omicron': 'ο', '\\pi': 'π',
        '\\rho': 'ρ', '\\sigma': 'σ', '\\tau': 'τ', '\\upsilon': 'υ',
        '\\phi': 'φ', '\\chi': 'χ', '\\psi': 'ψ', '\\omega': 'ω',
        '\\Gamma': 'Γ', '\\Delta': 'Δ', '\\Theta': 'Θ', '\\Lambda': 'Λ',
        '\\Xi': 'Ξ', '\\Pi': 'Π', '\\Sigma': 'Σ', '\\Upsilon': 'Υ',
        '\\Phi': 'Φ', '\\Psi': 'Ψ', '\\Omega': 'Ω',
        '\\times': '×', '\\div': '÷', '\\pm': '±', '\\mp': '∓',
        '\\cdot': '·', '\\ast': '∗', '\\neq': '≠', '\\infty': '∞',
        '\\equiv': '≡', '\\in': '∈', '\\notin': '∉', '\\subset': '⊂',
        '\\forall': '∀', '\\exists': '∃', '\\nabla': 'nabla', '\\partial': '∂',
        '\\square': '□'
    };
    return map[latex] || latex;
}
