"""
Equation Editor integration for Jupyter notebooks.

Embeds the existing Kleis Equation Editor (static/index.html) in Jupyter 
notebooks via iframe for visual equation creation.

The Equation Editor is a full-featured visual math editor with:
- Symbol palettes (Greek letters, operators, relations)
- Structural editing (fractions, integrals, matrices)
- Tensor notation with index handling
- Matrix and piecewise function builders
- Real-time SVG preview
- Export to EditorNode AST (for re-editing)
- Export to Typst, LaTeX, HTML, Unicode

Usage:
    from kleis_kernel.equation_editor import equation_editor
    
    # Open the equation editor
    equation_editor()
    
    # Open with an existing equation (for re-editing)
    equation_editor(initial={"Operation": {"name": "equals", ...}})

Prerequisites:
    The kleis server must be running:
    $ cargo run --bin kleis -- server --port 3000
"""

from IPython.display import display, HTML, Javascript
from typing import Optional, Dict, Any
import json
import uuid


# Default server port where kleis server runs
# This serves the Equation Editor at static/index.html
DEFAULT_KLEIS_PORT = 3000


def equation_editor(
    initial: Optional[Dict[str, Any]] = None,
    port: int = DEFAULT_KLEIS_PORT,
    width: str = "100%",
    height: str = "700px",
    show_immediately: bool = True
) -> HTML:
    """
    Display the Kleis Equation Editor in the Jupyter notebook.
    
    This embeds the existing Equation Editor (static/index.html) via iframe.
    The kleis server must be running to serve the editor.
    
    Args:
        initial: Optional EditorNode AST to pre-populate the editor
        port: Port where the kleis server is running (default: 3000)
        width: Width of the editor iframe
        height: Height of the editor iframe
        show_immediately: If True, show editor immediately; if False, show toggle button
    
    Returns:
        HTML widget that displays the equation editor
    
    Example:
        # Start the kleis server first:
        # $ cargo run --bin kleis -- server --port 3000
        
        # Open a blank editor
        equation_editor()
        
        # Edit an existing equation (will pre-populate the editor)
        eq_ast = {"Operation": {"name": "equals", "args": [...]}}
        equation_editor(initial=eq_ast)
    """
    widget_id = f"kleis-eq-editor-{uuid.uuid4().hex[:8]}"
    receiver_id = f"kleis-receiver-{uuid.uuid4().hex[:8]}"
    
    # URL for the equation editor (served by kleis server from static/index.html)
    # Adding ?mode=jupyter signals the editor to enable Jupyter-specific behavior
    editor_url = f"http://localhost:{port}/?mode=jupyter"
    
    initial_display = "block" if show_immediately else "none"
    button_display = "none" if show_immediately else "inline-block"
    
    html = f'''
    <div id="{widget_id}" style="border: 2px solid #667eea; border-radius: 8px; overflow: hidden; margin: 10px 0;">
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white;">
            <span style="font-weight: bold; font-size: 14px;">📐 Kleis Equation Editor</span>
            <div>
                <button id="{widget_id}-open-btn"
                        onclick="document.getElementById('{widget_id}-frame').style.display = 'block'; this.style.display='none'; document.getElementById('{widget_id}-close-btn').style.display='inline-block';" 
                        style="display: {button_display}; padding: 5px 15px; background: white; color: #667eea; border: none; border-radius: 4px; cursor: pointer; font-weight: bold;">
                    Open Editor
                </button>
                <button id="{widget_id}-close-btn"
                        onclick="document.getElementById('{widget_id}-frame').style.display = 'none'; document.getElementById('{widget_id}-open-btn').style.display='inline-block'; this.style.display='none';" 
                        style="display: {'inline-block' if show_immediately else 'none'}; padding: 5px 15px; background: rgba(255,255,255,0.2); color: white; border: 1px solid rgba(255,255,255,0.5); border-radius: 4px; cursor: pointer;">
                    Close
                </button>
            </div>
        </div>
        
        <iframe id="{widget_id}-frame" 
                src="{editor_url}"
                style="width: {width}; height: {height}; border: none; display: {initial_display};">
        </iframe>
        
        <div id="{receiver_id}" style="padding: 10px; background: #f8f9fa; border-top: 1px solid #ddd; display: none;">
            <strong>📋 Equation Data Received:</strong>
            <pre id="{receiver_id}-data" style="margin: 5px 0 0 0; padding: 10px; background: white; border-radius: 4px; white-space: pre-wrap; font-size: 12px; max-height: 200px; overflow: auto;"></pre>
        </div>
    </div>
    
    <script>
    (function() {{
        // Listen for messages from the Equation Editor iframe
        window.addEventListener('message', function(event) {{
            // Verify origin in production (should be localhost:port)
            
            // Handle equation data sent from editor
            if (event.data && event.data.type === 'kleisEquation') {{
                var receiver = document.getElementById('{receiver_id}');
                var dataDisplay = document.getElementById('{receiver_id}-data');
                
                if (receiver && dataDisplay) {{
                    receiver.style.display = 'block';
                    dataDisplay.textContent = JSON.stringify(event.data.payload, null, 2);
                    
                    // Store the equation data for Python access
                    window.kleisEquationData = window.kleisEquationData || {{}};
                    window.kleisEquationData['{widget_id}'] = event.data.payload;
                    
                    // Dispatch a custom event for Jupyter integration
                    var customEvent = new CustomEvent('kleisEquationReceived', {{
                        detail: {{
                            widgetId: '{widget_id}',
                            payload: event.data.payload
                        }}
                    }});
                    document.dispatchEvent(customEvent);
                    
                    console.log('Kleis Equation received:', event.data.payload);
                }}
            }}
            
            // Handle initial data request from editor
            if (event.data && event.data.type === 'kleisRequestInitial') {{
                var iframe = document.getElementById('{widget_id}-frame');
                if (iframe && iframe.contentWindow) {{
                    var initialData = {json.dumps(initial) if initial else 'null'};
                    if (initialData) {{
                        iframe.contentWindow.postMessage({{
                            type: 'kleisInitialData',
                            payload: initialData
                        }}, '*');
                    }}
                }}
            }}
        }});
        
        // Send initial data when iframe loads (if we have initial data)
        var iframe = document.getElementById('{widget_id}-frame');
        if (iframe) {{
            iframe.onload = function() {{
                var initialData = {json.dumps(initial) if initial else 'null'};
                if (initialData) {{
                    // Give the editor a moment to initialize
                    setTimeout(function() {{
                        iframe.contentWindow.postMessage({{
                            type: 'kleisInitialData',
                            payload: initialData
                        }}, '*');
                    }}, 500);
                }}
            }};
        }}
    }})();
    </script>
    '''
    
    return HTML(html)


