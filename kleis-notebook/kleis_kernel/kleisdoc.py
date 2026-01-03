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
from pathlib import Path
from typing import Optional, Dict, List, Any, Union
from dataclasses import dataclass, field


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
    
    def __init__(self):
        self.metadata: Dict[str, Any] = {}
        self.template_path: Optional[str] = None
        self.template_info: Dict[str, Any] = {}
        self.sections: List[Section] = []
        self.equations: Dict[str, Equation] = {}
        self.figures: Dict[str, Figure] = {}
        self.bibliography: List[Dict[str, str]] = []
        self._kleis_path = self._find_kleis_binary()
    
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
        """Load a document from a .kleis file."""
        doc = cls()
        # TODO: Parse .kleis file and populate doc
        raise NotImplementedError("Loading from .kleis files not yet implemented")
    
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
                     ast: Optional[Dict] = None, numbered: bool = True) -> Equation:
        """Add an equation to the document.
        
        Args:
            label: Unique label (e.g., "eq:einstein")
            latex: LaTeX representation
            ast: EditorNode AST (for re-editing in Equation Editor)
            numbered: Whether equation is numbered
        
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
                   kleis_code: str = None, image_path: str = None) -> Figure:
        """Add a figure to the document.
        
        Args:
            label: Unique label (e.g., "fig:performance")
            caption: Figure caption
            kleis_code: Kleis plotting code (for regenerable figures)
            image_path: Path to static image file
        
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
        self.figures[label] = fig
        return fig
    
    def get_figure(self, label: str) -> Optional[Figure]:
        """Get a figure by label."""
        return self.figures.get(label)
    
    def regenerate_figure(self, label: str) -> bool:
        """Regenerate a figure from its Kleis code."""
        fig = self.figures.get(label)
        if fig and fig.kleis_code:
            # TODO: Call kleis to generate SVG/Typst
            return True
        return False
    
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
                lines.append(f'$ {item.latex} $ <{item.label}>')
                lines.append('')
            elif isinstance(item, Figure):
                if item.image_path:
                    lines.append(f'#figure(')
                    lines.append(f'  image("{item.image_path}"),')
                    lines.append(f'  caption: [{item.caption}]')
                    lines.append(f') <{item.label}>')
                lines.append('')
        
        return '\n'.join(lines)
    
    # =========================================================================
    # Persistence
    # =========================================================================
    
    def save(self, path: str):
        """Save document to a .kleis file."""
        raise NotImplementedError("Saving to .kleis files not yet implemented")
    
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
