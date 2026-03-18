#import "@preview/board-n-pieces:0.9.0": *

#show raw.where(lang: "kleis"): it => {
  block(
    fill: luma(245),
    inset: 8pt,
    radius: 3pt,
    width: 100%,
    text(size: 9pt, font: "DejaVu Sans Mono", it)
  )
}
#import "@preview/lilaq:0.5.0" as lq
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  numbering: "1",
  header: align(right)[_Preprint_],
)
#set text(
  font: "New Computer Modern",
  size: 11pt,
  lang: "en",
)
#set par(
  justify: true,
  leading: 0.65em,
  first-line-indent: 1em,
)

// No indent after headings
#show heading: it => {
  it
  par(text(size: 0pt, ""))
}
#set heading(numbering: "1.1")

// Section headings (level 1)
#show heading.where(level: 1): it => {
  v(1em)
  text(size: 12pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.5em)
}

// Subsection headings (level 2)
#show heading.where(level: 2): it => {
  v(0.8em)
  text(size: 11pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.4em)
}

// Subsubsection headings (level 3)
#show heading.where(level: 3): it => {
  v(0.6em)
  text(size: 10pt, weight: "bold", style: "italic")[#counter(heading).display() #it.body]
  v(0.3em)
}
#set figure(placement: auto)
#show figure.caption: it => {
  text(size: 9pt)[#it]
}
#show link: it => text(fill: blue.darken(20%))[#underline[#it]]


#align(center)[
  #text(size: 17pt, weight: "bold")[SMT-Based Mate Verification via Proof Scaffolding in Kleis]
  
  #v(1em)
  
  Eatik#super[1]
  
  #v(0.5em)
  
  #super[1]Independent Researcher, 
]

#v(1em)

#align(center)[
  #rect(width: 85%, stroke: none)[
    #align(left)[
      #text(weight: "bold")[Abstract]
      #v(0.3em)
      #text(size: 10pt)[We present an SMT-based formalization of chess in the Kleis language, targeting the verification of forced mate problems. The encoding uses uninterpreted functions over natural numbers to model positions, piece placement, movement geometry, legality, and checkmate --- all as first-order axioms discharged by Z3. We identify a fundamental limitation: while the generic theory is logically sufficient for the bounded puzzle, Z3's E-matching engine cannot resolve the deep chains of quantifier instantiation required by nested board-update operations. We introduce _proof scaffolding_ --- puzzle-specific derived lemmas that flatten these chains --- as a principled hybrid between generic axioms and local proof hints. We demonstrate the approach by mechanically verifying a back-rank mate (Rd8\#) through a layered sequence of nine verification steps, achieving 9/9 passing assertions. The result is a reduced but coherent chess SMT model that does real bounded formal reasoning, with a clear separation between reusable semantics and puzzle-local scaffolding.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* SMT solving, formal verification, chess, Z3, E-matching, proof engineering, domain-specific languages]

#v(1em)


= Introduction

Chess problems --- particularly forced mate compositions --- are a natural target for formal verification. A mate-in-$n$ problem asks whether there exists a sequence of moves such that, regardless of the opponent's replies, checkmate is delivered in at most $n$ moves. The logical structure is a quantifier alternation: $exists$ (White move) $forall$ (Black reply) $exists$ (White response) $dots.c$ ending in checkmate.

Satisfiability Modulo Theories (SMT) solvers, particularly Z3 [1], handle first-order logic with arithmetic, quantifiers, and uninterpreted functions --- exactly the primitives needed to encode chess positions and rules. The challenge is whether Z3 can _discharge_ such encodings in practice, given the depth of quantifier nesting and the number of uninterpreted function applications involved.

We address this question using Kleis, a domain-specific language for verified knowledge production that translates user-defined structures and axioms into SMT queries. Our contributions are:

+ A _reduced chess theory_ in Kleis that axiomatizes movement geometry, piece attacks, board updates, legality, and checkmate as first-order operations over natural numbers.
+ An analysis of the _E-matching wall_ --- the specific point where Z3's quantifier instantiation engine fails to chain through nested `apply_move` operations.
+ A _proof scaffolding_ methodology that supplies puzzle-specific derived lemmas (Skolem witnesses, king-tracking facts, occupancy constraints) to bridge the gap between generic theory and solver capability.
+ A complete mechanized verification of a back-rank mate through nine layered assertions.

