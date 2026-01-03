"""
Kleis Binary Discovery

This module provides a unified way to find the Kleis binary and project root
across all Python code in the kleis-notebook package.

Environment Variables:
    KLEIS_ROOT: Path to the Kleis project root (highest priority)
                Expected structure: $KLEIS_ROOT/target/release/kleis
                                   $KLEIS_ROOT/stdlib/prelude.kleis

Search Order for Binary:
    1. $KLEIS_ROOT/target/release/kleis (if KLEIS_ROOT set)
    2. $KLEIS_ROOT/target/debug/kleis (if KLEIS_ROOT set)
    3. System PATH (via shutil.which)
    4. ~/.cargo/bin/kleis (cargo install location)
    5. /usr/local/bin/kleis
    6. /usr/bin/kleis

Search Order for Project Root:
    1. KLEIS_ROOT environment variable
    2. Parent of kleis binary if binary is in target/release/ or target/debug/
    3. Common development locations
    4. Current working directory if it has stdlib/

Example:
    >>> from kleis_kernel.kleis_binary import find_kleis_binary, find_kleis_root
    >>> binary = find_kleis_binary()
    >>> if binary:
    ...     print(f"Found Kleis at: {binary}")
    >>> root = find_kleis_root()
    >>> if root:
    ...     print(f"Kleis project root: {root}")
"""

import os
import shutil
import subprocess
from pathlib import Path
from typing import Optional, Tuple

# Cache for discovered paths (avoid repeated subprocess calls)
_cached_binary: Optional[str] = None
_cached_root: Optional[str] = None
_cache_initialized: bool = False


def find_kleis_binary(use_cache: bool = True) -> Optional[str]:
    """Find the Kleis binary.
    
    Args:
        use_cache: If True, return cached result if available.
                   Set to False to force re-discovery.
    
    Returns:
        Path to the kleis binary, or None if not found.
    
    Search order:
        1. $KLEIS_ROOT/target/release/kleis
        2. $KLEIS_ROOT/target/debug/kleis  
        3. System PATH (shutil.which)
        4. ~/.cargo/bin/kleis
        5. /usr/local/bin/kleis
        6. /usr/bin/kleis
    """
    global _cached_binary, _cache_initialized
    
    if use_cache and _cache_initialized and _cached_binary is not None:
        return _cached_binary
    
    candidates = []
    
    # 1. Check KLEIS_ROOT environment variable first (highest priority)
    kleis_root = os.environ.get("KLEIS_ROOT")
    if kleis_root:
        candidates.extend([
            os.path.join(kleis_root, "target", "release", "kleis"),
            os.path.join(kleis_root, "target", "debug", "kleis"),
        ])
    
    # 2. Check system PATH using shutil.which (proper PATH lookup)
    path_binary = shutil.which("kleis")
    if path_binary:
        candidates.append(path_binary)
    
    # 3. Check common installation locations
    home = os.path.expanduser("~")
    candidates.extend([
        os.path.join(home, ".cargo", "bin", "kleis"),
        "/usr/local/bin/kleis",
        "/usr/bin/kleis",
    ])
    
    # Test each candidate
    for candidate in candidates:
        if _is_valid_kleis_binary(candidate):
            _cached_binary = candidate
            _cache_initialized = True
            return candidate
    
    _cache_initialized = True
    return None