class EquationEditorWidget:
    """
    A class-based wrapper for the Equation Editor widget.
    
    Provides a more object-oriented interface for working with the editor.
    
    Usage:
        editor = EquationEditorWidget()
        display(editor)  # Shows the editor
        
        # After user creates an equation, get the result
        # (requires Jupyter comms or polling window.kleisEquationData)
    """
    
    def __init__(
        self, 
        initial: Optional[Dict[str, Any]] = None, 
        port: int = DEFAULT_KLEIS_PORT,
        height: str = "700px"
    ):
        self.initial = initial
        self.port = port
        self.height = height
        self.result: Optional[Dict[str, Any]] = None
        self._widget_id = f"kleis-widget-{uuid.uuid4().hex[:8]}"
    
    def display(self) -> HTML:
        """Display the equation editor."""
        return equation_editor(
            initial=self.initial, 
            port=self.port,
            height=self.height
        )
    
    def get_result(self) -> Optional[Dict[str, Any]]:
        """
        Get the equation result after user interaction.
        
        Note: In the current implementation, this requires checking
        window.kleisEquationData[widget_id] in JavaScript.
        
        A full implementation would use Jupyter comms for bidirectional
        communication between Python and JavaScript.
        """
        return self.result
    
    def _repr_html_(self) -> str:
        """HTML representation for Jupyter."""
        return equation_editor(
            initial=self.initial, 
            port=self.port,
            height=self.height
        )._repr_html_()


def check_server(port: int = DEFAULT_KLEIS_PORT) -> bool:
    """
    Check if the kleis server is running.
    
    Args:
        port: Port to check (default: 3000)
    
    Returns:
        True if server is accessible, False otherwise
    """
    import urllib.request
    try:
        urllib.request.urlopen(f"http://localhost:{port}/", timeout=2)
        return True
    except Exception:
        return False


def start_server_instructions() -> str:
    """Return instructions for starting the kleis server."""
    return """
To use the Equation Editor, start the kleis server:

    cd /path/to/kleis
    cargo run --bin kleis -- server --port 3000

Then call equation_editor() again.
"""
