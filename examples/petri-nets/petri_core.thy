(* Companion Isabelle theory for petri_core.kleis
   
   Provides formal definitions and proofs for Petri net axioms.
*)

theory petri_core
imports Complex_Main "HOL-Library.Multiset"
begin

(* Type aliases matching Kleis definitions *)
type_synonym Marking = "int \<Rightarrow> nat"

(* Operations as uninterpreted functions *)
consts 
  input_weight :: "int \<Rightarrow> int \<Rightarrow> nat"
  output_weight :: "int \<Rightarrow> int \<Rightarrow> nat"
  enabled :: "int \<Rightarrow> Marking \<Rightarrow> bool"
  fire :: "int \<Rightarrow> Marking \<Rightarrow> Marking"
  k_bounded :: "int \<Rightarrow> nat \<Rightarrow> Marking \<Rightarrow> bool"

(* Reachability is defined inductively *)
inductive reachable :: "Marking \<Rightarrow> Marking \<Rightarrow> bool" where
  refl: "reachable m m" |
  step: "\<lbrakk>reachable m1 m2; enabled t m2\<rbrakk> \<Longrightarrow> reachable m1 (fire t m2)"

(* Safe = 1-bounded *)
definition safe :: "int \<Rightarrow> Marking \<Rightarrow> bool" where
  "safe p m0 \<longleftrightarrow> k_bounded p 1 m0"

(* --- Proofs for Kleis axioms --- *)

(* PetriNetSemantics.reachable_refl *)
lemma reachable_refl: "\<forall>(m :: Marking). reachable m m"
  using reachable.refl by auto

(* SafetyProperties.safe_is_1bounded *)
lemma safe_is_1bounded: "\<forall>(p :: int). \<forall>(m0 :: Marking). safe p m0 \<longleftrightarrow> k_bounded p 1 m0"
  unfolding safe_def by auto

end

