#!/usr/bin/env python3
"""
DLMF Equation Fetcher v2
Uses a curated list of interesting DLMF equations with direct LaTeX

Since DLMF uses complex MathML rendering, we'll use a hybrid approach:
1. Manually curated equations from documentation
2. Or use the DLMF LaTeX sources (if available)
3. Or fetch from specific well-known sections
"""

import argparse
from pathlib import Path
from typing import List, Dict


# Curated list of interesting DLMF equations by chapter
DLMF_EQUATIONS = {
    "gamma_function": [
        (r"\Gamma(z) = \int_0^\infty t^{z-1} e^{-t} \, dt", "5.2.1", "Gamma integral"),
        (r"\Gamma(z+1) = z\Gamma(z)", "5.5.1", "Recurrence"),
        (r"\Gamma(z)\Gamma(1-z) = \frac{\pi}{\sin(\pi z)}", "5.5.3", "Reflection"),
        (r"\Gamma\left(\frac{1}{2}\right) = \sqrt{\pi}", "5.5.5", "Half-integer"),
        (r"\psi(z) = \frac{d}{dz}\ln\Gamma(z) = \frac{\Gamma'(z)}{\Gamma(z)}", "5.2.2", "Digamma"),
    ],
    
    "bessel_functions": [
        (r"J_\nu(z) = \sum_{k=0}^\infty \frac{(-1)^k}{k!\,\Gamma(k+\nu+1)} \left(\frac{z}{2}\right)^{2k+\nu}", "10.2.2", "Bessel series"),
        (r"J_{-n}(z) = (-1)^n J_n(z)", "10.4.1", "Negative integer"),
        (r"Y_\nu(z) = \frac{J_\nu(z)\cos(\nu\pi) - J_{-\nu}(z)}{\sin(\nu\pi)}", "10.2.3", "Bessel Y"),
        (r"H_\nu^{(1)}(z) = J_\nu(z) + iY_\nu(z)", "10.2.5", "Hankel first"),
        (r"I_\nu(z) = e^{-\nu\pi i/2} J_\nu(ze^{\pi i/2})", "10.27.6", "Modified Bessel"),
    ],
    
    "hypergeometric": [
        (r"{}_2F_1(a,b;c;z) = \sum_{n=0}^\infty \frac{(a)_n(b)_n}{(c)_n} \frac{z^n}{n!}", "15.2.1", "Hypergeometric series"),
        (r"{}_2F_1(a,b;c;z) = \frac{\Gamma(c)}{\Gamma(b)\Gamma(c-b)} \int_0^1 t^{b-1}(1-t)^{c-b-1}(1-zt)^{-a} dt", "15.6.1", "Integral rep"),
        (r"{}_1F_1(a;b;z) = \sum_{n=0}^\infty \frac{(a)_n}{(b)_n} \frac{z^n}{n!}", "13.2.2", "Confluent"),
        (r"M(a,b,z) = e^z {}_1F_1(b-a;b;-z)", "13.2.39", "Kummer transform"),
    ],
    
    "legendre_polynomials": [
        (r"P_n(x) = \frac{1}{2^n n!} \frac{d^n}{dx^n}(x^2-1)^n", "18.5.5", "Rodrigues"),
        (r"P_n(x) = \sum_{k=0}^{\lfloor n/2\rfloor} \frac{(-1)^k(2n-2k)!}{2^n k!(n-k)!(n-2k)!} x^{n-2k}", "18.5.6", "Series"),
        (r"\int_{-1}^1 P_m(x) P_n(x) \, dx = \frac{2}{2n+1} \delta_{mn}", "18.3.1", "Orthogonality"),
        (r"(n+1)P_{n+1}(x) = (2n+1)xP_n(x) - nP_{n-1}(x)", "18.9.2", "Recurrence"),
    ],
    
    "zeta_function": [
        (r"\zeta(s) = \sum_{n=1}^\infty \frac{1}{n^s}", "25.2.1", "Zeta series"),
        (r"\zeta(s) = \frac{1}{\Gamma(s)} \int_0^\infty \frac{t^{s-1}}{e^t-1} \, dt", "25.5.1", "Integral"),
        (r"\zeta(s)\zeta(s-1) = \frac{\pi^s}{\Gamma(1-s)}", "25.4.1", "Functional eq (simplified)"),
        (r"\zeta(2) = \frac{\pi^2}{6}", "25.6.1", "Basel problem"),
        (r"\eta(s) = \sum_{n=1}^\infty \frac{(-1)^{n+1}}{n^s} = (1-2^{1-s})\zeta(s)", "25.7.1", "Dirichlet eta"),
    ],
    
    "elliptic_integrals": [
        (r"K(k) = \int_0^{\pi/2} \frac{d\theta}{\sqrt{1-k^2\sin^2\theta}}", "19.2.8", "Complete elliptic K"),
        (r"E(k) = \int_0^{\pi/2} \sqrt{1-k^2\sin^2\theta} \, d\theta", "19.2.9", "Complete elliptic E"),
        (r"F(\phi,k) = \int_0^\phi \frac{d\theta}{\sqrt{1-k^2\sin^2\theta}}", "19.2.4", "Incomplete F"),
        (r"E(\phi,k) = \int_0^\phi \sqrt{1-k^2\sin^2\theta} \, d\theta", "19.2.5", "Incomplete E"),
    ],
    
    "orthogonal_polynomials": [
        (r"H_n(x) = (-1)^n e^{x^2} \frac{d^n}{dx^n} e^{-x^2}", "18.5.9", "Hermite"),
        (r"L_n(x) = \frac{e^x}{n!} \frac{d^n}{dx^n}(x^n e^{-x})", "18.5.12", "Laguerre"),
        (r"T_n(x) = \cos(n\arccos x)", "18.3.1", "Chebyshev T"),
        (r"U_n(x) = \frac{\sin((n+1)\arccos x)}{\sin(\arccos x)}", "18.3.2", "Chebyshev U"),
    ],
    
    "special_cases": [
        (r"\int_0^\infty e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2}", "7.4.1", "Gaussian integral"),
        (r"B(x,y) = \frac{\Gamma(x)\Gamma(y)}{\Gamma(x+y)}", "5.12.1", "Beta function"),
        (r"\text{erf}(x) = \frac{2}{\sqrt{\pi}} \int_0^x e^{-t^2} \, dt", "7.2.1", "Error function"),
        (r"\text{Si}(x) = \int_0^x \frac{\sin t}{t} \, dt", "6.2.3", "Sine integral"),
        (r"\text{Ci}(x) = -\int_x^\infty \frac{\cos t}{t} \, dt", "6.2.4", "Cosine integral"),
    ],
}