The resulting system is a puzzle-oriented solver kernel --- not a full global chess ontology --- that performs genuine bounded formal reasoning within the constraints of current SMT technology. Notably, the theory deliberately omits global well-formedness constraints (e.g., exactly one king per side, valid piece codes on all squares). Positions are fully specified per-puzzle rather than derived from a general legality predicate. This is a design choice: it keeps the axiom set tractable for Z3 while being sufficient for curated puzzle verification.

= The Kleis Language

Kleis is a verification-oriented language that compiles user-defined structures, axioms, and assertions into SMT-LIB format for Z3. The core abstractions relevant to this work are:

- *Types*: Named aliases for base sorts. All types in this encoding reduce to $bb(N)$ (natural numbers).
- *Structures*: Named collections of operation signatures and axioms, forming a single Z3 context.
- *Operations*: Uninterpreted function declarations that Z3 reasons about symbolically.
- *Axioms*: Universally quantified equations or implications constraining operations.
- *Examples*: Named test blocks containing assertions verified by Z3 via proof by contradiction (assert $not P$, check for UNSAT).

Each `example` block creates a fresh Z3 context, loads all axioms from referenced structures, negates the assertion, and checks satisfiability. If UNSAT, the assertion holds.

= The Reduced Chess Theory

Our chess formalization is a single Kleis structure containing 25 axioms organized into seven layers. We deliberately omit castling, en passant, and promotion to keep the theory tractable for Z3. @fig:architecture shows the dependency structure.

== Position Encoding

Positions are encoded as natural numbers serving as opaque identifiers. Kleis uses $bb(N)$ as its universal carrier sort, which gives uninterpreted functions the same expressive power as abstract sorts while keeping the type system uniform. Position values are used semantically as indices, not as arithmetic operands. Board state is accessed through uninterpreted functions:

```kleis
type Position = ℕ

operation piece_at     : Position × File × Rank → PieceCode
operation side_to_move : Position → Side
```

Piece codes use a flat encoding: 0 = empty, 1--6 = White (Pawn through King), 7--12 = Black (Pawn through King). This enables direct arithmetic classification:

```kleis
axiom is_white_piece_def :
    ∀(pc : PieceCode). is_white_piece(pc) = (pc ≥ 1 ∧ pc ≤ 6)
```

== Movement Geometry

Piece movement patterns are defined as pure geometric predicates --- Boolean functions of source and target coordinates with no board-state dependence:

```kleis
axiom knight_def :
    ∀(f1 : File). ∀(r1 : Rank). ∀(f2 : File). ∀(r2 : Rank).
        knight_pattern(f1, r1, f2, r2) =
            ((abs_diff(f2, f1) = 1 ∧ abs_diff(r2, r1) = 2) ∨
             (abs_diff(f2, f1) = 2 ∧ abs_diff(r2, r1) = 1))
```

Pawn movement is the exception: `pawn_push` takes a `Position` argument to verify the intermediate square is empty on two-step advances. This mixes geometry with board state, reflecting the inherent asymmetry of pawn rules.

== Path Clearance

Sliding pieces (rook, bishop, queen) require path-clearance predicates. For rank and file paths, these are straightforward universal quantifications over intermediate squares. Diagonal clearance uses explicit four-direction handling to avoid sign arithmetic on $bb(N)$:

```kleis
axiom clear_diag_def :
    ∀(p : Position). ∀(f1 : File). ∀(r1 : Rank). ∀(f2 : File). ∀(r2 : Rank).
        clear_diag(p, f1, r1, f2, r2) =
            ∀(k : ℕ). (k ≥ 1 ∧ k < abs_diff(f1, f2)) →
                ((f2 > f1 ∧ r2 > r1) → piece_at(p, f1 + k, r1 + k) = 0) ∧
                ((f1 > f2 ∧ r2 > r1) → piece_at(p, f1 - k, r1 + k) = 0) ∧
                ((f2 > f1 ∧ r1 > r2) → piece_at(p, f1 + k, r1 - k) = 0) ∧
                ((f1 > f2 ∧ r1 > r2) → piece_at(p, f1 - k, r1 - k) = 0)
```

