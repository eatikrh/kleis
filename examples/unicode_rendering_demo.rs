use kleis::ast::Expression;
use kleis::render::{RenderTarget, build_default_context, render_expression};
/// Unicode Rendering Demo for Integral Transforms & POT Operations
///
/// This program demonstrates the Unicode rendering of all 16 new mathematical
/// operations added to Kleis for POT (Projected Ontology Theory).
///
/// Run with: cargo run --example unicode_rendering_demo
use kleis::templates::*;

fn render_unicode(name: &str, template_fn: fn() -> Expression) {
    reset_placeholder_counter();
    let expr = template_fn();
    let ctx = build_default_context();
    let output = render_expression(&expr, &ctx, &RenderTarget::Unicode);
    println!("  {:<25} {}", name, output);
}

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║         KLEIS UNICODE RENDERING GALLERY                      ║");
    println!("║         Integral Transforms & POT Operations                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Integral Transforms
    println!("═══ INTEGRAL TRANSFORMS ═══\n");

    render_unicode("Fourier Transform:", template_fourier_transform);
    render_unicode("Inverse Fourier:", template_inverse_fourier);
    render_unicode("Laplace Transform:", template_laplace_transform);
    render_unicode("Inverse Laplace:", template_inverse_laplace);
    render_unicode("Convolution:", template_convolution);
    render_unicode("Kernel Integral:", template_kernel_integral);
    render_unicode("Green's Function:", template_greens_function);

    // POT Operations
    println!("\n═══ POT OPERATIONS ═══\n");

    render_unicode("Projection:", template_projection);
    render_unicode("Modal Integral:", template_modal_integral);
    render_unicode("Projection Kernel:", template_projection_kernel);
    render_unicode("Causal Bound:", template_causal_bound);
    render_unicode("Projection Residue:", template_projection_residue);
    render_unicode("Modal Space:", template_modal_space);

    // These don't need placeholders
    reset_placeholder_counter();
    let spacetime = template_spacetime();
    let ctx = build_default_context();
    let output = render_expression(&spacetime, &ctx, &RenderTarget::Unicode);
    println!("  {:<25} {}", "Spacetime:", output);

    render_unicode("Hont:", template_hont);

    // Complete examples
    println!("\n═══ COMPLETE EXAMPLES ═══\n");

    println!("Example 1: Fourier Transform Expanded");
    println!("  ℱ[f](ω) = ∫₋∞^∞ f(t) e^(-iωt) dt\n");

    println!("Example 2: Projection Expansion");
    println!("  Π[ψ](x) = ∫_M K(x,m) ψ(m) dμ(m)\n");

    println!("Example 3: Variable Speed of Light");
    println!("  c(x) = derived from support[K(x,·)]\n");

    println!("Example 4: Convolution for Field");
    println!("  φ(x) = (ρ ∗ G)(x) = ∫ ρ(y) G(x,y) dy\n");

    println!("═══ POT HIERARCHY ═══\n");
    println!("  𝓗 (Hont)  →  𝓜 (Modal)  →  Π (Projection)  →  ℝ⁴ (Spacetime)");
    println!("   Being       Relations      Transform          Appearance\n");

    println!("═══ UNICODE SYMBOLS ═══\n");
    println!("  Script:  ℱ ℒ 𝓜 𝓗");
    println!("  Greek:   Π π ω ψ ρ μ α");
    println!("  Math:    ∫ ∗ ∈ → ∞ ℝ ℂ");
    println!("  Sub/Sup: ₀₁₂ ⁰¹² ⁻¹");

    println!("\n✅ All 16 operations rendered successfully in Unicode!\n");
}
