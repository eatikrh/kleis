# Jupyter Iframe Widget POC

Proof of concept for embedding a visual widget (like the Equation Editor) in Jupyter notebooks via iframe.

## Quick Start

1. **Start the HTTP server** (serves the widget HTML):
   ```bash
   cd examples/jupyter-iframe-poc
   python3 -m http.server 8888
   ```

2. **Open the notebook** in Jupyter:
   ```bash
   cd kleis-notebook
   ./start-jupyter.sh
   ```
   Then open `examples/jupyter-iframe-poc/test_iframe.ipynb`

3. **Run the cells** to see the widget embedded in the notebook

## What This Tests

| Method | Description |
|--------|-------------|
| **Direct IFrame** | Simple embedding, always visible |
| **Toggle Button** | Click to show/hide the widget |
| **Message Passing** | Widget sends data back to Python kernel |

## Files

- `simple_widget.html` - A minimal symbol palette widget
- `test_iframe.ipynb` - Jupyter notebook testing the embedding

## How It Works

```
┌─────────────────────────────────┐
│  Jupyter Notebook               │
│  ┌────────────────────────────┐ │
│  │ [📐 Open Symbol Palette]   │ │  ← Click to toggle
│  │                            │ │
│  │ ┌────────────────────────┐ │ │
│  │ │  simple_widget.html    │ │ │  ← iframe
│  │ │  [∑] [∫] [√] [∀] [π]   │ │ │  ← symbol palette
│  │ │  Output: ∑∫√           │ │ │
│  │ │  [Send to Jupyter]     │ │ │  ← postMessage
│  │ └────────────────────────┘ │ │
│  │                            │ │
│  │ Received: ∑∫√              │ │  ← message received!
│  └────────────────────────────┘ │
└─────────────────────────────────┘
```

## Next Steps

If this POC works, we can:
1. Point the iframe to the real Equation Editor (`kleis server`)
2. Add a `?mode=jupyter` parameter for Jupyter-specific behavior
3. Handle richer message types (SVG, LaTeX, Kleis code)

