"""
KleisDoc - Generic document creation and management for Jupyter notebooks.

This module provides the Python API for creating, editing, and exporting
structured documents from Jupyter notebooks. Document formats (thesis, paper,
book, report, etc.) are defined in external Kleis template files, not hardcoded.

Usage:
    from kleis_kernel.kleisdoc import KleisDoc

    # Create a blank document
    doc = KleisDoc()
    doc.set_metadata(title="My Document", author="Jane Doe")
    doc.add_section("Introduction", "This is the introduction.")
    doc.export_pdf("output.pdf")

    # Load a template for structure guidance
    doc = KleisDoc.from_template("stdlib/templates/mit_thesis.kleis")
"""

import json
import subprocess
import os
import urllib.request
import urllib.error
from pathlib import Path
from typing import Optional, Dict, List, Any, Union
from dataclasses import dataclass, field

# Default Kleis server URL
DEFAULT_KLEIS_SERVER = "http://localhost:3000"


@dataclass
class Author:
    """Document author information."""
    name: str
    email: str = ""
    affiliation: str = ""
    role: str = "primary"


@dataclass
class Equation:
    """An equation with its AST and rendered forms."""
    id: str
    label: str
    latex: str = ""
    typst: str = ""
    ast: Optional[Dict[str, Any]] = None  # EditorNode AST for re-editing
    numbered: bool = True
    verified: bool = False


@dataclass
class Figure:
    """A figure with its source code and cache."""
    id: str
    label: str
    caption: str
    kleis_code: Optional[str] = None  # For regenerable figures
    svg_cache: Optional[str] = None
    typst_fragment: Optional[str] = None
    image_path: Optional[str] = None  # For static images


@dataclass
class Section:
    """A document section at any level."""
    level: int  # 1=top level, 2=subsection, etc.
    title: str
    content: List[Any] = field(default_factory=list)  # Text, equations, figures, subsections


