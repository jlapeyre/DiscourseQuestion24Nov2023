#!/bin/env sh

# -march=native seems slower here for some reason
#clang++ -Wall -O3 -march=native -o trnbias_gcc trnbias.cc

clang++ -Wall -O3 -o trnbias_gcc trnbias.cc