def find_kleis_root(use_cache: bool = True) -> Optional[str]:
    """Find the Kleis project root directory.
    
    The project root is the directory containing stdlib/ and typically
    the Cargo.toml for the Kleis project.
    
    Args:
        use_cache: If True, return cached result if available.
    
    Returns:
        Path to the Kleis project root, or None if not found.
    
    Search order:
        1. KLEIS_ROOT environment variable
        2. Parent of kleis binary (if in target/release/ or target/debug/)
        3. Common development locations
        4. Current working directory if it has stdlib/
    """
    global _cached_root, _cache_initialized
    
    if use_cache and _cache_initialized and _cached_root is not None:
        return _cached_root
    
    # 1. Check environment variable first
    env_root = os.environ.get("KLEIS_ROOT")
    if env_root and _is_valid_kleis_root(env_root):
        _cached_root = env_root
        return env_root
    
    # 2. Infer from binary location
    binary = find_kleis_binary(use_cache=use_cache)
    if binary:
        binary_path = Path(binary).resolve()
        # If binary is at .../target/release/kleis or .../target/debug/kleis
        # then project root is 3 levels up
        if binary_path.parent.name in ("release", "debug"):
            if binary_path.parent.parent.name == "target":
                potential_root = str(binary_path.parent.parent.parent)
                if _is_valid_kleis_root(potential_root):
                    _cached_root = potential_root
                    return potential_root
    
    # 3. Check common development locations
    home = os.path.expanduser("~")
    candidates = [
        os.path.join(home, "git", "kleis"),
        os.path.join(home, "projects", "kleis"),
        os.path.join(home, "src", "kleis"),
        os.path.join(home, "code", "kleis"),
        os.path.join(home, "kleis"),
    ]
    
    for candidate in candidates:
        if _is_valid_kleis_root(candidate):
            _cached_root = candidate
            return candidate
    
    # 4. Check current working directory
    cwd = os.getcwd()
    if _is_valid_kleis_root(cwd):
        _cached_root = cwd
        return cwd
    
    # 5. Walk up from current directory
    current = Path(cwd)
    for _ in range(5):  # Check up to 5 levels up
        if _is_valid_kleis_root(str(current)):
            _cached_root = str(current)
            return str(current)
        if current.parent == current:
            break
        current = current.parent
    
    return None


def find_kleis_binary_and_root(use_cache: bool = True) -> Tuple[Optional[str], Optional[str]]:
    """Find both the Kleis binary and project root.
    
    Returns:
        Tuple of (binary_path, root_path). Either may be None if not found.
    """
    return find_kleis_binary(use_cache), find_kleis_root(use_cache)


def clear_cache():
    """Clear the cached paths, forcing re-discovery on next call."""
    global _cached_binary, _cached_root, _cache_initialized
    _cached_binary = None
    _cached_root = None
    _cache_initialized = False


def _is_valid_kleis_binary(path: str) -> bool:
    """Test if a path is a valid kleis binary."""
    if not path:
        return False
    try:
        result = subprocess.run(
            [path, "--version"],
            capture_output=True,
            text=True,
            timeout=5
        )
        return result.returncode == 0
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError):
        return False


def _is_valid_kleis_root(path: str) -> bool:
    """Test if a path is a valid Kleis project root."""
    if not path or not os.path.isdir(path):
        return False
    # Check for stdlib directory (required)
    stdlib_path = os.path.join(path, "stdlib")
    if not os.path.isdir(stdlib_path):
        return False
    # Check for prelude.kleis (sanity check)
    prelude_path = os.path.join(stdlib_path, "prelude.kleis")
    return os.path.isfile(prelude_path)


def get_status() -> dict:
    """Get status information about Kleis binary discovery.
    
    Useful for debugging and displaying to users.
    
    Returns:
        Dictionary with discovery status information.
    """
    binary = find_kleis_binary()
    root = find_kleis_root()
    
    status = {
        "binary_found": binary is not None,
        "binary_path": binary,
        "root_found": root is not None,
        "root_path": root,
        "KLEIS_ROOT": os.environ.get("KLEIS_ROOT"),
        "PATH_contains_kleis": shutil.which("kleis") is not None,
    }
    
    if binary:
        try:
            result = subprocess.run(
                [binary, "--version"],
                capture_output=True,
                text=True,
                timeout=5
            )
            status["version"] = result.stdout.strip() if result.returncode == 0 else "unknown"
        except Exception:
            status["version"] = "unknown"
    
    return status

