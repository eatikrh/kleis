(* Companion Isabelle proofs for epsilon_delta_limits.kleis *)

theory epsilon_delta_limits
imports Complex_Main
begin

(* Define predicates using epsilon-delta directly *)
(* Mark as [simp] so auto can unfold them *)
definition has_limit :: "(real \<Rightarrow> real) \<Rightarrow> real \<Rightarrow> real \<Rightarrow> bool" where [simp]:
  "has_limit f a L \<longleftrightarrow>
   (\<forall>\<epsilon>>0. \<exists>\<delta>>0. \<forall>x. \<bar>x - a\<bar> < \<delta> \<and> x \<noteq> a \<longrightarrow> \<bar>f x - L\<bar> < \<epsilon>)"

definition continuous_at :: "(real \<Rightarrow> real) \<Rightarrow> real \<Rightarrow> bool" where [simp]:
  "continuous_at f a \<longleftrightarrow>
   (\<forall>\<epsilon>>0. \<exists>\<delta>>0. \<forall>x. \<bar>x - a\<bar> < \<delta> \<longrightarrow> \<bar>f x - f a\<bar> < \<epsilon>)"

definition converges_to :: "(nat \<Rightarrow> real) \<Rightarrow> real \<Rightarrow> bool" where [simp]:
  "converges_to a L \<longleftrightarrow>
   (\<forall>\<epsilon>>0. \<exists>N. \<forall>n>N. \<bar>a n - L\<bar> < \<epsilon>)"

definition is_cauchy :: "(nat \<Rightarrow> real) \<Rightarrow> bool" where [simp]:
  "is_cauchy a \<longleftrightarrow>
   (\<forall>\<epsilon>>0. \<exists>N. \<forall>m n. m > N \<and> n > N \<longrightarrow> \<bar>a m - a n\<bar> < \<epsilon>)"

definition uniformly_continuous :: "(real \<Rightarrow> real) \<Rightarrow> bool" where [simp]:
  "uniformly_continuous f \<longleftrightarrow>
   (\<forall>\<epsilon>>0. \<exists>\<delta>>0. \<forall>x y. \<bar>x - y\<bar> < \<delta> \<longrightarrow> \<bar>f x - f y\<bar> < \<epsilon>)"

(* Simple proofs that complete quickly *)

lemma abs_zero: "\<forall>x::real. \<bar>x\<bar> = 0 \<longleftrightarrow> x = 0"
  by simp

lemma limit_definition:
  "\<forall>(f::real\<Rightarrow>real) L a. has_limit f a L \<longleftrightarrow>
   (\<forall>\<epsilon>::real. \<epsilon> > 0 \<longrightarrow> (\<exists>\<delta>::real. \<delta> > 0 \<and>
       (\<forall>x::real. \<bar>x - a\<bar> < \<delta> \<and> x \<noteq> a \<longrightarrow> \<bar>f x - L\<bar> < \<epsilon>)))"
  unfolding has_limit_def by auto

lemma continuous_preserves_limit:
  "\<forall>(f::real\<Rightarrow>real) a. continuous_at f a \<longrightarrow> has_limit f a (f a)"
  unfolding continuous_at_def has_limit_def
  by (metis less_eq_real_def)

lemma constant_has_limit:
  "\<forall>(c::real) (a::real). has_limit (\<lambda>x. c) a c"
  unfolding has_limit_def by auto

lemma identity_has_limit:
  "\<forall>(a::real). has_limit (\<lambda>x. x) a a"
  unfolding has_limit_def by auto

lemma constant_continuous:
  "\<forall>(c::real) (a::real). continuous_at (\<lambda>x. c) a"
  unfolding continuous_at_def by auto

lemma identity_continuous:
  "\<forall>(a::real). continuous_at (\<lambda>x. x) a"
  unfolding continuous_at_def by auto

end
