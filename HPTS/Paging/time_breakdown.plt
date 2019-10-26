#!/usr/bin/gnuplot
load '../bar-styles.gp'

set terminal postscript eps enhanced color solid font "Helvetica,14" fontscale 1.0 #size 4,2
set output 'CPU_breakdown_10.eps'

set border 3 front
set tics nomirror in scale 0.75
set xtics nomirror in scale 0.75,0.75 rotate by 315 autojustify

set ylabel "CPU Usage ( \% )"
set xlabel "Throughput (MOPS)"

set style data histogram
set style histogram rowstacked
set style fill solid border -1
set boxwidth 1

set yrange [0:110]
set ytics 0,5,100

set key samplen 2 spacing 0
set key width -8 vertical maxrows 1
set datafile separator ","
set title "Memory isolation with Paging for 10 tenants"

plot '10_tenants' using ($9/$10)*100:xtic(1) title "Context-Switch Overhead" ls 1, "" using ($8/$10)*100 title "Execution" ls 2, "" using (($10-$9-$8)/$10)*100 title "Polling" ls 3