def create_latex_file(equations: List[tuple], output_path: Path, title: str):
    """Create a LaTeX test file from equations."""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(f"% DLMF {title} - Curated Equations\n")
        f.write(f"% Source: https://dlmf.nist.gov/\n")
        f.write(f"% Manually curated for testing\n\n")
        
        for latex, eq_id, description in equations:
            f.write(f"% {eq_id}: {description}\n")
            # Wrap in display math
            if not latex.startswith('\\[') and not latex.startswith('$$'):
                f.write(f"\\[ {latex} \\]\n\n")
            else:
                f.write(f"{latex}\n\n")
    
    print(f"✓ Created {output_path} with {len(equations)} equations")


def main():
    parser = argparse.ArgumentParser(
        description='Generate DLMF equation test files from curated collection'
    )
    parser.add_argument(
        '--output',
        type=Path,
        default=Path('tests/golden/sources/dlmf'),
        help='Output directory for LaTeX files'
    )
    parser.add_argument(
        '--topics',
        type=str,
        default='all',
        help='Comma-separated list of topics (or "all")'
    )
    
    args = parser.parse_args()
    
    if args.topics == 'all':
        topics = list(DLMF_EQUATIONS.keys())
    else:
        topics = [t.strip() for t in args.topics.split(',')]
    
    print(f"DLMF Equation Generator v2")
    print(f"Topics: {', '.join(topics)}")
    print(f"Output: {args.output}\n")
    
    total = 0
    for topic in topics:
        if topic not in DLMF_EQUATIONS:
            print(f"⚠ Unknown topic: {topic}")
            continue
        
        equations = DLMF_EQUATIONS[topic]
        output_file = args.output / f"{topic}.tex"
        create_latex_file(equations, output_file, topic.replace('_', ' ').title())
        total += len(equations)
    
    print(f"\n✓ Done! Generated {total} equations across {len(topics)} files")
    print(f"\nNext steps:")
    print(f"  1. Review the files in {args.output}")
    print(f"  2. Run: cargo test golden_tests")
    print(f"  3. Check coverage with your template system")


if __name__ == '__main__':
    main()

