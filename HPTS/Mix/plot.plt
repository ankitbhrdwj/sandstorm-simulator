#!/usr/bin/gnuplot
load '../line-styles.gp'

set terminal postscript eps enhanced color solid font "Helvetica,14" fontscale 1.0 #size 4,2
set output 'comparison.eps'

set border 3 front
set tics nomirror in scale 0.75
set xtics nomirror in scale 0.75,0.75 #rotate by 315 autojustify

#set key left bottom

set ylabel "Median Latency ( {/Symbol m}s )"
set xlabel "Throughput (MOPS)"

set yrange [0:100]
set ytics 0,5,100

set title "32 Tenants on 32 cores, Service time dist - 99.99/0.1 - 1/500 us"
set grid back

set xrange [1000000:20000000]
set xtics("0" 0 , "1" 1e6, "2" 2e6, "3" 3e6, "4" 4e6, "5" 5e6, "6" 6e6, "7" 7e6 ,"8" 8e6, "9" 9e6, "10" 10e6, "11" 11e6, "12" 12e6, "13" 13e6, "14" 14e6, "15" 15e6, "16" 16e6, "17" 17e6, "18" 18e6, "19" 19e6, "20" 20e6, "21" 21e6, "22" 22e6, "23" 23e6, "24" 24e6, "25" 25e6, "26" 26e6, "27" 27e6, "28" 28e6, "29" 29e6, "30" 30e6, "31" 31e6, "32" 32e6)
plot 'Paging_NoPre' using 4:6 with linespoints title "Paging without Preemption" ls 1, \
'10_PRE_Paging' using 4:6 with linespoints title "Paging with Preemption" ls 2,\
'10_PRE_Mpk' using 4:6 with linespoints title "Mpk with Preemption" ls 3,\
'10_PRE_Vmfunc' using 4:6 with linespoints title "VMFunc with Preemption" ls 4