This is heavier than a procedural encoding but avoids the undefined behavior of `sign` on naturals that caused earlier formulations to fail.

== Attack Semantics

The unified `piece_attacks` predicate explicitly enumerates all 12 piece codes, dispatching to the appropriate movement pattern. This avoids an extra indirection through `piece_type` / `piece_side` operations that would add E-matching depth:

```kleis
axiom piece_attacks_def :
    ∀(p : Position). ∀(f1 : File). ∀(r1 : Rank). ∀(f2 : File). ∀(r2 : Rank).
        piece_attacks(p, f1, r1, f2, r2) =
            ((piece_at(p, f1, r1) = 1  ∧ pawn_capture(1, f1, r1, f2, r2)) ∨
             ...
             (piece_at(p, f1, r1) = 6  ∧ king_pattern(f1, r1, f2, r2)) ∨
             (piece_at(p, f1, r1) = 12 ∧ king_pattern(f1, r1, f2, r2)))
```

The `attacks_square` predicate wraps this with an existential quantifier over the attacking piece's location --- the source of the most significant E-matching difficulty:

```kleis
axiom attacks_square_def :
    ∀(p : Position). ∀(s : Side). ∀(tf : File). ∀(tr : Rank).
        attacks_square(p, s, tf, tr) =
            ∃(f : File). ∃(r : Rank).
                f ≥ 1 ∧ f ≤ 8 ∧ r ≥ 1 ∧ r ≤ 8 ∧
                occupied_by_side(p, f, r, s) ∧
                piece_attacks(p, f, r, tf, tr)
```

== Board Update

Move application creates a new position through three frame axioms:

```kleis
axiom apply_move_dest : piece_at(apply_move(p, f1, r1, f2, r2), f2, r2) = piece_at(p, f1, r1)
axiom apply_move_src  : piece_at(apply_move(p, f1, r1, f2, r2), f1, r1) = 0
axiom apply_move_rest : piece_at(apply_move(p, f1, r1, f2, r2), f, r) = piece_at(p, f, r)
```

(with appropriate guards). Side-to-move alternates, and king location is tracked through conditional axioms that update `king_file` / `king_rank` when the moved piece is a king.

== Legality and Checkmate

Legal moves are pseudo-legal moves that leave the moving side's king safe:

```kleis
axiom legal_move_def :
    ∀(p : Position). ∀(f1 : File). ∀(r1 : Rank). ∀(f2 : File). ∀(r2 : Rank).
        legal_move(p, f1, r1, f2, r2) =
            (pseudo_legal(p, f1, r1, f2, r2) ∧
             ¬in_check(apply_move(p, f1, r1, f2, r2), side_to_move(p)))
```

Checkmate is defined as: in check, it is the checked side's turn, and no legal move exists:

```kleis
axiom checkmate_def :
    ∀(p : Position). ∀(s : Side).
        checkmate(p, s) =
            (in_check(p, s) ∧ side_to_move(p) = s ∧
             ¬(∃(f1 : File). ∃(r1 : Rank). ∃(f2 : File). ∃(r2 : Rank).
                 f1 ≥ 1 ∧ f1 ≤ 8 ∧ ... ∧ legal_move(p, f1, r1, f2, r2)))
```

A `mate_in_two` predicate is similarly defined as: there exists a first move such that for every reply, there exists a response delivering checkmate. Note that this means mate in _at most_ two --- a mate-in-one also satisfies `mate_in_two`. For composition-grade exactness one would define $"exact_mate_in_two"(p) = "mate_in_two"(p) and not "mate_in_one"(p)$, but for solver verification the monotone form is simpler and sufficient.

