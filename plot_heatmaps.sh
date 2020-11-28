#!/bin/bash

if (( $# < 1)); then 
	echo "Usage:"
	echo "Parameter 1: What to use as label for x and y axis"
	echo "Parameter 2: (Optional) What to use as label for cb axis"
	exit -1
fi

var=""
if (( $# > 1 )); then
	var="set cblabel '\$$2\$' norotate center offset -6.3,13.5"
fi

shopt -s nullglob
for file in *.mean
do
    BASENAME=$file
   # if [ ! -f "$BASENAME.gp" ]; then

        cat << EOF >> "$BASENAME.gp"
$var
set xlabel '$ $1 $'
set ylabel '$ $1 $'
set t epslatex 9 standalone color size 12cm, 11cm header "\\\usepackage{amsmath}\n\\\usepackage{grffile}"
set output "$BASENAME.tex"
set font ",9"
parsed=system('tail -n 1 $BASENAME | wc -w')
fin = parsed -0.5
frac = parsed/5.0
set xrange[-0.5:fin]
set yrange[-0.5:fin]
set xtics ('0.0' -0.5, '0.2' frac -.5, '0.4' 2*frac -.5, '0.6' 3*frac -.5, '0.8' 4*frac -.5, '1.0' 5*frac -.5)
set ytics ('0.0' -0.5, '0.2' frac -.5, '0.4' 2*frac -.5, '0.6' 3*frac -.5, '0.8' 4*frac -.5, '1.0' 5*frac -.5)
set format x '%.0frac'
set palette model HSV
set palette negative defined  ( 0 0 1 0, 2.8 0.4 0.6 0.8, 5.5 0.83 0 1)
set view map
set datafile missing "NaN"

splot "$BASENAME" matrix with image t ""
set output
system('latexmk $BASENAME.tex -pdf -f')

EOF

#    fi
    gnuplot "$BASENAME.gp"
done

for file in *.mean.xz
do
    BASENAME=$file
   # if [ ! -f "$BASENAME.gp" ]; then
        cat << EOF > "$BASENAME.gp"
$var
set xlabel '$ $1 $'
set ylabel '$ $1 $'
set t epslatex 9 standalone color size 12cm, 11cm header "\\\usepackage{amsmath}\n\\\usepackage{grffile}"
set output "$BASENAME.tex"
set font ",9"
parsed=system('xzcat $BASENAME |tail -n 1 | wc -w')
fin = parsed -0.5
frac = parsed/5.0
set xrange[-0.5:fin]
set yrange[-0.5:fin]
set xtics ('0.0' -0.5, '0.2' frac -.5, '0.4' 2*frac -.5, '0.6' 3*frac -.5, '0.8' 4*frac -.5, '1.0' 5*frac -.5)
set ytics ('0.0' -0.5, '0.2' frac -.5, '0.4' 2*frac -.5, '0.6' 3*frac -.5, '0.8' 4*frac -.5, '1.0' 5*frac -.5)
set format x '%.0frac'
set palette model HSV
set palette negative defined  ( 0 0 1 0, 2.8 0.4 0.6 0.8, 5.5 0.83 0 1)
set view map
set datafile missing "NaN"

splot "< xzcat $BASENAME" matrix with image t ""
set output
system('latexmk $BASENAME.tex -pdf -f')

EOF

#    fi
    gnuplot "$BASENAME.gp"
    echo $BASENAME
done
