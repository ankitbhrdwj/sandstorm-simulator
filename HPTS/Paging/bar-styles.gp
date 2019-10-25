#!/usr/bin/gnuplot

set border 3 front
set tics nomirror in scale 0.75
set xtics nomirror in scale 0.75,0.75 autojustify

set style data histogram
set style histogram cluster gap 1
set style fill solid

set style line 1 linecolor rgb "#E41A1C" # Red
set style line 2 linecolor rgb "#377EB8" # Blue
set style line 3 linecolor rgb "#4D4D4D" # Grey
set style line 4 linecolor rgb "#55AE3A" # Green
