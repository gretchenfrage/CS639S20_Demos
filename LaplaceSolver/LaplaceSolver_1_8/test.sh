#!/usr/bin/env bash

rm *.pgm
rm -rf changes_false
rm -rf changes_true

echo ">>>> with changes"

rm changes.h
touch changes.h
echo '#define CHANGES true' > changes.h

mkdir changes_true
time cs39 run 1 8 > changes_true/out.txt || exit 1
mv *.pgm changes_true/


echo ">>>> without changes"

rm changes.h
touch changes.h
echo '#define CHANGES false' > changes.h

mkdir changes_false
time cs39 run 1 8 > changes_false/out.txt || exit 1
mv *.pgm changes_false/


echo ">>>> comparing"
diff changes_true changes_false