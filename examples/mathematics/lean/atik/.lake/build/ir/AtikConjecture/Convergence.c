// Lean compiler output
// Module: AtikConjecture.Convergence
// Imports: public import Init public import AtikConjecture.Basic public import Mathlib.Data.Complex.Basic public import Mathlib.Data.Matrix.Basic public import Mathlib.LinearAlgebra.Matrix.Defs public import Mathlib.Tactic.FinCases public import Mathlib.Tactic.LinearCombination
#include <lean/lean.h>
#if defined(__clang__)
#pragma clang diagnostic ignored "-Wunused-parameter"
#pragma clang diagnostic ignored "-Wunused-label"
#elif defined(__GNUC__) && !defined(__CLANG__)
#pragma GCC diagnostic ignored "-Wunused-parameter"
#pragma GCC diagnostic ignored "-Wunused-label"
#pragma GCC diagnostic ignored "-Wunused-but-set-variable"
#endif
#ifdef __cplusplus
extern "C" {
#endif
lean_object* lean_nat_mod(lean_object*, lean_object*);
static lean_once_cell_t lp_atik_AtikConjecture_spectralBlock___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_atik_AtikConjecture_spectralBlock___closed__0;
uint8_t lean_nat_dec_eq(lean_object*, lean_object*);
lean_object* lp_mathlib_Complex_ofReal(lean_object*);
lean_object* lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_spectralBlock(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_spectralBlock___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
extern lean_object* lp_mathlib_Real_definition_00___x40_Mathlib_Data_Real_Basic_1279875089____hygCtx___hyg_8_;
static lean_once_cell_t lp_atik_AtikConjecture_eigvec__plus___closed__0_once = LEAN_ONCE_CELL_INITIALIZER;
static lean_object* lp_atik_AtikConjecture_eigvec__plus___closed__0;
extern lean_object* lp_mathlib_Complex_I;
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__plus(lean_object*);
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__plus___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__minus(lean_object*);
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__minus___boxed(lean_object*);
static lean_object* _init_lp_atik_AtikConjecture_spectralBlock___closed__0(void) {
_start:
{
lean_object* x_1; lean_object* x_2; lean_object* x_3; 
x_1 = lean_unsigned_to_nat(2u);
x_2 = lean_unsigned_to_nat(0u);
x_3 = lean_nat_mod(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_spectralBlock(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
uint8_t x_5; 
x_5 = lean_nat_dec_eq(x_3, x_4);
if (x_5 == 0)
{
lean_object* x_6; uint8_t x_7; 
lean_dec(x_1);
x_6 = lean_obj_once(&lp_atik_AtikConjecture_spectralBlock___closed__0, &lp_atik_AtikConjecture_spectralBlock___closed__0_once, _init_lp_atik_AtikConjecture_spectralBlock___closed__0);
x_7 = lean_nat_dec_eq(x_3, x_6);
if (x_7 == 0)
{
lean_object* x_8; uint8_t x_9; 
x_8 = lp_mathlib_Complex_ofReal(x_2);
x_9 = !lean_is_exclusive(x_8);
if (x_9 == 0)
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_8, 0);
x_11 = lean_ctor_get(x_8, 1);
x_12 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_12, 0, x_10);
x_13 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_13, 0, x_11);
lean_ctor_set(x_8, 1, x_13);
lean_ctor_set(x_8, 0, x_12);
return x_8;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; 
x_14 = lean_ctor_get(x_8, 0);
x_15 = lean_ctor_get(x_8, 1);
lean_inc(x_15);
lean_inc(x_14);
lean_dec(x_8);
x_16 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_16, 0, x_14);
x_17 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_17, 0, x_15);
x_18 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_18, 0, x_16);
lean_ctor_set(x_18, 1, x_17);
return x_18;
}
}
else
{
lean_object* x_19; 
x_19 = lp_mathlib_Complex_ofReal(x_2);
return x_19;
}
}
else
{
lean_object* x_20; 
lean_dec(x_2);
x_20 = lp_mathlib_Complex_ofReal(x_1);
return x_20;
}
}
}
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_spectralBlock___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_atik_AtikConjecture_spectralBlock(x_1, x_2, x_3, x_4);
lean_dec(x_4);
lean_dec(x_3);
return x_5;
}
}
static lean_object* _init_lp_atik_AtikConjecture_eigvec__plus___closed__0(void) {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lp_mathlib_Real_definition_00___x40_Mathlib_Data_Real_Basic_1279875089____hygCtx___hyg_8_;
x_2 = lp_mathlib_Complex_ofReal(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__plus(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
x_2 = lean_obj_once(&lp_atik_AtikConjecture_spectralBlock___closed__0, &lp_atik_AtikConjecture_spectralBlock___closed__0_once, _init_lp_atik_AtikConjecture_spectralBlock___closed__0);
x_3 = lean_nat_dec_eq(x_1, x_2);
if (x_3 == 0)
{
lean_object* x_4; 
x_4 = lp_mathlib_Complex_I;
return x_4;
}
else
{
lean_object* x_5; 
x_5 = lean_obj_once(&lp_atik_AtikConjecture_eigvec__plus___closed__0, &lp_atik_AtikConjecture_eigvec__plus___closed__0_once, _init_lp_atik_AtikConjecture_eigvec__plus___closed__0);
return x_5;
}
}
}
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__plus___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_atik_AtikConjecture_eigvec__plus(x_1);
lean_dec(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__minus(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; 
x_2 = lean_obj_once(&lp_atik_AtikConjecture_spectralBlock___closed__0, &lp_atik_AtikConjecture_spectralBlock___closed__0_once, _init_lp_atik_AtikConjecture_spectralBlock___closed__0);
x_3 = lean_nat_dec_eq(x_1, x_2);
if (x_3 == 0)
{
lean_object* x_4; uint8_t x_5; 
x_4 = lp_mathlib_Complex_I;
x_5 = !lean_is_exclusive(x_4);
if (x_5 == 0)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_4, 0);
x_7 = lean_ctor_get(x_4, 1);
x_8 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_8, 0, x_6);
x_9 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_9, 0, x_7);
lean_ctor_set(x_4, 1, x_9);
lean_ctor_set(x_4, 0, x_8);
return x_4;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_4, 0);
x_11 = lean_ctor_get(x_4, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_4);
x_12 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_12, 0, x_10);
x_13 = lean_alloc_closure((void*)(lp_mathlib_Real_definition___lam__0_00___x40_Mathlib_Data_Real_Basic_2451848184____hygCtx___hyg_8_), 2, 1);
lean_closure_set(x_13, 0, x_11);
x_14 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_14, 0, x_12);
lean_ctor_set(x_14, 1, x_13);
return x_14;
}
}
else
{
lean_object* x_15; 
x_15 = lean_obj_once(&lp_atik_AtikConjecture_eigvec__plus___closed__0, &lp_atik_AtikConjecture_eigvec__plus___closed__0_once, _init_lp_atik_AtikConjecture_eigvec__plus___closed__0);
return x_15;
}
}
}
LEAN_EXPORT lean_object* lp_atik_AtikConjecture_eigvec__minus___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_atik_AtikConjecture_eigvec__minus(x_1);
lean_dec(x_1);
return x_2;
}
}
lean_object* initialize_Init(uint8_t builtin);
lean_object* initialize_atik_AtikConjecture_Basic(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_Data_Complex_Basic(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_Data_Matrix_Basic(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_LinearAlgebra_Matrix_Defs(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_Tactic_FinCases(uint8_t builtin);
lean_object* initialize_mathlib_Mathlib_Tactic_LinearCombination(uint8_t builtin);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_atik_AtikConjecture_Convergence(uint8_t builtin) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_atik_AtikConjecture_Basic(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_Data_Complex_Basic(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_Data_Matrix_Basic(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_LinearAlgebra_Matrix_Defs(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_Tactic_FinCases(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_mathlib_Mathlib_Tactic_LinearCombination(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
