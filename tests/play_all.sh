#!/bin/bash
# Runs the player-like bot over chapters, N at a time.
#   tests/play_all.sh desk zh "b612 king vanity" [maxSeconds] [parallel]
cd "$(dirname "$0")"
mode=${1:-desk}; lang=${2:-zh}; chs=${3:-"b612 king vanity bar businessman lamp geo earth"}; max=${4:-600}; par=${5:-2}
port=8810
for c in $chs; do
  port=$((port+1))
  EXTRA=${EXTRA:-slopes=1} PORT=$port timeout $((max+200)) node play_web.mjs $c $mode $lang $max 2>&1 | tail -1 &
  while [ $(jobs -r | wc -l) -ge $par ]; do sleep 2; done
done
wait
echo ALLDONE