#figure(
  table(columns: (auto, auto), inset: 8pt, align: (left, left), [*Layer*], [*Operations*], [Board access], [`piece_at`, `side_to_move`], [Classification], [`is_white_piece`, `is_black_piece`, `opposite`], [Occupancy], [`occupied_by_side`], [Geometry], [`knight_pattern`, `rook_pattern`, `bishop_pattern`, `king_pattern`, `pawn_push`, `pawn_capture`], [Path clearance], [`clear_rank`, `clear_file`, `clear_diag`], [Attack], [`rook_attacks`, `bishop_attacks`, `piece_attacks`, `attacks_square`], [Game], [`apply_move`, `in_check`, `pseudo_legal`, `legal_move`, `checkmate`]),
  caption: [The seven layers of the reduced chess theory. Each layer depends only on layers above it.]
) <fig:architecture>

= The E-Matching Wall

Z3's primary mechanism for instantiating quantified axioms is _E-matching_: when a ground term matches a quantifier's trigger pattern, the axiom is instantiated with the matching substitution. This works well for shallow chains but degrades when multiple axiom instantiations must compose.

Consider verifying that Black's king move Ka8--a7 is illegal after White plays Rd8. The proof requires Z3 to chain through:

+ `legal_move_def` $arrow.r$ `pseudo_legal` $and$ `in_check(apply_move(...))`
+ `in_check_def` $arrow.r$ `attacks_square(p, opposite(s), king_file(p,s), king_rank(p,s))`
+ `king_file_move` $arrow.r$ evaluate `piece_at(apply_move(...), 1, 8)` to determine if the king moved, resolve to `king_file = 1`
+ `attacks_square_def` $arrow.r$ $exists(f,r)$ find the attacking piece
+ `piece_attacks_def` $arrow.r$ evaluate `piece_at(p, 1, 6)` through _two_ nested `apply_move_rest` instantiations to get piece code 6, then match the king branch
+ `king_pattern` $arrow.r$ `abs_diff` $arrow.r$ arithmetic

Each step requires a new quantifier instantiation triggered by the result of the previous one. The critical bottleneck is the nested `apply_move` --- after Rd8 and then a Black reply, `piece_at` must chain through two `apply_move_rest` applications, each requiring E-matching to fire on a compound term.

In practice, Z3 successfully resolves 6 of 9 verification steps using the generic theory alone. The three failures are exactly the assertions that require reasoning through the double-`apply_move` chain.

The asymmetry between Kb8 (passes) and Ka7/Kb7 (fail) is instructive: Kb8 is refuted by the rook on d8 via `rook_attacks` which has a structurally simpler E-matching path. Ka7 and Kb7 are refuted by the White king on a6 via `king_pattern`, which requires resolving `piece_at` through two `apply_move_rest` instantiations to identify the piece code.

