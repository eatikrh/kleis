"""
KleisDoc - Document creation and management for Jupyter notebooks.

This module provides the Python API for creating, editing, and exporting
publishable documents from Jupyter notebooks.

Usage:
    from kleis_kernel.kleisdoc import KleisDoc, templates

    # Create from template
    thesis = KleisDoc.from_template("MIT Thesis")

    # Set metadata
    thesis.set_metadata(
        title="My Thesis",
        author="Jane Smith",
        date="May 2025"
    )

    # Add content
    thesis.add_chapter("Introduction", "This thesis presents...")

    # Export
    thesis.export_pdf("thesis.pdf")
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
    """A document section (chapter, section, subsection)."""
    level: int  # 1=chapter, 2=section, 3=subsection
    title: str
    content: List[Any] = field(default_factory=list)  # Text, equations, figures


class KleisDoc:
    """
    A Kleis document with structured content, equations, and figures.
    
    Supports:
    - Template-based creation (MIT Thesis, arXiv Paper, etc.)
    - Metadata management (title, authors, date)
    - Content organization (chapters, sections)
    - Equation management (with EditorNode AST for re-editing)
    - Figure management (with Kleis code for regeneration)
    - Export to PDF, Typst, LaTeX, HTML
    - Persistence to .kleis files
    """
    
    def __init__(self):
        self.metadata: Dict[str, Any] = {}
        self.template_name: Optional[str] = None
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
    def from_template(cls, template_name: str) -> "KleisDoc":
        """Create a document from a template.
        
        Available templates:
        - "MIT Thesis"
        - "arXiv Paper"
        - "IEEE Paper"
        - "Book Chapter"
        """
        doc = cls()
        doc.template_name = template_name
        
        # Load template structure
        if template_name == "MIT Thesis":
            doc._init_mit_thesis_template()
        elif template_name == "arXiv Paper":
            doc._init_arxiv_template()
        else:
            raise ValueError(f"Unknown template: {template_name}")
        
        return doc
    
    @classmethod
    def load(cls, path: str) -> "KleisDoc":
        """Load a document from a .kleis file."""
        doc = cls()
        # TODO: Parse .kleis file and populate doc
        raise NotImplementedError("Loading from .kleis files not yet implemented")
    
    def _find_kleis_binary(self) -> Optional[str]:
        """Find the kleis binary path."""
        # Try common locations
        candidates = [
            "kleis",  # In PATH
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
    
    def _init_mit_thesis_template(self):
        """Initialize MIT Thesis template structure."""
        self.metadata = {
            "type": "PhD Thesis",
            "institution": "Massachusetts Institute of Technology",
            "degree": "Doctor of Philosophy",
        }
        # Create default chapter structure
        self.sections = [
            Section(level=1, title="Introduction"),
            Section(level=1, title="Background"),
            Section(level=1, title="Methodology"),
            Section(level=1, title="Results"),
            Section(level=1, title="Conclusion"),
        ]
    
    def _init_arxiv_template(self):
        """Initialize arXiv Paper template structure."""
        self.metadata = {
            "type": "Research Paper",
        }
        self.sections = [
            Section(level=1, title="Abstract"),
            Section(level=1, title="Introduction"),
            Section(level=1, title="Related Work"),
            Section(level=1, title="Method"),
            Section(level=1, title="Experiments"),
            Section(level=1, title="Conclusion"),
        ]
    
    # =========================================================================
    # Metadata Management
    # =========================================================================
    
    def set_metadata(self, **kwargs):
        """Set document metadata.
        
        Common fields:
        - title: Document title
        - author: Author name or list of Author objects
        - email: Author email
        - date: Publication date
        - department: Academic department
        - degree: Degree type
        - supervisor: Thesis supervisor
        - keywords: List of keywords
        """
        self.metadata.update(kwargs)
    
    def get_metadata(self) -> Dict[str, Any]:
        """Get all metadata."""
        return self.metadata.copy()
    
    # =========================================================================
    # Content Management
    # =========================================================================
    
    def add_chapter(self, title: str, content: str = "") -> Section:
        """Add a new chapter to the document."""
        chapter = Section(level=1, title=title, content=[content] if content else [])
        self.sections.append(chapter)
        return chapter
    
    def add_section(self, chapter_index: int, title: str, content: str = "") -> Section:
        """Add a section to a chapter."""
        section = Section(level=2, title=title, content=[content] if content else [])
        if chapter_index < len(self.sections):
            self.sections[chapter_index].content.append(section)
        return section
    
    def add_text(self, text: str, chapter: int = -1, section: int = -1):
        """Add text content to the document."""
        if chapter == -1:
            # Add to last chapter
            if self.sections:
                self.sections[-1].content.append(text)
            else:
                # Create a default chapter
                self.add_chapter("Content", text)
        else:
            if section == -1:
                self.sections[chapter].content.append(text)
            else:
                # Add to specific section within chapter
                pass  # TODO
    
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
    
    def export_pdf(self, output_path: str) -> bool:
        """Export document to PDF via Typst.
        
        Args:
            output_path: Path for the output PDF file
        
        Returns:
            True if export succeeded
        """
        # First export to Typst
        typst_path = output_path.replace(".pdf", ".typ")
        if not self.export_typst(typst_path):
            return False
        
        # Compile Typst to PDF
        try:
            result = subprocess.run(
                ["typst", "compile", typst_path, output_path],
                capture_output=True, text=True
            )
            return result.returncode == 0
        except FileNotFoundError:
            print("Error: typst not found. Install with: cargo install typst-cli")
            return False
    
    def export_typst(self, output_path: str) -> bool:
        """Export document to Typst source.
        
        Args:
            output_path: Path for the output .typ file
        
        Returns:
            True if export succeeded
        """
        typst_code = self._generate_typst()
        with open(output_path, "w") as f:
            f.write(typst_code)
        return True
    
    def export_latex(self, output_dir: str) -> bool:
        """Export document to LaTeX (for arXiv submission).
        
        Creates a directory with main.tex and supporting files.
        """
        # TODO: Implement LaTeX export
        raise NotImplementedError("LaTeX export not yet implemented")
    
    def export_html(self, output_dir: str) -> bool:
        """Export document to HTML.
        
        Creates a directory with index.html and supporting files.
        """
        # TODO: Implement HTML export
        raise NotImplementedError("HTML export not yet implemented")
    
    def _generate_typst(self) -> str:
        """Generate Typst code for the document."""
        lines = []
        
        # Document setup
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
        
        # Page setup
        lines.append('#set page(paper: "us-letter", margin: 1in)')
        lines.append('#set text(font: "New Computer Modern", size: 11pt)')
        lines.append('')
        
        # Title
        if "title" in self.metadata:
            lines.append(f'#align(center)[')
            lines.append(f'  #text(size: 24pt, weight: "bold")[{self.metadata["title"]}]')
            lines.append(f']')
            lines.append('')
        
        # Author
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
        
        # Heading
        lines.append(f'#heading(level: {section.level})[{section.title}]')
        lines.append('')
        
        # Content
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
                elif item.typst_fragment:
                    lines.append(f'#figure(')
                    lines.append(item.typst_fragment)
                    lines.append(f'  caption: [{item.caption}]')
                    lines.append(f') <{item.label}>')
                lines.append('')
        
        return '\n'.join(lines)
    
    # =========================================================================
    # Persistence
    # =========================================================================
    
    def save(self, path: str):
        """Save document to a .kleis file."""
        # TODO: Generate .kleis code and save
        raise NotImplementedError("Saving to .kleis files not yet implemented")
    
    # =========================================================================
    # Display (for Jupyter)
    # =========================================================================
    
    def _repr_html_(self) -> str:
        """HTML representation for Jupyter display."""
        html = ['<div class="kleisdoc">']
        html.append(f'<h2>📄 {self.metadata.get("title", "Untitled Document")}</h2>')
        
        if self.template_name:
            html.append(f'<p><em>Template: {self.template_name}</em></p>')
        
        # Structure overview
        html.append('<h3>Structure</h3>')
        html.append('<ul>')
        for section in self.sections:
            html.append(f'<li>{section.title} ({len(section.content)} items)</li>')
        html.append('</ul>')
        
        # Counts
        html.append('<h3>Content</h3>')
        html.append(f'<p>📝 {len(self.sections)} chapters/sections</p>')
        html.append(f'<p>🔢 {len(self.equations)} equations</p>')
        html.append(f'<p>📊 {len(self.figures)} figures</p>')
        
        html.append('</div>')
        return '\n'.join(html)


class Templates:
    """Template registry for KleisDoc."""
    
    @staticmethod
    def list() -> List[str]:
        """List available document templates."""
        return [
            "MIT Thesis",
            "arXiv Paper",
            "IEEE Paper",
            "Book Chapter",
        ]
    
    @staticmethod
    def info(template_name: str) -> Dict[str, Any]:
        """Get information about a template."""
        templates = {
            "MIT Thesis": {
                "name": "MIT Thesis",
                "description": "PhD thesis format for MIT",
                "required": ["Title", "Abstract (≤350 words)", "3+ chapters", "Bibliography"],
                "optional": ["Acknowledgments", "Appendices"],
                "style": "US Letter, 1\" margins, New Computer Modern font",
            },
            "arXiv Paper": {
                "name": "arXiv Paper",
                "description": "Standard research paper for arXiv",
                "required": ["Title", "Abstract", "Introduction", "Conclusion"],
                "optional": ["Appendices"],
                "style": "US Letter, standard article format",
            },
        }
        return templates.get(template_name, {})


# Module-level instance for convenience
templates = Templates()