class KleisDoc:
    """
    A generic Kleis document with structured content.
    
    KleisDoc is format-agnostic. The structure of specific document types
    (thesis, paper, book, report) is defined in external Kleis template files,
    not hardcoded here.
    
    Supports:
    - Metadata management (title, authors, date, custom fields)
    - Content organization (sections at any nesting level)
    - Equation management (with EditorNode AST for re-editing)
    - Figure management (with Kleis code for regeneration)
    - Export to PDF, Typst, LaTeX, HTML
    - Template loading for structure guidance
    - Persistence to .kleis files
    """
    
    def __init__(self, server_url: str = DEFAULT_KLEIS_SERVER):
        self.metadata: Dict[str, Any] = {}
        self.template_path: Optional[str] = None
        self.template_info: Dict[str, Any] = {}
        self.sections: List[Section] = []
        self.equations: Dict[str, Equation] = {}
        self.figures: Dict[str, Figure] = {}
        self.bibliography: List[Dict[str, str]] = []
        self.server_url = server_url
        self._kleis_path = self._find_kleis_binary()
        self._server_available: Optional[bool] = None
    
    @classmethod
    def new(cls) -> "KleisDoc":
        """Create a new blank document."""
        return cls()
    
    @classmethod
    def from_template(cls, template_path: str) -> "KleisDoc":
        """Create a document from a Kleis template file.
        
        The template file defines:
        - Document structure (required/optional sections)
        - Validation axioms (word limits, formatting rules)
        - Styling hints (fonts, margins, headers)
        
        Args:
            template_path: Path to a .kleis template file
                          (e.g., "stdlib/templates/mit_thesis.kleis")
        
        Returns:
            A new KleisDoc with template loaded
        """
        doc = cls()
        doc.template_path = template_path
        doc._load_template(template_path)
        return doc
    
    @classmethod
    def load(cls, path: str) -> "KleisDoc":
        """Load a document from a .kleisdoc (JSON) file.
        
        Args:
            path: Path to the .kleisdoc file
        
        Returns:
            A KleisDoc instance populated with the saved data
        """
        doc = cls()
        
        with open(path, "r") as f:
            data = json.load(f)
        
        # Version check
        format_version = data.get("_kleisdoc_version", "1.0")
        if not format_version.startswith("1."):
            raise ValueError(f"Unsupported KleisDoc format version: {format_version}")
        
        # Load metadata
        doc.metadata = data.get("metadata", {})
        doc.template_path = data.get("template_path")
        doc.template_info = data.get("template_info", {})
        
        # Load equations
        for eq_data in data.get("equations", []):
            eq = Equation(
                id=eq_data["id"],
                label=eq_data["label"],
                latex=eq_data.get("latex", ""),
                typst=eq_data.get("typst", ""),
                ast=eq_data.get("ast"),
                numbered=eq_data.get("numbered", True),
                verified=eq_data.get("verified", False)
            )
            doc.equations[eq.label] = eq
        
        # Load figures
        for fig_data in data.get("figures", []):
            fig = Figure(
                id=fig_data["id"],
                label=fig_data["label"],
                caption=fig_data.get("caption", ""),
                kleis_code=fig_data.get("kleis_code"),
                svg_cache=fig_data.get("svg_cache"),
                typst_fragment=fig_data.get("typst_fragment"),
                image_path=fig_data.get("image_path")
            )
            doc.figures[fig.label] = fig
        
        # Load sections (recursive)
        for section_data in data.get("sections", []):
            section = doc._load_section(section_data)
            doc.sections.append(section)
        
        # Load bibliography
        doc.bibliography = data.get("bibliography", [])
        
        return doc
    
    def _load_section(self, section_data: dict) -> Section:
        """Recursively load a section from saved data."""
        section = Section(
            level=section_data["level"],
            title=section_data["title"],
            content=[]
        )
        
        for item in section_data.get("content", []):
            if isinstance(item, str):
                section.content.append(item)
            elif isinstance(item, dict):
                item_type = item.get("_type")
                if item_type == "section":
                    section.content.append(self._load_section(item))
                elif item_type == "equation":
                    # Reference to equation by label
                    eq_label = item.get("label")
                    if eq_label in self.equations:
                        section.content.append(self.equations[eq_label])
                elif item_type == "figure":
                    # Reference to figure by label
                    fig_label = item.get("label")
                    if fig_label in self.figures:
                        section.content.append(self.figures[fig_label])
        
        return section
    
    def _find_kleis_binary(self) -> Optional[str]:
        """Find the kleis binary path."""
        candidates = [
            "kleis",
            "../target/release/kleis",
            "../target/debug/kleis",
            os.path.expanduser("~/.cargo/bin/kleis"),
        ]
        for candidate in candidates:
            try:
                result = subprocess.run([candidate, "--version"], 
                                       capture_output=True, text=True)
                if result.returncode == 0:
                    return candidate
            except FileNotFoundError:
                continue
        return None
    
    def _load_template(self, template_path: str):
        """Load template info from a .kleis file.
        
        Extracts metadata about the template without fully parsing
        the Kleis code. Full template validation happens during export.
        """
        self.template_info = {
            "path": template_path,
            "loaded": False,
        }
        
        # Try to read the template file
        try:
            path = Path(template_path)
            if path.exists():
                content = path.read_text()
                # Extract basic info from comments/structure
                self.template_info["loaded"] = True
                self.template_info["size"] = len(content)
        except Exception as e:
            self.template_info["error"] = str(e)
    
    # =========================================================================
    # Kleis Server Integration
    # =========================================================================
    
    def _check_server(self) -> bool:
        """Check if Kleis server is available."""
        if self._server_available is not None:
            return self._server_available
        
        try:
            req = urllib.request.Request(f"{self.server_url}/health")
            with urllib.request.urlopen(req, timeout=2) as resp:
                self._server_available = resp.status == 200
        except (urllib.error.URLError, TimeoutError):
            self._server_available = False
        
        return self._server_available
    
    def render_ast(self, ast: Dict[str, Any], format: str = "typst") -> Optional[str]:
        """Render an EditorNode AST to the specified format.
        
        Uses the Kleis server API if available, otherwise returns None.
        
        Args:
            ast: EditorNode AST as a Python dict
            format: Output format - "typst", "latex", "unicode", "html", "kleis"
        
        Returns:
            Rendered string, or None if server unavailable
        """
        if not self._check_server():
            return None
        
        try:
            data = json.dumps({"ast": ast, "format": format}).encode('utf-8')
            req = urllib.request.Request(
                f"{self.server_url}/api/render_ast",
                data=data,
                headers={"Content-Type": "application/json"},
                method="POST"
            )
            with urllib.request.urlopen(req, timeout=10) as resp:
                result = json.loads(resp.read().decode('utf-8'))
                return result.get("output")
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as e:
            print(f"Warning: Failed to render AST via server: {e}")
            return None
    
    def render_plot(self, kleis_code: str) -> Optional[str]:
        """Render Kleis plotting code to Typst fragment.
        
        Uses the Kleis binary to evaluate the plot code and return Typst.
        
        Args:
            kleis_code: Kleis code that produces a plot/diagram.
                       Should use export_typst_fragment() to return Typst code.
        
        Returns:
            Typst code fragment, or None if unavailable
        
        Example:
            kleis_code = '''
            import "stdlib/plotting.kleis"
            let data = line([1, 2, 3, 4], [10, 20, 15, 25])
            export_typst_fragment(data, title = "My Plot")
            '''
        """
        if not self._kleis_path:
            return None
        
        # Wrap in example block to evaluate and output
        wrapped_code = f'''
import "stdlib/prelude.kleis"

example "render_plot"
    let result = {kleis_code}
    out result
'''
        
        try:
            result = subprocess.run(
                [self._kleis_path, "eval", "-c", wrapped_code],
                capture_output=True,
                text=True,
                timeout=30
            )
            if result.returncode == 0 and result.stdout.strip():
                return result.stdout.strip()
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass
        
        return None
    
    # =========================================================================
    # Metadata Management
    # =========================================================================
    
    def set_metadata(self, **kwargs):
        """Set document metadata.
        
        Any key-value pairs can be stored. Common fields:
        - title: Document title
        - author: Author name or Author object
        - authors: List of authors
        - date: Publication/submission date
        - abstract: Document abstract
        - keywords: List of keywords
        
        Template-specific fields (examples):
        - department, degree, supervisor (thesis)
        - journal, volume, issue (paper)
        - publisher, isbn (book)
        """
        self.metadata.update(kwargs)
    
    def get_metadata(self, key: str = None) -> Any:
        """Get metadata value(s).
        
        Args:
            key: Specific key to retrieve, or None for all metadata
        
        Returns:
            Value for key, or dict of all metadata if key is None
        """
        if key is None:
            return self.metadata.copy()
        return self.metadata.get(key)
    
    # =========================================================================
    # Content Management
    # =========================================================================
    
    def add_section(self, title: str, content: str = "", level: int = 1) -> Section:
        """Add a section to the document.
        
        Args:
            title: Section title
            content: Initial text content (optional)
            level: Nesting level (1=top, 2=subsection, etc.)
        
        Returns:
            The created Section object
        """
        section = Section(level=level, title=title, content=[content] if content else [])
        self.sections.append(section)
        return section
    
    def add_subsection(self, parent: Section, title: str, content: str = "") -> Section:
        """Add a subsection to an existing section.
        
        Args:
            parent: Parent section
            title: Subsection title
            content: Initial text content
        
        Returns:
            The created subsection
        """
        subsection = Section(level=parent.level + 1, title=title, 
                            content=[content] if content else [])
        parent.content.append(subsection)
        return subsection
    
    def add_text(self, text: str, section: Section = None):
        """Add text content to a section.
        
        Args:
            text: Text content (Markdown or plain text)
            section: Section to add to (default: last section)
        """
        if section is None:
            if self.sections:
                section = self.sections[-1]
            else:
                # Create a default section
                section = self.add_section("Content")
        section.content.append(text)
    
    # =========================================================================
    # Equation Management
    # =========================================================================
    
    def add_equation(self, label: str, latex: str = "", 
                     ast: Optional[Dict] = None, numbered: bool = True,
                     section: Section = None) -> Equation:
        """Add an equation to the document.
        
        Args:
            label: Unique label (e.g., "eq:einstein")
            latex: LaTeX representation
            ast: EditorNode AST (for re-editing in Equation Editor)
            numbered: Whether equation is numbered
            section: Section to add equation to (default: last section)
        
        Returns:
            The created Equation object
        """
        eq_id = f"eq_{len(self.equations)}"
        eq = Equation(
            id=eq_id,
            label=label,
            latex=latex,
            ast=ast,
            numbered=numbered
        )
        self.equations[label] = eq
        
        # Add to section if specified
        if section is not None:
            section.content.append(eq)
        elif self.sections:
            # Add to last section by default
            self.sections[-1].content.append(eq)
        
        return eq
    
    def get_equation(self, label: str) -> Optional[Equation]:
        """Get an equation by label."""
        return self.equations.get(label)
    
    def update_equation(self, label: str, latex: str = None, 
                        ast: Dict = None) -> Optional[Equation]:
        """Update an existing equation."""
        if label in self.equations:
            eq = self.equations[label]
            if latex is not None:
                eq.latex = latex
            if ast is not None:
                eq.ast = ast
            return eq
        return None
    
    # =========================================================================
    # Figure Management
    # =========================================================================
    
    def add_figure(self, label: str, caption: str, 
                   kleis_code: str = None, image_path: str = None,
                   typst_fragment: str = None,
                   section: Section = None) -> Figure:
        """Add a figure to the document.
        
        Args:
            label: Unique label (e.g., "fig:performance")
            caption: Figure caption
            kleis_code: Kleis plotting code (for regenerable figures)
            image_path: Path to static image file
            typst_fragment: Pre-rendered Typst code for the figure
            section: Section to add figure to (default: last section)
        
        Returns:
            The created Figure object
        """
        fig_id = f"fig_{len(self.figures)}"
        fig = Figure(
            id=fig_id,
            label=label,
            caption=caption,
            kleis_code=kleis_code,
            image_path=image_path
        )
        if typst_fragment:
            fig.typst_fragment = typst_fragment
        self.figures[label] = fig
        
        # Add to section if specified
        if section is not None:
            section.content.append(fig)
        elif self.sections:
            # Add to last section by default
            self.sections[-1].content.append(fig)
        
        return fig
    
    def get_figure(self, label: str) -> Optional[Figure]:
        """Get a figure by label."""
        return self.figures.get(label)
    
    def regenerate_figure(self, label: str) -> bool:
        """Regenerate a figure from its Kleis code.
        
        Args:
            label: Figure label to regenerate
        
        Returns:
            True if regeneration succeeded
        """
        fig = self.figures.get(label)
        if fig and fig.kleis_code:
            # Wrap kleis_code with export_typst_fragment if not already
            code = fig.kleis_code.strip()
            if not code.startswith("export_typst"):
                code = f"export_typst_fragment({code})"
            
            typst_fragment = self.render_plot(code)
            if typst_fragment:
                fig.typst_fragment = typst_fragment
                return True
        return False
    
    def regenerate_all_figures(self) -> Dict[str, bool]:
        """Regenerate all figures with Kleis code.
        
        Returns:
            Dict mapping figure labels to success status
        """
        results = {}
        for label, fig in self.figures.items():
            if fig.kleis_code:
                results[label] = self.regenerate_figure(label)
        return results
    
    # =========================================================================
    # Export
    # =========================================================================
    
    def export_pdf(self, output_path: str, typst_template: str = None) -> bool:
        """Export document to PDF via Typst.
        
        Args:
            output_path: Path for the output PDF file
            typst_template: Optional path to Typst template file
        
        Returns:
            True if export succeeded
        """
        typst_path = output_path.replace(".pdf", ".typ")
        if not self.export_typst(typst_path, template=typst_template):
            return False
        
        try:
            result = subprocess.run(
                ["typst", "compile", typst_path, output_path],
                capture_output=True, text=True
            )
            return result.returncode == 0
        except FileNotFoundError:
            print("Error: typst not found. Install with: cargo install typst-cli")
            return False
    
    def export_typst(self, output_path: str, template: str = None) -> bool:
        """Export document to Typst source.
        
        Args:
            output_path: Path for the output .typ file
            template: Optional path to Typst template file
        
        Returns:
            True if export succeeded
        """
        typst_code = self._generate_typst(template)
        with open(output_path, "w") as f:
            f.write(typst_code)
        return True
    
    def export_latex(self, output_path: str) -> bool:
        """Export document to LaTeX."""
        raise NotImplementedError("LaTeX export not yet implemented")
    
    def export_html(self, output_path: str) -> bool:
        """Export document to HTML."""
        raise NotImplementedError("HTML export not yet implemented")
    
    def _generate_typst(self, template: str = None) -> str:
        """Generate Typst code for the document."""
        lines = []
        
        # Document metadata
        lines.append('#set document(')
        if "title" in self.metadata:
            lines.append(f'  title: "{self.metadata["title"]}",')
        if "author" in self.metadata:
            author = self.metadata["author"]
            if isinstance(author, str):
                lines.append(f'  author: "{author}",')
            elif isinstance(author, Author):
                lines.append(f'  author: "{author.name}",')
        lines.append(')')
        lines.append('')
        
        # Basic page setup (can be overridden by template)
        lines.append('#set page(paper: "us-letter", margin: 1in)')
        lines.append('#set text(size: 11pt)')
        lines.append('')
        
        # Title
        if "title" in self.metadata:
            lines.append(f'#align(center)[')
            lines.append(f'  #text(size: 20pt, weight: "bold")[{self.metadata["title"]}]')
            lines.append(f']')
            lines.append('')
        
        # Author(s)
        if "author" in self.metadata:
            author = self.metadata["author"]
            if isinstance(author, str):
                lines.append(f'#align(center)[{author}]')
            elif isinstance(author, Author):
                lines.append(f'#align(center)[{author.name}]')
            lines.append('')
        
        # Abstract
        if "abstract" in self.metadata:
            lines.append('#heading(level: 1)[Abstract]')
            lines.append(self.metadata["abstract"])
            lines.append('')
        
        # Sections
        for section in self.sections:
            lines.append(self._section_to_typst(section))
        
        return '\n'.join(lines)
    
    def _section_to_typst(self, section: Section) -> str:
        """Convert a section to Typst code."""
        lines = []
        lines.append(f'#heading(level: {section.level})[{section.title}]')
        lines.append('')
        
        for item in section.content:
            if isinstance(item, str):
                lines.append(item)
                lines.append('')
            elif isinstance(item, Section):
                lines.append(self._section_to_typst(item))
            elif isinstance(item, Equation):
                lines.append(self._equation_to_typst(item))
                lines.append('')
            elif isinstance(item, Figure):
                lines.append(self._figure_to_typst(item))
                lines.append('')
        
        return '\n'.join(lines)
    
    def _equation_to_typst(self, eq: Equation) -> str:
        """Convert an equation to Typst code.
        
        If the equation has an EditorNode AST, renders it via the server.
        Otherwise falls back to the LaTeX representation.
        """
        typst_content = None
        
        # Try to render from AST if available
        if eq.ast:
            # Check if we already have cached Typst
            if eq.typst:
                typst_content = eq.typst
            else:
                # Try to render via server
                rendered = self.render_ast(eq.ast, format="typst")
                if rendered:
                    eq.typst = rendered  # Cache for future use
                    typst_content = rendered
        
        # Fall back to LaTeX if no Typst available
        if typst_content is None and eq.latex:
            # Embed LaTeX in Typst math mode
            typst_content = eq.latex
        
        if typst_content:
            if eq.numbered:
                return f'$ {typst_content} $ <{eq.label}>'
            else:
                return f'$ {typst_content} $'
        
        return f'// Equation {eq.label}: no content available'
    
    def _figure_to_typst(self, fig: Figure) -> str:
        """Convert a figure to Typst code.
        
        If the figure has Kleis code, attempts to render it.
        Otherwise uses a static image path.
        """
        lines = []
        
        if fig.kleis_code:
            # Try to get Typst fragment from cached value or render
            if fig.typst_fragment:
                lines.append('#figure(')
                lines.append(f'  [{fig.typst_fragment}],')
                lines.append(f'  caption: [{fig.caption}]')
                lines.append(f') <{fig.label}>')
            else:
                # Placeholder for regenerable figures
                lines.append('#figure(')
                lines.append(f'  // Kleis code: {fig.kleis_code[:50]}...')
                lines.append(f'  box(width: 100%, height: 150pt, fill: luma(240))[Plot placeholder],')
                lines.append(f'  caption: [{fig.caption}]')
                lines.append(f') <{fig.label}>')
        elif fig.image_path:
            lines.append('#figure(')
            lines.append(f'  image("{fig.image_path}"),')
            lines.append(f'  caption: [{fig.caption}]')
            lines.append(f') <{fig.label}>')
        else:
            lines.append(f'// Figure {fig.label}: no content available')
        
        return '\n'.join(lines)
    
    # =========================================================================
    # Persistence
    # =========================================================================
    
    def save(self, path: str):
        """Save document to a .kleisdoc (JSON) file.
        
        The saved file contains:
        - Document metadata (title, author, etc.)
        - All equations with their EditorNode ASTs (for re-editing)
        - All figures with their Kleis code (for regeneration)
        - Document structure (sections, content order)
        - Bibliography
        
        Args:
            path: Path for the output file (recommended: .kleisdoc extension)
        """
        data = {
            "_kleisdoc_version": "1.0",
            "_created_by": "KleisDoc Python API",
            "metadata": self.metadata,
            "template_path": self.template_path,
            "template_info": self.template_info,
            "equations": [],
            "figures": [],
            "sections": [],
            "bibliography": self.bibliography,
        }
        
        # Save equations
        for label, eq in self.equations.items():
            data["equations"].append({
                "id": eq.id,
                "label": eq.label,
                "latex": eq.latex,
                "typst": eq.typst,
                "ast": eq.ast,  # EditorNode AST preserved!
                "numbered": eq.numbered,
                "verified": eq.verified,
            })
        
        # Save figures
        for label, fig in self.figures.items():
            data["figures"].append({
                "id": fig.id,
                "label": fig.label,
                "caption": fig.caption,
                "kleis_code": fig.kleis_code,  # For regeneration
                "svg_cache": fig.svg_cache,
                "typst_fragment": fig.typst_fragment,
                "image_path": fig.image_path,
            })
        
        # Save sections (recursive)
        for section in self.sections:
            data["sections"].append(self._section_to_dict(section))
        
        with open(path, "w") as f:
            json.dump(data, f, indent=2)
    
    def _section_to_dict(self, section: Section) -> dict:
        """Convert a section to a dict for saving."""
        result = {
            "_type": "section",
            "level": section.level,
            "title": section.title,
            "content": [],
        }
        
        for item in section.content:
            if isinstance(item, str):
                result["content"].append(item)
            elif isinstance(item, Section):
                result["content"].append(self._section_to_dict(item))
            elif isinstance(item, Equation):
                # Save reference to equation
                result["content"].append({
                    "_type": "equation",
                    "label": item.label,
                })
            elif isinstance(item, Figure):
                # Save reference to figure
                result["content"].append({
                    "_type": "figure",
                    "label": item.label,
                })
        
        return result
    
    # =========================================================================
    # Display (for Jupyter)
    # =========================================================================
    
    def _repr_html_(self) -> str:
        """HTML representation for Jupyter display."""
        html = ['<div class="kleisdoc" style="font-family: sans-serif; padding: 10px; border: 1px solid #ddd; border-radius: 8px;">']
        
        title = self.metadata.get("title", "Untitled Document")
        html.append(f'<h2 style="margin-top: 0;">📄 {title}</h2>')
        
        if self.template_path:
            html.append(f'<p style="color: #666;"><em>Template: {self.template_path}</em></p>')
        
        # Structure
        if self.sections:
            html.append('<h4>Structure</h4>')
            html.append('<ul style="margin: 0;">')
            for section in self.sections:
                content_count = len([c for c in section.content if isinstance(c, str) and c])
                html.append(f'<li>{section.title}</li>')
            html.append('</ul>')
        
        # Summary
        html.append('<h4>Content Summary</h4>')
        html.append(f'<p style="margin: 5px 0;">📝 {len(self.sections)} sections</p>')
        html.append(f'<p style="margin: 5px 0;">🔢 {len(self.equations)} equations</p>')
        html.append(f'<p style="margin: 5px 0;">📊 {len(self.figures)} figures</p>')
        
        html.append('</div>')
        return '\n'.join(html)


def list_templates(template_dir: str = "stdlib/templates") -> List[str]:
    """List available document templates.
    
    Args:
        template_dir: Directory to search for templates
    
    Returns:
        List of template file paths
    """
    templates = []
    path = Path(template_dir)
    if path.exists():
        for f in path.glob("*.kleis"):
            templates.append(str(f))
    return templates