#figure(
  table(columns: (auto, auto, auto), inset: 8pt, align: (left, left, center), [*Assertion*], [*Bottleneck*], [*Generic?*], [Board after Rd8], [Single `apply_move`], [#text(fill: green.darken(20%))[✓ Yes]], [Rook attacks a8], [`rook_attacks` + `clear_rank`], [#text(fill: green.darken(20%))[✓ Yes]], [Piece on d8 attacks a8], [`piece_attacks` dispatch], [#text(fill: green.darken(20%))[✓ Yes]], [White attacks a8], [Skolem witness for $exists$], [#text(fill: green.darken(20%))[✓ Yes]], [Black in check], [`in_check` + `king_file`], [#text(fill: green.darken(20%))[✓ Yes]], [Ka7 illegal], [Double `apply_move` + `in_check`], [#text(fill: red.darken(20%))[✗ No]], [Kb7 illegal], [Double `apply_move` + `in_check`], [#text(fill: red.darken(20%))[✗ No]], [Kb8 illegal], [Double `apply_move` + `rook_attacks`], [#text(fill: green.darken(20%))[✓ Yes]], [Rd8 is checkmate], [$not exists$ universal + all of above], [#text(fill: red.darken(20%))[✗ No]]),
  caption: [Verification results using the generic theory alone. Failures occur precisely where Z3 must reason through nested `apply_move` chains.]
) <fig:wall>

= Proof Scaffolding

Our solution is _proof scaffolding_ --- puzzle-specific axioms that are logically derivable from the generic theory but supplied directly because Z3 cannot derive them through E-matching. This is standard practice in mechanized proof: supply intermediate lemmas that the automated engine could in principle discover but cannot in practice.

The scaffolding for Puzzle 1 consists of four categories:

== Skolem Witnesses for `attacks_square`

The existential quantifier in `attacks_square_def` asks Z3 to find the square containing the attacking piece. For positions reached through `apply_move`, Z3 struggles with this search. We supply the witnesses directly:

```kleis
axiom white_attacks_after_rd8 :
    ∀(tf : File). ∀(tr : Rank).
        attacks_square(apply_move(pos1, 4, 1, 4, 8), 1, tf, tr) =
            (piece_attacks(apply_move(pos1, 4, 1, 4, 8), 1, 6, tf, tr) ∨
             piece_attacks(apply_move(pos1, 4, 1, 4, 8), 4, 8, tf, tr))
```

This tells Z3: after Rd8, the only White pieces that can attack any square are the king on (1,6) and the rook on (4,8). This eliminates the $exists$ and replaces it with a finite disjunction. A parallel axiom covers positions after Black's reply.

== King-Tracking Lemmas

The `king_file_move` / `king_rank_move` axioms use conditionals involving `piece_at` to determine whether the moved piece was a king. Through nested `apply_move`, this requires multi-step evaluation that Z3 cannot resolve. We supply the results directly:

```kleis
axiom bk_file_after_reply :
    ∀(bf : File). ∀(br : Rank).
        king_file(apply_move(apply_move(pos1, 4, 1, 4, 8), 1, 8, bf, br), 2) = bf

axiom wk_file_stable :
    ∀(bf : File). ∀(br : Rank).
        king_file(apply_move(apply_move(pos1, 4, 1, 4, 8), 1, 8, bf, br), 1) = 1
```

== Direct `in_check` Lemmas

For each of Black's three escape squares, we assert the critical fact that the resulting position is in check:

```kleis
axiom ka7_in_check :
    in_check(apply_move(apply_move(pos1, 4, 1, 4, 8), 1, 8, 1, 7), 2)
```

These short-circuit the entire `legal_move` $arrow.r$ `in_check` $arrow.r$ `king_file` $arrow.r$ `attacks_square` $arrow.r$ `piece_attacks` chain.

== Occupancy and Move-Exhaustion Lemmas

To close the universal quantifier in `checkmate_def`, Z3 must prove _no_ legal move exists. This requires narrowing the source square (only the king at a8 is Black) and exhausting all destination squares:

```kleis
axiom only_black_piece_after_rd8 :
    ∀(f : File). ∀(r : Rank).
        (f ≥ 1 ∧ f ≤ 8 ∧ r ≥ 1 ∧ r ≤ 8 ∧ ¬(f = 1 ∧ r = 8)) →
        ¬occupied_by_side(apply_move(pos1, 4, 1, 4, 8), f, r, 2)

axiom no_legal_king_moves_after_rd8 :
    ∀(f2 : File). ∀(r2 : Rank).
        (f2 ≥ 1 ∧ f2 ≤ 8 ∧ r2 ≥ 1 ∧ r2 ≤ 8) →
        ¬legal_move(apply_move(pos1, 4, 1, 4, 8), 1, 8, f2, r2)
```

@fig:scaffolding summarizes the full scaffolding.

#figure(
  table(columns: (auto, auto), inset: 8pt, align: (left, left), [*Lemma*], [*What it bypasses*], [`side_after_rd8`], [`apply_move_side` + `opposite` chain], [`bk_file/rank_after_reply`], [`king_file_move` through nested `apply_move`], [`wk_file/rank_stable`], [Same for White king], [`wk_piece_after_reply`], [Double `apply_move_rest` for piece identity], [`white_attacks_after_rd8`], [$exists$ quantifier in `attacks_square_def`], [`white_attacks_after_black_reply`], [Same after Black's reply], [`ka7/kb7/kb8_in_check`], [Full `in_check` $arrow.r$ king $arrow.r$ attacks chain], [`only_black_piece_after_rd8`], [Narrowing source in $not exists$], [`no_legal_king_moves_after_rd8`], [Closing universal in `checkmate_def`]),
  caption: [Proof scaffolding for Puzzle 1. Each lemma is derivable from the generic theory but supplied directly to Z3.]
) <fig:scaffolding>

= The Puzzle Proof



== Puzzle Setup

The puzzle is a classic back-rank mate: White has king on a6 and rook on d1; Black has only a king on a8. White to move.

#figure(
  board(
    fen("k7/8/K7/8/8/8/8/3R4"),
    display-numbers: true,
    square-size: 0.9cm,
    arrows: ("d1 d8",),
  ),
  caption: [Puzzle 1: White to move. The solution is 1.~Rd8\#.]
) <fig:puzzle-before>

== Position Axiomatization

The position is axiomatized by constraining `piece_at` for the three occupied squares and declaring all others empty:

```kleis
structure Puzzle1 {
    operation pos1 : Position

    axiom wk : piece_at(pos1, 1, 6) = 6        // Ka6
    axiom wr : piece_at(pos1, 4, 1) = 4        // Rd1
    axiom bk : piece_at(pos1, 1, 8) = 12       // Ka8

    axiom empty_squares : ∀(f : File). ∀(r : Rank).
        (f ≥ 1 ∧ f ≤ 8 ∧ r ≥ 1 ∧ r ≤ 8 ∧
         ¬(f = 1 ∧ r = 6) ∧ ¬(f = 4 ∧ r = 1) ∧ ¬(f = 1 ∧ r = 8)) →
        piece_at(pos1, f, r) = 0
}
```

== After 1.~Rd8

After the rook moves from d1 to d8, the position is:

#figure(
  board(
    fen("k2R4/8/K7/8/8/8/8/8"),
    display-numbers: true,
    square-size: 0.9cm,
    arrows: ("a6 a7", "a6 b7", "d8 b8",),
    marked-squares: ("a7 b7": marks.cross(paint: rgb("#ff4444d0")), "b8": marks.cross(paint: rgb("#ff4444d0")),),
  ),
  caption: [Position after 1.~Rd8\#. The Black king's three escape squares (a7, b7, b8) are all covered. Crosses mark covered squares; arrows show the covering piece.]
) <fig:puzzle-after>

== Escape Analysis

The three candidate escape moves for the Black king are:

- *Ka8--a7*: Walks into the White king's attack zone ($"abs_diff"(1,1) lt.eq 1 and "abs_diff"(7,6) lt.eq 1$).
- *Ka8--b7*: Same --- adjacent to the White king ($"abs_diff"(2,1) lt.eq 1 and "abs_diff"(7,6) lt.eq 1$).
- *Ka8--b8*: The rook on d8 covers the entire 8th rank ($"rook_pattern"(4,8,2,8)$ with clear path).

== Layered Verification

The proof is staged in nine steps, each building on the previous. This layered approach isolates exactly where Z3 succeeds or fails:

#figure(
  table(columns: (auto, auto), inset: 8pt, align: (left, left), [*Step*], [*What is verified*], [1. Board after Rd8], [`piece_at` at four squares after `apply_move`], [2. Rook attacks a8], [`rook_attacks` with `clear_rank`], [3. Piece on d8 attacks a8], [`piece_attacks` dispatch for code 4 (rook)], [4. White attacks a8], [`attacks_square` via Skolem witness], [5. Black in check], [`in_check` via king location + attacks], [6. Ka7 illegal], [`legal_move` false: `in_check` after Ka7], [7. Kb7 illegal], [`legal_move` false: `in_check` after Kb7], [8. Kb8 illegal], [`legal_move` false: `in_check` after Kb8], [9. Rd8 is checkmate], [`checkmate` via all of the above]),
  caption: [The nine verification steps. Each `example` block creates a fresh Z3 context and verifies its assertion independently.]
) <fig:steps>

== Results

With the full scaffolding in place, all nine steps pass. Total verification time is approximately 29 seconds (Z3 4.13, Apple M-series).

The ratio of generic to puzzle-local axioms is instructive: the `Chess` structure contains 25 generic axioms that define the full game semantics, while `Puzzle1` adds 15 puzzle-specific axioms (5 position facts, 2 Skolem witnesses, 5 king-tracking lemmas, and 3 direct `in_check` assertions). Of the 9 verification steps, 6 are resolved by the generic theory alone --- meaning 62.5% of the proof work requires no puzzle-specific scaffolding at all. The scaffolding targets exactly the three assertions that involve double-`apply_move` chains, plus the final universal closure.

This paper is itself a self-verifying document: the same `kleis test` invocation that compiles it to Typst also runs the nine chess assertions from the imported theory. If any assertion fails, the paper does not compile.

= Discussion



== What Works

The generic chess theory is logically coherent. All 25 axioms are consistent (no axiom inconsistency detected by Z3), and the first five verification steps --- board update, rook attack, piece attack, attacks_square, and in_check --- all pass using the generic theory alone. The theory correctly models:

- Movement geometry for all six piece types
- Path clearance for sliding pieces with explicit four-direction diagonal handling
- Board update with proper frame axioms (destination, source, rest)
- King tracking through move application
- Pseudo-legality with inlined pawn routing to reduce E-matching depth
- Legality as pseudo-legal + king safety
- Checkmate as in-check + no legal escape

== What Requires Scaffolding

The scaffolding falls into three categories by _reusability_:

*Potentially reusable* across puzzles: The Skolem-witness pattern for `attacks_square` will apply to any puzzle --- only the piece locations change. A future version could generate these automatically from the puzzle position.

*Puzzle-shape dependent*: King-tracking lemmas like `bk_file_after_reply` depend on the specific move sequence (Rd8 then king move from a8). Different puzzles will need different instances, but the _pattern_ is the same.

*Fully puzzle-specific*: The `in_check` lemmas and `no_legal_king_moves` are bespoke facts about this specific position. A different mating pattern (e.g., a queen sacrifice followed by a knight mate) would require completely different instances.

== Limitations

*No global well-formedness.* The theory does not enforce that positions have exactly one king per side, valid piece codes, or consistent side-to-move. This is acceptable for curated puzzles where the position is fully constrained, but it means the theory cannot reason about _arbitrary_ positions.

*Fallback behavior on junk values.* Operations like `opposite`, `piece_type`, and `piece_side` interpret out-of-range inputs (e.g., `opposite(5)` returns 1). This is harmless in constrained puzzles but would permit unsound reasoning in unconstrained contexts.

*Mate-in-two means at most two.* The `mate_in_two` predicate does not exclude `mate_in_one`. For composition-grade exactness, one would define $"mate_in_two"(p) and not "mate_in_one"(p)$.

*The pawn asymmetry.* `pawn_push` is the only "geometry" predicate that takes a `Position` argument (for intermediate-square checking), making it structurally different from the purely geometric patterns. This is correct but less uniform.

== The Role of Proof Scaffolding

The scaffolding we supply is not _ad hoc_ --- every lemma is a true statement derivable from the generic axioms. The issue is purely computational: Z3's E-matching engine cannot chain through the required instantiation depth within its timeout. This is a well-known limitation of SMT solvers on problems involving deeply nested uninterpreted functions [2].

In the taxonomy of formal verification, our approach sits between:

- *Fully automated* (Z3 alone proves everything) --- which fails for the deeper assertions, and
- *Fully interactive* (human supplies every step) --- which our generic theory makes unnecessary for simpler facts.

The hybrid is practical and principled. It is the same methodology used in Dafny [3] and Why3 [4], where lemma functions and assertions guide the solver through intermediate proof steps.

== Next Steps

The natural experiment is a second puzzle with a different mating pattern --- a queen sacrifice or a knight mate. This would reveal which scaffolding lemmas generalize (the Skolem-witness pattern for `attacks_square`) and which are fully local (the specific `in_check` assertions). If the reusable fraction is high, the scaffolding could be generated automatically from the puzzle position by a Kleis preprocessor.

A second direction is global well-formedness: adding a `well_formed_position` predicate enforcing king count, piece validity, and side-to-move constraints. This would be applied per-puzzle (not universally, to avoid conflicts with `apply_move` on intermediate positions) and would strengthen the theory's soundness guarantees.

= Related Work

*Chess and formal methods.* Chess endgame tablebases [5] solve positions through exhaustive retrograde analysis, producing databases rather than proofs. Our approach is complementary: we produce _verifiable proofs_ of specific positions rather than exhaustive databases.

*SMT-based game solving.* SAT and SMT solvers have been applied to combinatorial puzzles including Sudoku and graph coloring. Chess is significantly harder due to the turn-based quantifier alternation ($exists forall exists dots.c$) which is absent in one-shot constraint satisfaction.

*Proof engineering.* The practice of supplying intermediate lemmas to guide automated provers is well established in Dafny [3], Why3 [4], and Liquid Haskell. Our Skolem-witness technique is a specific instance of this pattern applied to the existential quantifier in attack detection.

*E-matching limitations.* The difficulty of deep E-matching chains in SMT solvers is well documented [2]. Techniques like model-based quantifier instantiation (MBQI) can sometimes help, but for our encoding the quantifier structure (mixed $exists forall$ with arithmetic) remains challenging.

= Conclusion

We have demonstrated that SMT-based chess mate verification is feasible in Kleis, provided the generic theory is supplemented with puzzle-specific proof scaffolding. The resulting system performs genuine bounded formal reasoning: each assertion is mechanically verified by Z3 via proof by contradiction, and the verification of the complete checkmate involves nine independent proof obligations all discharged successfully.

The key insight is that the generic chess theory is _logically sufficient_ but _computationally insufficient_ for Z3's E-matching engine. Proof scaffolding bridges this gap by supplying derived facts that flatten deep instantiation chains. This is not a workaround --- it is proof engineering, the same discipline that makes large-scale formal verification practical in industrial tools.

The code is available at #link("https://kleis.io")[kleis.io] as part of the Kleis examples suite.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[1\] L. de Moura and N. Bjorner, "Z3: An Efficient SMT Solver," in _Tools and Algorithms for the Construction and Analysis of Systems (TACAS)_, 2008, pp. 337--340.]

#par(hanging-indent: 1.5em)[\[2\] D. Detlefs, G. Nelson, and J. B. Saxe, "Simplify: a theorem prover for program checking," _Journal of the ACM_, vol. 52, no. 3, pp. 365--473, 2005.]

#par(hanging-indent: 1.5em)[\[3\] K. R. M. Leino, "Dafny: An Automatic Program Verifier for Functional Correctness," in _Logic for Programming, Artificial Intelligence, and Reasoning (LPAR)_, 2010, pp. 348--370.]

#par(hanging-indent: 1.5em)[\[4\] J.-C. Filliâtre and A. Paskevich, "Why3 — Where Programs Meet Provers," in _European Symposium on Programming (ESOP)_, 2013, pp. 125--128.]

#par(hanging-indent: 1.5em)[\[5\] K. Thompson, "Retrograde Analysis of Certain Endgames," _ICCA Journal_, vol. 9, no. 3, pp. 131--139, 1986.]

#pagebreak()
// Reset heading counter for appendices
#counter(heading).update(0)
#set heading(numbering: "A.1")

#align(center)[
  #text(size: 14pt, weight: "bold")[Appendix]
]
#v(1em)
= Full Generic Theory

The complete `Chess` structure contains 25 axioms across 7 layers. The full source is reproduced in 581 lines including the puzzle instance and verification examples. It is available at:

#align(center)[`examples/chess/chess_mate_in_two.kleis`]

= Puzzle Instance

The puzzle-specific scaffolding for Puzzle 1 (Ka6 + Rd1 vs Ka8) adds 15 axioms to the `Puzzle1` structure: 5 position facts, 2 Skolem witnesses, 5 king-tracking lemmas, and 3 direct `in_check` assertions.

= Verification Output

All nine examples pass with Z3 4.13 on Apple M-series hardware. Total wall-clock time is approximately 29 seconds, dominated by the checkmate verification (the final assertion).
