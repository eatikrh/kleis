\version "2.24.0"

\header {
  title = "Sonata No. 14 'Moonlight'"
  composer = "Ludwig van Beethoven (Op. 27, No. 2)"
}

\score {
  \new PianoStaff <<
  \new Staff {
    \clef treble
    \key cis \minor
    \time 2/2
    <<
    { \voiceOne \tempo "Adagio sostenuto" s1 |
    \tempo "Adagio sostenuto" s1 |
    \tempo "Adagio sostenuto" s1 |
    \tempo "Adagio sostenuto" s1 |
    s2 r4 gis'8.\pp gis'16 |
    gis'2. gis'8. gis'16 |
    gis'2( a'2) |
    gis'2 fis'4 b'4 |
    e'4 r4 s2 |
    s2 r4 g'8. g'16 |
    g'2. g'8. g'16 |
    g'2. fis'4 |
    fis'2( g'4 e'4) |
    fis'2 fis'2 }
    \\
    { \voiceTwo \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } |
    \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } |
    \tuplet 3/2 { a8 cis'8 e'8 } \tuplet 3/2 { a8 cis'8 e'8 } \tuplet 3/2 { a8 d'8 fis'8 } \tuplet 3/2 { a8 d'8 fis'8 } |
    \tuplet 3/2 { gis8 bis8 fis'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 dis'8 } \tuplet 3/2 { fis8 bis8 dis'8 } |
    \tuplet 3/2 { e8 gis8 cis'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } |
    \tuplet 3/2 { gis8 dis'8 fis'8 } \tuplet 3/2 { gis8 dis'8 fis'8 } \tuplet 3/2 { gis8 dis'8 fis'8 } \tuplet 3/2 { gis8 dis'8 fis'8 } |
    \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { gis8 cis'8 e'8 } \tuplet 3/2 { a8 cis'8 fis'8 } \tuplet 3/2 { a8 cis'8 fis'8 } |
    \tuplet 3/2 { gis8 b8 e'8 } \tuplet 3/2 { gis8 b8 e'8 } \tuplet 3/2 { a8 b8 dis'8 } \tuplet 3/2 { a8 b8 dis'8 } |
    \tuplet 3/2 { gis8 b8 e'8 } \tuplet 3/2 { gis8 b8 e'8 } \tuplet 3/2 { gis8 b8 e'8 } \tuplet 3/2 { gis8 b8 e'8 } |
    \tuplet 3/2 { g8 b8 e'8 } \tuplet 3/2 { g8 b8 e'8 } \tuplet 3/2 { g8 b8 e'8 } \tuplet 3/2 { g8 b8 e'8 } |
    \tuplet 3/2 { g8 b8 f'8 } \tuplet 3/2 { g8 b8 f'8 } \tuplet 3/2 { g8 b8 f'8 } \tuplet 3/2 { g8 b8 f'8 } |
    \tuplet 3/2 { g8 c'8 e'8 } \tuplet 3/2 { g8 b8 e'8 } \tuplet 3/2 { g8 cis'8 e'8 } \tuplet 3/2 { fis8 cis'8 e'8 } |
    \tuplet 3/2 { fis8 b8 d'8 } \tuplet 3/2 { fis8 b8 d'8 } \tuplet 3/2 { g8 b8 cis'8 } \tuplet 3/2 { e8 b8 cis'8 } |
    \tuplet 3/2 { fis8 b8 d'8 } \tuplet 3/2 { fis8 b8 d'8 } \tuplet 3/2 { fis8 ais8 cis'8 } \tuplet 3/2 { fis8 ais8 cis'8 } }
    >>
  }
  \new Staff {
    \clef bass
    \key cis \minor
    \time 2/2
    <cis, cis>1 |
    <b,, b,>1 |
    <a,, a,>2 <fis,, fis,>2 |
    <gis,, gis,>2 <gis,, gis,>2 |
    <cis, gis, cis>1 |
    <bis,, gis, bis,>1 |
    <cis, cis>2 <fis,, fis,>2 |
    <b,, b,>2 <b,, b,>2 |
    <e, e>1 |
    <e, e>1 |
    <d, d>1 |
    <c, c>4 <b,, b,>4 <ais,, ais,>2 |
    <b,, b,>2 e,4 g,4 |
    fis,2 <fis,, fis,>2
  }
  >>
  \layout { indent = 0 }
  \midi { }
}

