#!/bin/bash

PREFIX="$(pwd)"
INTENSITY=0.1
INNER_ANGLE_FACTOR=1.0
P3_PROFILE="/System/Library/ColorSync/Profiles/Display P3.icc"
TEMPERATURES="1500 2500 3000 4500 5500 8000"
TEMPLATE="$(mktemp -d)/description.json"

lines=$(system_profiler SPDisplaysDataType | grep 'Resolution' | awk '{print $2 "x" $4}')
mkdir -p "${PREFIX}/heic"
for line in $lines; do
  width=$(echo ${line} | cut -dx -f1)
  height=$(echo ${line} | cut -dx -f2)
  for temp in ${TEMPERATURES}; do
    stem="${width}x${height}_${temp}K"
    # png="${PREFIX}/png/${stem}.png"
    heic="${PREFIX}/heic/${stem}.heic"
    tiff="${PREFIX}/tiff/${stem}.tiff"

    # mkdir -p "$(dirname "${png}")"
    mkdir -p "$(dirname "${tiff}")"

    lighting ${tiff} \
      --intensity ${INTENSITY} \
      --inner-angle-factor ${INNER_ANGLE_FACTOR} \
      --temperature ${temp} \
      --dithering 0.002 \
      --width ${width} \
      --height ${height} &
  done
  wait
  sed "s%{{prefix}}%${PREFIX}%; s%{{width}}%${width}%; s%{{height}}%${height}%" \
    description.json.mustache > "${TEMPLATE}"
  wallpapper -i "${TEMPLATE}" -o "${PREFIX}/heic/${width}x${height}.heic"
done
