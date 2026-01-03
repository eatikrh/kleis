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
    doc = KleisDoc.from_template("stdlib/templates/article.kleis")
"""

import json
import re
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


@dataclass
class BibEntry:
    """A bibliography entry."""
    key: str  # Citation key (e.g., "einstein1905")
    entry_type: str  # article, book, inproceedings, etc.
    title: str
    authors: str  # Author string (e.g., "Einstein, A. and Bohr, N.")
    year: str = ""
    journal: str = ""
    volume: str = ""
    pages: str = ""
    publisher: str = ""
    doi: str = ""
    url: str = ""
    note: str = ""
    
    def to_bibtex(self) -> str:
        """Convert to BibTeX format."""
        lines = [f"@{self.entry_type}{{{self.key},"]
        lines.append(f'  author = {{{self.authors}}},')
        lines.append(f'  title = {{{self.title}}},')
        if self.year:
            lines.append(f'  year = {{{self.year}}},')
        if self.journal:
            lines.append(f'  journal = {{{self.journal}}},')
        if self.volume:
            lines.append(f'  volume = {{{self.volume}}},')
        if self.pages:
            lines.append(f'  pages = {{{self.pages}}},')
        if self.publisher:
            lines.append(f'  publisher = {{{self.publisher}}},')
        if self.doi:
            lines.append(f'  doi = {{{self.doi}}},')
        if self.url:
            lines.append(f'  url = {{{self.url}}},')
        if self.note:
            lines.append(f'  note = {{{self.note}}},')
        lines.append("}")
        return "\n".join(lines)


@dataclass 
class CrossRef:
    """A cross-reference to another element in the document."""
    ref_type: str  # "eq", "fig", "sec", "tab", "thm", "bib"
    target: str    # Label of the target element
    text: Optional[str] = None  # Custom display text (optional)


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
        # Core document data
        self.metadata: Dict[str, Any] = {}
        self.template_path: Optional[str] = None
        self.template_info: Dict[str, Any] = {}
        
        # Standard content types (common across most formats)
        self.sections: List[Section] = []
        self.equations: Dict[str, Equation] = {}
        self.figures: Dict[str, Figure] = {}
        self.bibliography: Dict[str, BibEntry] = {}  # Keyed by citation key
        self.cross_refs: List[CrossRef] = []  # Track cross-references for validation
        
        # Extensible content blocks for format-specific needs
        # Examples:
        #   arXiv: {"pacs": [...], "msc_codes": [...], "supplementary": [...]}
        #   IEEE: {"keywords": [...], "index_terms": [...]}
        #   Nature: {"methods": "...", "data_availability": "...", "competing_interests": "..."}
        #   Legal: {"case_citations": [...], "statutes": [...]}
        self.content_blocks: Dict[str, Any] = {}
        
        # Tables (separate from figures for semantic clarity)
        self.tables: Dict[str, Any] = {}
        
        # Algorithms/pseudocode (common in CS papers)
        self.algorithms: Dict[str, Any] = {}
        
        # Theorems, lemmas, proofs (common in math papers)
        self.theorems: Dict[str, Any] = {}
        
        # Server configuration
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
                          (e.g., "stdlib/templates/article.kleis")
        
        Returns:
            A new KleisDoc with template loaded
        """
        doc = cls()
        doc.template_path = template_path
        doc._load_template(template_path)
        return doc
    
    @classmethod
    def load(cls, path: str) -> "KleisDoc":
        """Load a document from a .kleis file.
        
        Uses the Kleis binary to parse and evaluate the file, then extracts
        the document structure from the evaluated result.
        
        Args:
            path: Path to the .kleis file
        
        Returns:
            A KleisDoc instance populated with the saved data
        """
        doc = cls()
        
        # Read the file to extract data using pattern matching
        # (We parse the Kleis code structure directly since we generated it)
        with open(path, "r") as f:
            content = f.read()
        
        # Parse metadata (format-agnostic)
        doc.metadata = doc._extract_metadata(content)
        
        # Restore template_path if it was saved
        if "_template_path" in doc.metadata:
            doc.template_path = doc.metadata.pop("_template_path")
        
        # Parse equations
        doc.equations = doc._extract_equations(content)
        
        # Parse figures
        doc.figures = doc._extract_figures(content)
        
        # Parse content blocks
        doc.content_blocks = doc._extract_content_blocks(content)
        
        # Parse tables
        doc.tables = doc._extract_tables(content)
        
        # Parse theorems
        doc.theorems = doc._extract_theorems(content)
        
        # Parse algorithms
        doc.algorithms = doc._extract_algorithms(content)
        
        # Parse bibliography
        doc.bibliography = doc._extract_bibliography(content)
        
        # Parse cross-references
        doc.cross_refs = doc._extract_cross_refs(content)
        
        # Parse sections
        doc.sections = doc._extract_sections(content, doc.equations, doc.figures)
        
        return doc
    
    def _extract_metadata(self, content: str) -> Dict[str, Any]:
        """Extract metadata from Kleis file content.
        
        Looks for patterns like:
            define meta_title = "..."
            define meta_author = "..."
            define meta_author = Author(name = "...", email = "...", ...)
            define meta_committee_members = List("A", "B", "C")
        
        This is format-agnostic - extracts ALL meta_* definitions.
        """
        metadata = {}
        
        # Find all: define meta_xxx = Author(...)
        author_pattern = r'define\s+meta_(\w+)\s*=\s*Author\(\s*name\s*=\s*"([^"]*)"(?:,\s*email\s*=\s*"([^"]*)")?(?:,\s*affiliation\s*=\s*"([^"]*)")?(?:,\s*role\s*=\s*"([^"]*)")?\s*\)'
        for match in re.finditer(author_pattern, content):
            key = match.group(1)
            metadata[key] = Author(
                name=match.group(2) or "",
                email=match.group(3) or "",
                affiliation=match.group(4) or "",
                role=match.group(5) or "primary"
            )
        
        # Find all: define meta_xxx = List(Author(...), Author(...))
        # Match List of Authors - each Author ends with ), so match until final )
        list_author_pattern = r'define\s+meta_(\w+)\s*=\s*List\(((?:[^)]*Author[^)]*\)[,\s]*)+)\)'
        for match in re.finditer(list_author_pattern, content, re.DOTALL):
            key = match.group(1)
            list_content = match.group(2)
            authors = []
            # Find each Author(...) in the list
            for author_match in re.finditer(r'Author\(\s*name\s*=\s*"([^"]*)"(?:,\s*email\s*=\s*"([^"]*)")?(?:,\s*affiliation\s*=\s*"([^"]*)")?(?:,\s*role\s*=\s*"([^"]*)")?\s*\)', list_content):
                authors.append(Author(
                    name=author_match.group(1) or "",
                    email=author_match.group(2) or "",
                    affiliation=author_match.group(3) or "",
                    role=author_match.group(4) or "primary"
                ))
            if authors:
                metadata[key] = authors
        
        # Find all: define meta_xxx = "string value" (but not if already matched as Author)
        for match in re.finditer(r'define\s+meta_(\w+)\s*=\s*"([^"]*)"', content):
            key = match.group(1)
            if key not in metadata:  # Don't overwrite Author objects
                value = match.group(2)
                metadata[key] = value
        
        # Find all: define meta_xxx = List("a", "b", ...) (simple string lists, not Author lists)
        for match in re.finditer(r'define\s+meta_(\w+)\s*=\s*List\(([^)]*)\)', content):
            key = match.group(1)
            if key not in metadata:  # Don't overwrite Author lists
                list_content = match.group(2)
                if 'Author(' not in list_content:  # Skip if it contains Author
                    items = re.findall(r'"([^"]*)"', list_content)
                    if items:
                        metadata[key] = items
        
        # Find all: define meta_xxx = true/false
        for match in re.finditer(r'define\s+meta_(\w+)\s*=\s*(true|false)', content):
            key = match.group(1)
            if key not in metadata:
                value = match.group(2) == "true"
                metadata[key] = value
        
        # Find all: define meta_xxx = number
        for match in re.finditer(r'define\s+meta_(\w+)\s*=\s*(\d+(?:\.\d+)?)\s*$', content, re.MULTILINE):
            key = match.group(1)
            if key not in metadata:
                value_str = match.group(2)
                if '.' in value_str:
                    metadata[key] = float(value_str)
                else:
                    metadata[key] = int(value_str)
        
        # Also check for template import
        template_match = re.search(r'import\s+"([^"]+\.kleis)"', content)
        if template_match:
            # Store as special metadata so we can restore template_path
            metadata["_template_path"] = template_match.group(1)
        
        return metadata
    
    def _extract_equations(self, content: str) -> Dict[str, Equation]:
        """Extract equations from Kleis file content."""
        equations = {}
        
        # Find equation definitions: define eq_xxx = Equation(...)
        pattern = r'define\s+(\w+)\s*=\s*Equation\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            var_name = match.group(1)
            eq_body = match.group(2)
            
            # Extract fields
            id_match = re.search(r'id\s*=\s*"([^"]*)"', eq_body)
            label_match = re.search(r'label\s*=\s*"([^"]*)"', eq_body)
            latex_match = re.search(r'latex\s*=\s*"([^"]*)"', eq_body)
            typst_match = re.search(r'typst\s*=\s*"([^"]*)"', eq_body)
            numbered_match = re.search(r'numbered\s*=\s*(true|false)', eq_body)
            verified_match = re.search(r'verified\s*=\s*(true|false)', eq_body)
            
            # Extract AST (complex nested structure)
            ast_match = re.search(r'ast\s*=\s*(EOp\(.*?\)|EObject\(.*?\)|EConst\(.*?\)|None)', eq_body)
            ast = None
            if ast_match:
                ast_str = ast_match.group(1)
                # Find the full AST by counting parentheses
                ast = self._parse_kleis_ast(eq_body)
            
            if label_match:
                label = label_match.group(1)
                equations[label] = Equation(
                    id=id_match.group(1) if id_match else "",
                    label=label,
                    latex=latex_match.group(1) if latex_match else "",
                    typst=typst_match.group(1) if typst_match else "",
                    ast=ast,
                    numbered=numbered_match.group(1) == "true" if numbered_match else True,
                    verified=verified_match.group(1) == "true" if verified_match else False,
                )
        
        return equations
    
    def _parse_kleis_ast(self, eq_body: str) -> Optional[Dict]:
        """Parse EditorNode AST from Kleis code."""
        # Find ast = ...
        ast_start = eq_body.find("ast = ")
        if ast_start == -1:
            return None
        
        ast_str = eq_body[ast_start + 6:]
        
        # Simple recursive parser for our generated format
        return self._parse_ast_expr(ast_str)
    
    def _parse_ast_expr(self, s: str) -> Optional[Dict]:
        """Parse a single AST expression."""
        s = s.strip()
        
        if s.startswith("None"):
            return None
        
        if s.startswith("EObject("):
            # EObject("symbol")
            match = re.match(r'EObject\("([^"]*)"\)', s)
            if match:
                return {"Object": match.group(1)}
        
        if s.startswith("EConst("):
            # EConst("value")
            match = re.match(r'EConst\("([^"]*)"\)', s)
            if match:
                return {"Const": match.group(1)}
        
        if s.startswith("EOp("):
            # EOp("name", List(...), "", NoMeta)
            # Find the name
            name_match = re.match(r'EOp\("([^"]*)",\s*List\(', s)
            if name_match:
                name = name_match.group(1)
                # Find the args list
                args = self._extract_list_args(s)
                return {
                    "Operation": {
                        "name": name,
                        "args": args
                    }
                }
        
        return None
    
    def _extract_list_args(self, s: str) -> List[Dict]:
        """Extract arguments from a List(...) in an EOp."""
        # Find List( after the name
        list_start = s.find("List(")
        if list_start == -1:
            return []
        
        # Find matching parenthesis
        depth = 0
        start = list_start + 5
        args = []
        current_arg_start = start
        
        i = start
        while i < len(s):
            c = s[i]
            if c == '(':
                depth += 1
            elif c == ')':
                if depth == 0:
                    # End of List
                    arg_str = s[current_arg_start:i].strip()
                    if arg_str:
                        parsed = self._parse_ast_expr(arg_str)
                        if parsed:
                            args.append(parsed)
                    break
                depth -= 1
            elif c == ',' and depth == 0:
                # Argument separator
                arg_str = s[current_arg_start:i].strip()
                if arg_str:
                    parsed = self._parse_ast_expr(arg_str)
                    if parsed:
                        args.append(parsed)
                current_arg_start = i + 1
            i += 1
        
        return args
    
    def _extract_figures(self, content: str) -> Dict[str, Figure]:
        """Extract figures from Kleis file content."""
        figures = {}
        
        # Find figure definitions: define fig_xxx = Figure(...)
        pattern = r'define\s+(\w+)\s*=\s*Figure\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            var_name = match.group(1)
            fig_body = match.group(2)
            
            # Extract fields
            id_match = re.search(r'id\s*=\s*"([^"]*)"', fig_body)
            label_match = re.search(r'label\s*=\s*"([^"]*)"', fig_body)
            caption_match = re.search(r'caption\s*=\s*"([^"]*)"', fig_body)
            
            # Source type
            kleis_code = None
            image_path = None
            if "Regenerable(" in fig_body:
                code_match = re.search(r'Regenerable\("([^"]*)"', fig_body)
                if code_match:
                    kleis_code = code_match.group(1)
            elif "Imported(" in fig_body:
                path_match = re.search(r'Imported\("([^"]*)"', fig_body)
                if path_match:
                    image_path = path_match.group(1)
            
            if label_match:
                label = label_match.group(1)
                figures[label] = Figure(
                    id=id_match.group(1) if id_match else "",
                    label=label,
                    caption=caption_match.group(1) if caption_match else "",
                    kleis_code=kleis_code,
                    image_path=image_path,
                )
        
        return figures
    
    def _extract_content_blocks(self, content: str) -> Dict[str, Any]:
        """Extract content blocks from Kleis file content.
        
        Looks for patterns like:
            define content_xxx = "string value"
            define content_xxx = List("a", "b")
        """
        blocks = {}
        
        # Find string content blocks
        for match in re.finditer(r'define\s+content_(\w+)\s*=\s*"([^"]*)"', content):
            key = match.group(1)
            value = match.group(2)
            blocks[key] = value
        
        # Find list content blocks
        for match in re.finditer(r'define\s+content_(\w+)\s*=\s*List\((.*?)\)', content):
            key = match.group(1)
            list_content = match.group(2)
            items = re.findall(r'"([^"]*)"', list_content)
            blocks[key] = items
        
        return blocks
    
    def _extract_tables(self, content: str) -> Dict[str, Any]:
        """Extract tables from Kleis file content."""
        tables = {}
        
        # Find table definitions: define xxx = Table(...)
        pattern = r'define\s+(\w+)\s*=\s*Table\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            var_name = match.group(1)
            table_body = match.group(2)
            
            label_match = re.search(r'label\s*=\s*"([^"]*)"', table_body)
            caption_match = re.search(r'caption\s*=\s*"([^"]*)"', table_body)
            headers_match = re.search(r'headers\s*=\s*List\((.*?)\)', table_body)
            
            if label_match:
                label = label_match.group(1)
                headers = []
                if headers_match:
                    headers = re.findall(r'"([^"]*)"', headers_match.group(1))
                
                tables[label] = {
                    "label": label,
                    "headers": headers,
                    "rows": [],  # Rows not currently saved in full
                    "caption": caption_match.group(1) if caption_match else "",
                }
        
        return tables
    
    def _extract_theorems(self, content: str) -> Dict[str, Any]:
        """Extract theorems from Kleis file content."""
        theorems = {}
        
        # Find theorem definitions: define xxx = Theorem(...)
        pattern = r'define\s+(\w+)\s*=\s*Theorem\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            var_name = match.group(1)
            thm_body = match.group(2)
            
            label_match = re.search(r'label\s*=\s*"([^"]*)"', thm_body)
            kind_match = re.search(r'kind\s*=\s*"([^"]*)"', thm_body)
            statement_match = re.search(r'statement\s*=\s*"([^"]*)"', thm_body)
            proof_match = re.search(r'proof\s*=\s*"([^"]*)"', thm_body)
            name_match = re.search(r'name\s*=\s*"([^"]*)"', thm_body)
            
            if label_match:
                label = label_match.group(1)
                theorems[label] = {
                    "label": label,
                    "kind": kind_match.group(1) if kind_match else "theorem",
                    "statement": statement_match.group(1) if statement_match else "",
                    "proof": proof_match.group(1) if proof_match else None,
                    "name": name_match.group(1) if name_match else None,
                }
        
        return theorems
    
    def _extract_algorithms(self, content: str) -> Dict[str, Any]:
        """Extract algorithms from Kleis file content."""
        algorithms = {}
        
        # Find algorithm definitions: define xxx = Algorithm(...)
        pattern = r'define\s+(\w+)\s*=\s*Algorithm\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            var_name = match.group(1)
            alg_body = match.group(2)
            
            label_match = re.search(r'label\s*=\s*"([^"]*)"', alg_body)
            name_match = re.search(r'name\s*=\s*"([^"]*)"', alg_body)
            pseudocode_match = re.search(r'pseudocode\s*=\s*"([^"]*)"', alg_body)
            caption_match = re.search(r'caption\s*=\s*"([^"]*)"', alg_body)
            
            if label_match:
                label = label_match.group(1)
                algorithms[label] = {
                    "label": label,
                    "name": name_match.group(1) if name_match else "",
                    "pseudocode": pseudocode_match.group(1) if pseudocode_match else "",
                    "caption": caption_match.group(1) if caption_match else "",
                }
        
        return algorithms
    
    def _extract_bibliography(self, content: str) -> Dict[str, BibEntry]:
        """Extract bibliography entries from Kleis file content."""
        bibliography = {}
        
        # Find BibEntry definitions: define bib_xxx = BibEntry(...)
        pattern = r'define\s+bib_(\w+)\s*=\s*BibEntry\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            bib_body = match.group(2)
            
            key_match = re.search(r'key\s*=\s*"([^"]*)"', bib_body)
            type_match = re.search(r'entry_type\s*=\s*"([^"]*)"', bib_body)
            title_match = re.search(r'title\s*=\s*"([^"]*)"', bib_body)
            authors_match = re.search(r'authors\s*=\s*"([^"]*)"', bib_body)
            year_match = re.search(r'year\s*=\s*"([^"]*)"', bib_body)
            journal_match = re.search(r'journal\s*=\s*"([^"]*)"', bib_body)
            volume_match = re.search(r'volume\s*=\s*"([^"]*)"', bib_body)
            pages_match = re.search(r'pages\s*=\s*"([^"]*)"', bib_body)
            publisher_match = re.search(r'publisher\s*=\s*"([^"]*)"', bib_body)
            doi_match = re.search(r'doi\s*=\s*"([^"]*)"', bib_body)
            url_match = re.search(r'url\s*=\s*"([^"]*)"', bib_body)
            note_match = re.search(r'note\s*=\s*"([^"]*)"', bib_body)
            
            if key_match:
                key = key_match.group(1)
                bibliography[key] = BibEntry(
                    key=key,
                    entry_type=type_match.group(1) if type_match else "article",
                    title=title_match.group(1) if title_match else "",
                    authors=authors_match.group(1) if authors_match else "",
                    year=year_match.group(1) if year_match else "",
                    journal=journal_match.group(1) if journal_match else "",
                    volume=volume_match.group(1) if volume_match else "",
                    pages=pages_match.group(1) if pages_match else "",
                    publisher=publisher_match.group(1) if publisher_match else "",
                    doi=doi_match.group(1) if doi_match else "",
                    url=url_match.group(1) if url_match else "",
                    note=note_match.group(1) if note_match else ""
                )
        
        return bibliography
    
    def _extract_cross_refs(self, content: str) -> List[CrossRef]:
        """Extract cross-references from Kleis file content."""
        cross_refs = []
        
        # Find CrossRef definitions: define crossref_N = CrossRef(...)
        pattern = r'define\s+crossref_\d+\s*=\s*CrossRef\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            ref_body = match.group(1)
            
            type_match = re.search(r'ref_type\s*=\s*"([^"]*)"', ref_body)
            target_match = re.search(r'target\s*=\s*"([^"]*)"', ref_body)
            text_match = re.search(r'text\s*=\s*"([^"]*)"', ref_body)
            
            if type_match and target_match:
                cross_refs.append(CrossRef(
                    ref_type=type_match.group(1),
                    target=target_match.group(1),
                    text=text_match.group(1) if text_match else None
                ))
        
        return cross_refs
    
    def _extract_sections(self, content: str, equations: Dict[str, Equation], 
                          figures: Dict[str, Figure]) -> List[Section]:
        """Extract sections from Kleis file content and reconstruct hierarchy.
        
        Sections are saved with their level, and on load we reconstruct
        the parent-child relationships based on those levels.
        """
        flat_sections = []
        
        # Find section definitions: define section_N = Section(...) or define section_N_subM = Section(...)
        pattern = r'define\s+(section_[\w]+)\s*=\s*Section\((.*?)\n\)'
        for match in re.finditer(pattern, content, re.DOTALL):
            var_name = match.group(1)
            sec_body = match.group(2)
            
            # Extract fields
            level_match = re.search(r'level\s*=\s*(\d+)', sec_body)
            title_match = re.search(r'title\s*=\s*"([^"]*)"', sec_body)
            
            level = int(level_match.group(1)) if level_match else 1
            title = title_match.group(1) if title_match else ""
            
            section = Section(level=level, title=title, content=[])
            
            # Extract content items
            content_match = re.search(r'content\s*=\s*List\((.*)\)', sec_body, re.DOTALL)
            if content_match:
                content_str = content_match.group(1)
                
                # Parse Text("...") items
                for text_match in re.finditer(r'Text\("([^"]*)"\)', content_str):
                    section.content.append(text_match.group(1))
                
                # Parse EqRef("...") items
                for eq_match in re.finditer(r'EqRef\("([^"]*)"\)', content_str):
                    eq_label = eq_match.group(1)
                    if eq_label in equations:
                        section.content.append(equations[eq_label])
                
                # Parse FigRef("...") items
                for fig_match in re.finditer(r'FigRef\("([^"]*)"\)', content_str):
                    fig_label = fig_match.group(1)
                    if fig_label in figures:
                        section.content.append(figures[fig_label])
            
            flat_sections.append(section)
        
        # Reconstruct hierarchy: subsections become children of parent sections
        return self._build_section_hierarchy(flat_sections)
    
    def _build_section_hierarchy(self, flat_sections: List[Section]) -> List[Section]:
        """Build nested section hierarchy from flat list based on levels.
        
        Level 1 sections are top-level. Level 2 sections become children
        of the preceding level 1 section, etc.
        """
        if not flat_sections:
            return []
        
        root_sections = []
        stack = []  # Stack of (level, section) to track current path
        
        for section in flat_sections:
            # Pop sections from stack that are at same or deeper level
            while stack and stack[-1][0] >= section.level:
                stack.pop()
            
            if not stack:
                # This is a top-level section
                root_sections.append(section)
            else:
                # This is a subsection - add to parent's content
                parent = stack[-1][1]
                parent.content.append(section)
            
            # Push current section onto stack
            stack.append((section.level, section))
        
        return root_sections
    
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
        
        Extracts Typst styling definitions from the template so they can
        be used during export. This is what makes new styles work without
        code changes - all styling comes from the .kleis template file.
        """
        self.template_info = {
            "path": template_path,
            "loaded": False,
            "typst": {},  # Will hold all typst_* definitions
        }
        
        # Try to read and parse the template file
        try:
            path = Path(template_path)
            if not path.exists():
                # Try relative to project root
                for base in [Path.cwd(), Path(__file__).parent.parent.parent]:
                    candidate = base / template_path
                    if candidate.exists():
                        path = candidate
                        break
            
            if path.exists():
                content = path.read_text()
                self.template_info["loaded"] = True
                self.template_info["size"] = len(content)
                
                # Extract typst_* definitions using regex
                # Pattern matches: define typst_xxx = """...""" or define typst_xxx = "..."
                
                # Multi-line strings with triple quotes
                for match in re.finditer(
                    r'define\s+(typst_\w+)\s*=\s*"""(.*?)"""',
                    content, re.DOTALL
                ):
                    name = match.group(1)
                    value = match.group(2)
                    self.template_info["typst"][name] = value
                
                # Single-line strings (for simple definitions)
                for match in re.finditer(
                    r'define\s+(typst_\w+)\s*=\s*"([^"]*)"',
                    content
                ):
                    name = match.group(1)
                    value = match.group(2)
                    # Don't overwrite if already found as multi-line
                    if name not in self.template_info["typst"]:
                        self.template_info["typst"][name] = value
                
                # Also extract template_name for display
                name_match = re.search(r'define\s+template_name\s*=\s*"([^"]*)"', content)
                if name_match:
                    self.template_info["name"] = name_match.group(1)
                    
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
        
        Template-specific fields vary by document type:
        - journal, volume, issue (journal article)
        - publisher, isbn (book)
        - conference, location (proceedings)
        - institution, department (report)
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
    # Extensible Content Blocks
    # =========================================================================
    
    def set_content_block(self, name: str, content: Any):
        """Set a format-specific content block.
        
        Use this for content types not covered by the standard types
        (sections, equations, figures). Templates define what blocks
        they expect.
        
        Args:
            name: Block name (e.g., "acknowledgments", "data_availability")
            content: Block content (string, list, dict, etc.)
        
        Examples:
            # arXiv paper
            doc.set_content_block("pacs_numbers", ["03.65.-w", "02.10.Yn"])
            doc.set_content_block("msc_codes", ["81P05", "03G12"])
            
            # Nature paper
            doc.set_content_block("data_availability", "Data available at...")
            doc.set_content_block("competing_interests", "None declared.")
            
            # IEEE paper
            doc.set_content_block("keywords", ["machine learning", "neural networks"])
        """
        self.content_blocks[name] = content
    
    def get_content_block(self, name: str) -> Optional[Any]:
        """Get a content block by name."""
        return self.content_blocks.get(name)
    
    def add_table(self, label: str, headers: List[str], rows: List[List[Any]],
                  caption: str = "", kleis_code: str = None) -> Dict:
        """Add a table to the document.
        
        Args:
            label: Unique label (e.g., "tab:results")
            headers: Column headers
            rows: Table data rows
            caption: Table caption
            kleis_code: Optional Kleis code for computed tables
        
        Returns:
            The table dict
        """
        table = {
            "label": label,
            "headers": headers,
            "rows": rows,
            "caption": caption,
            "kleis_code": kleis_code,
        }
        self.tables[label] = table
        return table
    
    def add_theorem(self, label: str, kind: str, statement: str,
                    proof: str = None, name: str = None) -> Dict:
        """Add a theorem, lemma, proposition, or similar to the document.
        
        Args:
            label: Unique label (e.g., "thm:main")
            kind: Type ("theorem", "lemma", "proposition", "corollary", "definition")
            statement: The statement text
            proof: Optional proof text
            name: Optional theorem name (e.g., "Fermat's Last Theorem")
        
        Returns:
            The theorem dict
        """
        thm = {
            "label": label,
            "kind": kind,
            "statement": statement,
            "proof": proof,
            "name": name,
        }
        self.theorems[label] = thm
        return thm
    
    def add_algorithm(self, label: str, name: str, pseudocode: str,
                      caption: str = "") -> Dict:
        """Add an algorithm/pseudocode block to the document.
        
        Args:
            label: Unique label (e.g., "alg:sort")
            name: Algorithm name
            pseudocode: The pseudocode text
            caption: Algorithm caption
        
        Returns:
            The algorithm dict
        """
        alg = {
            "label": label,
            "name": name,
            "pseudocode": pseudocode,
            "caption": caption,
        }
        self.algorithms[label] = alg
        return alg
    
    # =========================================================================
    # Bibliography
    # =========================================================================
    
    def add_bib_entry(self, key: str, entry_type: str, title: str, authors: str,
                      year: str = "", journal: str = "", volume: str = "",
                      pages: str = "", publisher: str = "", doi: str = "",
                      url: str = "", note: str = "") -> BibEntry:
        """Add a bibliography entry.
        
        Args:
            key: Citation key (e.g., "einstein1905")
            entry_type: Type (article, book, inproceedings, thesis, etc.)
            title: Work title
            authors: Author string (e.g., "Einstein, A.")
            year: Publication year
            journal: Journal name (for articles)
            volume: Volume number
            pages: Page range
            publisher: Publisher (for books)
            doi: Digital Object Identifier
            url: URL
            note: Additional notes
        
        Returns:
            The created BibEntry
        """
        entry = BibEntry(
            key=key,
            entry_type=entry_type,
            title=title,
            authors=authors,
            year=year,
            journal=journal,
            volume=volume,
            pages=pages,
            publisher=publisher,
            doi=doi,
            url=url,
            note=note
        )
        self.bibliography[key] = entry
        return entry
    
    def cite(self, key: str, text: str = None) -> CrossRef:
        """Create a citation cross-reference.
        
        Args:
            key: Citation key (must exist in bibliography)
            text: Optional custom citation text
        
        Returns:
            CrossRef object for the citation
        """
        ref = CrossRef(ref_type="bib", target=key, text=text)
        self.cross_refs.append(ref)
        return ref
    
    def ref_equation(self, label: str, text: str = None) -> CrossRef:
        """Create a cross-reference to an equation."""
        ref = CrossRef(ref_type="eq", target=label, text=text)
        self.cross_refs.append(ref)
        return ref
    
    def ref_figure(self, label: str, text: str = None) -> CrossRef:
        """Create a cross-reference to a figure."""
        ref = CrossRef(ref_type="fig", target=label, text=text)
        self.cross_refs.append(ref)
        return ref
    
    def ref_section(self, label: str, text: str = None) -> CrossRef:
        """Create a cross-reference to a section."""
        ref = CrossRef(ref_type="sec", target=label, text=text)
        self.cross_refs.append(ref)
        return ref
    
    def ref_table(self, label: str, text: str = None) -> CrossRef:
        """Create a cross-reference to a table."""
        ref = CrossRef(ref_type="tab", target=label, text=text)
        self.cross_refs.append(ref)
        return ref
    
    def ref_theorem(self, label: str, text: str = None) -> CrossRef:
        """Create a cross-reference to a theorem."""
        ref = CrossRef(ref_type="thm", target=label, text=text)
        self.cross_refs.append(ref)
        return ref
    
    def export_bibtex(self, path: str = None) -> str:
        """Export bibliography to BibTeX format.
        
        Args:
            path: Optional file path to save to
        
        Returns:
            BibTeX string
        """
        bibtex = "\n\n".join(entry.to_bibtex() for entry in self.bibliography.values())
        if path:
            with open(path, "w") as f:
                f.write(bibtex)
        return bibtex
    
    def validate_cross_refs(self) -> List[str]:
        """Validate all cross-references have valid targets.
        
        Returns:
            List of error messages (empty if all valid)
        """
        errors = []
        for ref in self.cross_refs:
            if ref.ref_type == "bib" and ref.target not in self.bibliography:
                errors.append(f"Missing bibliography entry: {ref.target}")
            elif ref.ref_type == "eq" and ref.target not in self.equations:
                errors.append(f"Missing equation: {ref.target}")
            elif ref.ref_type == "fig" and ref.target not in self.figures:
                errors.append(f"Missing figure: {ref.target}")
            elif ref.ref_type == "tab" and ref.target not in self.tables:
                errors.append(f"Missing table: {ref.target}")
            elif ref.ref_type == "thm" and ref.target not in self.theorems:
                errors.append(f"Missing theorem: {ref.target}")
        return errors
    
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
    
    def _substitute_placeholders(self, template_str: str) -> str:
        """Substitute ALL placeholders in a template string generically.
        
        Placeholders are UPPERCASE words like TITLE, AUTHOR, JOURNAL_NAME, etc.
        This method makes the system truly extensible - any new template can use
        any placeholder names without Python code changes.
        
        Lookup order:
        1. self.metadata (for document properties)
        2. self.content_blocks (for custom content)
        
        Lists are joined with line breaks for Typst.
        """
        result = template_str
        
        # Find all UPPERCASE placeholders (words of 2+ uppercase letters/underscores)
        placeholder_pattern = re.compile(r'\b([A-Z][A-Z0-9_]+)\b')
        
        for match in placeholder_pattern.finditer(template_str):
            placeholder = match.group(1)
            key = placeholder.lower().replace("_", "-")  # DEGREE_NAME -> degree-name
            key_underscore = placeholder.lower()  # DEGREE_NAME -> degree_name
            
            # Try multiple key formats
            value = (self.metadata.get(key) or 
                     self.metadata.get(key_underscore) or
                     self.metadata.get(placeholder) or
                     self.content_blocks.get(key) or
                     self.content_blocks.get(key_underscore) or
                     self.content_blocks.get(placeholder))
            
            if value is not None:
                if isinstance(value, list):
                    # Join lists with Typst line breaks
                    result = result.replace(placeholder, " #linebreak()\n      ".join(str(v) for v in value))
                else:
                    result = result.replace(placeholder, str(value))
        
        return result
    
    def _generate_typst(self, template: str = None) -> str:
        """Generate Typst code for the document.
        
        If a template is loaded, uses the typst_* definitions from the template.
        This is what makes new document styles work WITHOUT code changes -
        all styling comes from the .kleis template file.
        """
        lines = []
        
        # Get Typst definitions from template (if loaded)
        typst_defs = self.template_info.get("typst", {}) if self.template_info else {}
        
        # Use template preamble if available, otherwise minimal defaults
        if "typst_preamble" in typst_defs:
            lines.append("// Styling from template: " + self.template_info.get("name", self.template_path or "unknown"))
            lines.append(typst_defs["typst_preamble"])
            lines.append('')
        elif "typst_page_setup" in typst_defs:
            # Individual setup definitions
            lines.append("// Styling from template")
            if "typst_page_setup" in typst_defs:
                lines.append(typst_defs["typst_page_setup"])
            if "typst_text_setup" in typst_defs:
                lines.append(typst_defs["typst_text_setup"])
            if "typst_paragraph_setup" in typst_defs:
                lines.append(typst_defs["typst_paragraph_setup"])
            if "typst_heading_setup" in typst_defs:
                lines.append(typst_defs["typst_heading_setup"])
            lines.append('')
        else:
            # Minimal defaults when no template
            lines.append('// No template - using minimal defaults')
            lines.append('#set page(margin: 1in)')
            lines.append('#set text(size: 11pt)')
            lines.append('')
        
        # Document metadata (standard Typst)
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
        
        # Title page from template or default
        if "typst_title_page" in typst_defs:
            # Substitute ALL placeholders generically
            title_page = self._substitute_placeholders(typst_defs["typst_title_page"])
            lines.append(title_page)
            lines.append('')
        else:
            # Default title rendering
            if "title" in self.metadata:
                lines.append(f'#align(center)[')
                lines.append(f'  #text(size: 20pt, weight: "bold")[{self.metadata["title"]}]')
                lines.append(f']')
                lines.append('')
            
            if "author" in self.metadata:
                author = self.metadata["author"]
                if isinstance(author, str):
                    lines.append(f'#align(center)[{author}]')
                elif isinstance(author, Author):
                    lines.append(f'#align(center)[{author.name}]')
                lines.append('')
        
        # Abstract from template or default
        if "abstract" in self.metadata:
            if "typst_abstract_box" in typst_defs:
                # Use generic substitution for abstract box
                lines.append(self._substitute_placeholders(typst_defs["typst_abstract_box"]))
            else:
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
        """Save document to a .kleis file.
        
        Generates valid Kleis code that can be parsed and evaluated.
        The saved file contains:
        - Import of kleisdoc_types.kleis
        - Document metadata as Kleis data structures
        - All equations with their EditorNode ASTs (for re-editing)
        - All figures with their Kleis code (for regeneration)
        - Document structure (sections, content order)
        
        Args:
            path: Path for the output file (should be .kleis extension)
        """
        lines = []
        
        # Header
        lines.append("// ============================================================================")
        lines.append("// KleisDoc - Auto-generated document file")
        lines.append("// ============================================================================")
        lines.append("//")
        lines.append("// This file was generated by KleisDoc Python API.")
        lines.append("// It can be loaded back into KleisDoc for continued editing.")
        lines.append("//")
        lines.append("// ============================================================================")
        lines.append("")
        lines.append('import "examples/documents/kleisdoc_types.kleis"')
        lines.append("")
        
        # Template (if specified)
        if self.template_path:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Template")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            lines.append(f'import {self._to_kleis_string(self.template_path)}')
            lines.append("")
        
        # Metadata (save ALL fields, not just hardcoded ones)
        lines.append("// ----------------------------------------------------------------------------")
        lines.append("// Document Metadata")
        lines.append("// ----------------------------------------------------------------------------")
        lines.append("")
        
        # Save each metadata field as a separate definition (format-agnostic)
        for key, value in self.metadata.items():
            safe_key = key.replace("-", "_").replace(":", "_").replace(" ", "_")
            if isinstance(value, str):
                lines.append(f'define meta_{safe_key} = {self._to_kleis_string(value)}')
            elif isinstance(value, Author):
                # Save Author as structured data
                lines.append(f'define meta_{safe_key} = Author(')
                lines.append(f'    name = {self._to_kleis_string(value.name)},')
                lines.append(f'    email = {self._to_kleis_string(value.email)},')
                lines.append(f'    affiliation = {self._to_kleis_string(value.affiliation)},')
                lines.append(f'    role = {self._to_kleis_string(value.role)}')
                lines.append(f')')
            elif isinstance(value, list):
                # Check if it's a list of Authors
                if value and isinstance(value[0], Author):
                    lines.append(f'define meta_{safe_key} = List(')
                    for i, author in enumerate(value):
                        comma = "," if i < len(value) - 1 else ""
                        lines.append(f'    Author(name = {self._to_kleis_string(author.name)}, email = {self._to_kleis_string(author.email)}, affiliation = {self._to_kleis_string(author.affiliation)}, role = {self._to_kleis_string(author.role)}){comma}')
                    lines.append(f')')
                else:
                    items = ", ".join(self._to_kleis_string(str(item)) for item in value)
                    lines.append(f'define meta_{safe_key} = List({items})')
            elif isinstance(value, bool):
                lines.append(f'define meta_{safe_key} = {str(value).lower()}')
            elif isinstance(value, (int, float)):
                lines.append(f'define meta_{safe_key} = {value}')
            else:
                # For complex objects, convert to string
                lines.append(f'define meta_{safe_key} = {self._to_kleis_string(str(value))}')
        lines.append("")
        
        # Equations
        if self.equations:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Equations (with EditorNode AST for re-editing)")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for label, eq in self.equations.items():
                safe_name = label.replace(":", "_").replace("-", "_")
                lines.append(f"define {safe_name} = Equation(")
                lines.append(f'    id = {self._to_kleis_string(eq.id)},')
                lines.append(f'    label = {self._to_kleis_string(eq.label)},')
                lines.append(f'    latex = {self._to_kleis_string(eq.latex)},')
                lines.append(f'    typst = {self._to_kleis_string(eq.typst)},')
                lines.append(f'    ast = {self._ast_to_kleis(eq.ast)},')
                lines.append(f'    numbered = {str(eq.numbered).lower()},')
                lines.append(f'    verified = {str(eq.verified).lower()}')
                lines.append(")")
                lines.append("")
        
        # Figures
        if self.figures:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Figures (with Kleis code for regeneration)")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for label, fig in self.figures.items():
                safe_name = label.replace(":", "_").replace("-", "_")
                lines.append(f"define {safe_name} = Figure(")
                lines.append(f'    id = {self._to_kleis_string(fig.id)},')
                lines.append(f'    label = {self._to_kleis_string(fig.label)},')
                lines.append(f'    caption = {self._to_kleis_string(fig.caption)},')
                if fig.kleis_code:
                    lines.append(f'    source = Regenerable({self._to_kleis_string(fig.kleis_code)}, List()),')
                elif fig.image_path:
                    lines.append(f'    source = Imported({self._to_kleis_string(fig.image_path)}, ""),')
                else:
                    lines.append('    source = Static,')
                lines.append(f'    typst_fragment = {self._to_kleis_string(fig.typst_fragment or "")},')
                lines.append(f'    svg_cache = {self._to_kleis_string(fig.svg_cache or "")}')
                lines.append(")")
                lines.append("")
        
        # Content blocks (extensible format-specific content)
        if self.content_blocks:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Content Blocks (format-specific)")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for name, content in self.content_blocks.items():
                safe_name = name.replace("-", "_").replace(":", "_")
                if isinstance(content, str):
                    lines.append(f'define content_{safe_name} = {self._to_kleis_string(content)}')
                elif isinstance(content, list):
                    items = ", ".join(self._to_kleis_string(str(item)) for item in content)
                    lines.append(f'define content_{safe_name} = List({items})')
                else:
                    lines.append(f'define content_{safe_name} = {self._to_kleis_string(str(content))}')
                lines.append("")
        
        # Tables
        if self.tables:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Tables")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for label, table in self.tables.items():
                safe_name = label.replace(":", "_").replace("-", "_")
                lines.append(f"define {safe_name} = Table(")
                lines.append(f'    label = {self._to_kleis_string(table["label"])},')
                headers = ", ".join(self._to_kleis_string(h) for h in table.get("headers", []))
                lines.append(f'    headers = List({headers}),')
                lines.append(f'    caption = {self._to_kleis_string(table.get("caption", ""))}')
                lines.append(")")
                lines.append("")
        
        # Theorems
        if self.theorems:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Theorems and Proofs")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for label, thm in self.theorems.items():
                safe_name = label.replace(":", "_").replace("-", "_")
                lines.append(f"define {safe_name} = Theorem(")
                lines.append(f'    label = {self._to_kleis_string(thm["label"])},')
                lines.append(f'    kind = {self._to_kleis_string(thm.get("kind", "theorem"))},')
                lines.append(f'    statement = {self._to_kleis_string(thm.get("statement", ""))},')
                lines.append(f'    proof = {self._to_kleis_string(thm.get("proof") or "")},')
                lines.append(f'    name = {self._to_kleis_string(thm.get("name") or "")}')
                lines.append(")")
                lines.append("")
        
        # Algorithms
        if self.algorithms:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Algorithms")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for label, alg in self.algorithms.items():
                safe_name = label.replace(":", "_").replace("-", "_")
                lines.append(f"define {safe_name} = Algorithm(")
                lines.append(f'    label = {self._to_kleis_string(alg["label"])},')
                lines.append(f'    name = {self._to_kleis_string(alg.get("name", ""))},')
                lines.append(f'    pseudocode = {self._to_kleis_string(alg.get("pseudocode", ""))},')
                lines.append(f'    caption = {self._to_kleis_string(alg.get("caption", ""))}')
                lines.append(")")
                lines.append("")
        
        # Bibliography
        if self.bibliography:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Bibliography")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for key, entry in self.bibliography.items():
                safe_name = f"bib_{key}".replace(":", "_").replace("-", "_")
                lines.append(f"define {safe_name} = BibEntry(")
                lines.append(f'    key = {self._to_kleis_string(entry.key)},')
                lines.append(f'    entry_type = {self._to_kleis_string(entry.entry_type)},')
                lines.append(f'    title = {self._to_kleis_string(entry.title)},')
                lines.append(f'    authors = {self._to_kleis_string(entry.authors)},')
                lines.append(f'    year = {self._to_kleis_string(entry.year)},')
                lines.append(f'    journal = {self._to_kleis_string(entry.journal)},')
                lines.append(f'    volume = {self._to_kleis_string(entry.volume)},')
                lines.append(f'    pages = {self._to_kleis_string(entry.pages)},')
                lines.append(f'    publisher = {self._to_kleis_string(entry.publisher)},')
                lines.append(f'    doi = {self._to_kleis_string(entry.doi)},')
                lines.append(f'    url = {self._to_kleis_string(entry.url)},')
                lines.append(f'    note = {self._to_kleis_string(entry.note)}')
                lines.append(")")
                lines.append("")
        
        # Cross-references
        if self.cross_refs:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Cross-References")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for i, ref in enumerate(self.cross_refs):
                lines.append(f"define crossref_{i} = CrossRef(")
                lines.append(f'    ref_type = {self._to_kleis_string(ref.ref_type)},')
                lines.append(f'    target = {self._to_kleis_string(ref.target)},')
                lines.append(f'    text = {self._to_kleis_string(ref.text) if ref.text else "None"}')
                lines.append(")")
                lines.append("")
        
        # Sections
        if self.sections:
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("// Document Structure")
            lines.append("// ----------------------------------------------------------------------------")
            lines.append("")
            
            for i, section in enumerate(self.sections):
                lines.append(self._section_to_kleis(section, f"section_{i}"))
                lines.append("")
        
        # Document assembly
        lines.append("// ----------------------------------------------------------------------------")
        lines.append("// Document Assembly")
        lines.append("// ----------------------------------------------------------------------------")
        lines.append("")
        lines.append("define document = KleisDoc(")
        lines.append("    metadata = doc_metadata,")
        
        # Equations list
        if self.equations:
            eq_refs = ", ".join(label.replace(":", "_").replace("-", "_") for label in self.equations.keys())
            lines.append(f"    equations = List({eq_refs}),")
        else:
            lines.append("    equations = List(),")
        
        # Figures list
        if self.figures:
            fig_refs = ", ".join(label.replace(":", "_").replace("-", "_") for label in self.figures.keys())
            lines.append(f"    figures = List({fig_refs}),")
        else:
            lines.append("    figures = List(),")
        
        # Sections list
        if self.sections:
            sec_refs = ", ".join(f"section_{i}" for i in range(len(self.sections)))
            lines.append(f"    sections = List({sec_refs})")
        else:
            lines.append("    sections = List()")
        
        lines.append(")")
        lines.append("")
        
        with open(path, "w") as f:
            f.write("\n".join(lines))
    
    def _to_kleis_string(self, s: str) -> str:
        """Convert a Python string to a Kleis string literal."""
        if s is None:
            return '""'
        # Escape quotes and backslashes
        escaped = s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
        return f'"{escaped}"'
    
    def _ast_to_kleis(self, ast: Optional[Dict]) -> str:
        """Convert an EditorNode AST dict to Kleis code."""
        if ast is None:
            return "None"
        
        if "Object" in ast:
            return f'EObject({self._to_kleis_string(ast["Object"])})'
        
        if "Const" in ast:
            return f'EConst({self._to_kleis_string(ast["Const"])})'
        
        if "Operation" in ast:
            op = ast["Operation"]
            name = op.get("name", "")
            args = op.get("args", [])
            args_kleis = ", ".join(self._ast_to_kleis(arg) for arg in args)
            return f'EOp({self._to_kleis_string(name)}, List({args_kleis}), "", NoMeta)'
        
        if "Placeholder" in ast:
            ph = ast["Placeholder"]
            return f'EPlaceholder(Placeholder({ph.get("id", 0)}, {self._to_kleis_string(ph.get("hint", ""))}))'
        
        if "List" in ast:
            items = ast["List"]
            items_kleis = ", ".join(self._ast_to_kleis(item) for item in items)
            return f'EList(List({items_kleis}))'
        
        # Fallback for unknown structures
        return "None"
    
    def _section_to_kleis(self, section: Section, var_name: str) -> str:
        """Convert a section to Kleis code, including nested subsections.
        
        Parent sections are output BEFORE their subsections so that
        hierarchy reconstruction works correctly on load.
        """
        lines = []
        lines.append(f"define {var_name} = Section(")
        lines.append(f"    level = {section.level},")
        lines.append(f"    title = {self._to_kleis_string(section.title)},")
        
        # Build content list
        content_items = []
        subsection_defs = []  # Store subsection definitions to emit AFTER this section
        subsection_counter = [0]
        
        for item in section.content:
            if isinstance(item, str):
                content_items.append(f'Text({self._to_kleis_string(item)})')
            elif isinstance(item, Equation):
                content_items.append(f'EqRef({self._to_kleis_string(item.label)})')
            elif isinstance(item, Figure):
                content_items.append(f'FigRef({self._to_kleis_string(item.label)})')
            elif isinstance(item, Section):
                # Nested sections - save with full content
                sub_var = f'{var_name}_sub{subsection_counter[0]}'
                subsection_counter[0] += 1
                # Recursively get the subsection definition
                sub_def = self._section_to_kleis(item, sub_var)
                subsection_defs.append(sub_def)
                content_items.append(f'SectionRef({self._to_kleis_string(sub_var)})')
        
        if content_items:
            lines.append(f"    content = List({', '.join(content_items)})")
        else:
            lines.append("    content = List()")
        
        lines.append(")")
        
        # Parent section FIRST, then subsection definitions
        result = "\n".join(lines)
        if subsection_defs:
            result = result + '\n\n' + '\n\n'.join(subsection_defs)
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
