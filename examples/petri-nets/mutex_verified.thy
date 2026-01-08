(* Companion Isabelle theory for mutex_verified.kleis
   
   Provides definitions for the semantic operations that Isabelle
   cannot auto-prove as biconditionals.
*)

theory mutex_verified
imports Complex_Main
begin

(* Operations as functions with specific values *)
consts iw :: "int \<Rightarrow> int \<Rightarrow> int"
consts ow :: "int \<Rightarrow> int \<Rightarrow> int"
consts m :: "int \<Rightarrow> int \<Rightarrow> int"
consts reach :: "int \<Rightarrow> int \<Rightarrow> bool"
consts fire :: "int \<Rightarrow> int \<Rightarrow> int \<Rightarrow> bool"

(* Enabled: transition t is enabled in marking mi *)
definition enabled_fn :: "int \<Rightarrow> int \<Rightarrow> bool" where
  "enabled_fn mi t \<longleftrightarrow> (
     m mi 0 \<ge> iw t 0 \<and>
     m mi 1 \<ge> iw t 1 \<and>
     m mi 2 \<ge> iw t 2 \<and>
     m mi 3 \<ge> iw t 3 \<and>
     m mi 4 \<ge> iw t 4)"

(* Enabled as axiomatized predicate matching Kleis *)
consts enabled :: "int \<Rightarrow> int \<Rightarrow> bool"
axiomatization where enabled_is_enabled_fn: "enabled = enabled_fn"

(* --- Proofs for Kleis axioms --- *)

(* EnablingSemantics.enabled_def *)
lemma enabled_def: 
  "\<forall>(mi :: int). \<forall>(t :: int).
     enabled mi t \<longleftrightarrow> (
       m mi 0 \<ge> iw t 0 \<and>
       m mi 1 \<ge> iw t 1 \<and>
       m mi 2 \<ge> iw t 2 \<and>
       m mi 3 \<ge> iw t 3 \<and>
       m mi 4 \<ge> iw t 4)"
  unfolding enabled_is_enabled_fn enabled_fn_def by auto

(* FiringSemantics.fire_def - defined as axiomatization since fire is a relation *)
lemma fire_def:
  "\<forall>(mi :: int). \<forall>(t :: int). \<forall>(mj :: int).
     fire mi t mj \<longleftrightarrow> (
       enabled mi t \<and>
       m mj 0 = m mi 0 - iw t 0 + ow t 0 \<and>
       m mj 1 = m mi 1 - iw t 1 + ow t 1 \<and>
       m mj 2 = m mi 2 - iw t 2 + ow t 2 \<and>
       m mj 3 = m mi 3 - iw t 3 + ow t 3 \<and>
       m mj 4 = m mi 4 - iw t 4 + ow t 4)"
  sorry (* This requires axiomatizing fire's semantics *)

(* ReachabilitySemantics.reach_step *)
lemma reach_step:
  "\<forall>(mi :: int). \<forall>(mj :: int). \<forall>(mk :: int). \<forall>(t :: int).
     (reach mi mj \<and> fire mj t mk) \<longrightarrow> reach mi mk"
  sorry (* This requires inductive definition of reach *)

(* TokenConservation.total_tokens *)
lemma total_tokens:
  "\<forall>(mi :: int).
     reach 0 mi \<longrightarrow> (m mi 0 + m mi 1 + m mi 2 + m mi 3 + m mi 4 = 3)"
  sorry (* This requires proving invariant preservation *)

end

