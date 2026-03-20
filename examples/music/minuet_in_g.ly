\version "2.24.0"

\header {
  title = "Minuet in G Major"
  composer = "Christian Petzold (attr. J.S. Bach)"
}

\score {
  \new PianoStaff <<
  \new Staff {
    \clef treble
    \key g \major
    \time 3/4
    d''4 g'8( a'8 b'8 c''8) |
    d''4 g'4 g'4 |
    e''4 c''8( d''8 e''8 fis''8) |
    g''2 g'4 |
    c''4\mp d''8( c''8 b'8 a'8) |
    b'4 c''8( b'8 a'8 g'8) |
    fis'4 g'8( a'8 b'8 g'8) |
    a'2. |
    d''4\mf g'8( a'8 b'8 c''8) |
    d''4 g'4 g'4 |
    e''4 c''8( d''8 e''8 fis''8) |
    g''2 g'4 |
    c''4 d''8( c''8 b'8 a'8) |
    b'4 c''8( b'8 a'8 g'8) |
    a'4 b'8( a'8 g'8 fis'8) |
    g'2.\fermata
  }
  \new Staff {
    \clef bass
    \key g \major
    \time 3/4
    g2 a4 |
    b4 c4 b4 |
    c2 b4 |
    a4 b4 a4 |
    a2 fis4 |
    g2 b4 |
    d4 e4 d4 |
    d2.\fermata |
    g2 a4 |
    b4 c4 b4 |
    c2 b4 |
    a4 b4 a4 |
    a2 fis4 |
    g2 b4 |
    c4 d4 d,4 |
    g,2.\fermata
  }
  >>
  \layout { indent = 0 }
  \midi { }
}

