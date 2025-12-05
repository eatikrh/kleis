# Integral Transforms & POT - Quick Start

**Quick reference for the 16 new operations added to Kleis**

## Access

Open: **http://localhost:3000**

**Tabs:**
- **Calculus** → Scroll down → 7 transform buttons
- **POT** (far right) → 8 POT buttons

## Quick Reference

### Transforms (Calculus Tab)

```
ℱ[f](ω)      Fourier Transform
ℱ⁻¹[F](t)    Inverse Fourier
ℒ[f](s)      Laplace Transform
ℒ⁻¹[F](t)    Inverse Laplace
(f ∗ g)(x)   Convolution
∫_D K f dμ   Kernel Integral
G(x,m)       Green's Function
```

### POT (POT Tab)

```
Π[ψ](x)         Projection: Modal → Spacetime
∫_M f dμ(m)     Modal Integral
K(x,m)          Projection Kernel
c(x)            Causal Bound (VSL)
Residue[Π,X]    Constants as Residues
𝓜_name          Modal Space
ℝ⁴              Spacetime
𝓗_dim           Hont (Hilbert Ontology)
```

## Key POT Expression

```
Π[ψ](x) = ∫_M K(x,m) ψ(m) dμ(m)

Projection of modal state ψ to spacetime field φ
```

## Important: Text in Subscripts

**⚠️ When typing multi-letter text in Typst:**

✅ Use quotes: `"Hont"`, `"config"`, `"dimension"`  
❌ Don't type: `Hont`, `config`, `dimension` (causes "unknown variable" error)

**Single letters OK without quotes:**
```
✅ n, i, x, H, ∞    (no quotes needed)
```

## Quick Examples

**Projection:**
```
Insert: Π[□](□)
Fill:   ψ, x
→ Π[ψ](x)
```

**Hont:**
```
Insert: 𝓗_[□]
Fill:   "Hont"  (with quotes!)
→ 𝓗_("Hont")
```

**Fourier:**
```
Insert: ℱ[□](□)
Fill:   f, ω
→ ℱ[f](ω)
```

## Troubleshooting

**"Template not implemented"** → Refresh browser  
**"unknown variable: Hont"** → Use quotes: `"Hont"`  
**"unknown variable: variable"** → Server restarted, should be fixed

## Full Documentation

See: **`docs/INTEGRAL_TRANSFORMS_REFERENCE.md`**

