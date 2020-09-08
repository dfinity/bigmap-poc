#!/bin/bash

set -e

./bootstrap.sh

echo 'Get key "abc" before adding it'
./bigmap-cli --get abc
echo 'Insert key "abc" ==> "def"'
./bigmap-cli --put abc def
echo 'Get key "abc" after adding it'
./bigmap-cli --get abc

echo "Test done!"
