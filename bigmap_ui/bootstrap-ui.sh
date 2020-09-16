#!/bin/bash

set -e
echo "Bootstraping BigMap..."

# Support bootstrapping from any folder
BASEDIR=$(cd "$(dirname "$0")"; pwd)

"$BASEDIR/../bootstrap.sh"

cd "$BASEDIR/.."
address=$(dfx config defaults.start.address | tr -d '"')
port=$(dfx config defaults.start.port)

cd "$BASEDIR"

mkdir -p .dfx
test -d ../.dfx/local && rsync -aP ../.dfx/local/ .dfx/local/
test -d ../.dfx/tungsten && rsync -aP ../.dfx/tungsten/ .dfx/tungsten/

echo "Bootstraping BigMap UI..."

npm install

echo "dfx build"
dfx build
echo "dfx canister install bigmap_ui --mode=reinstall"
dfx canister install bigmap_ui --mode=reinstall


URL="http://127.0.0.1:$port/?canisterId=$(dfx canister id bigmap_ui)"

echo "Opening $URL"

case $(uname) in
  Linux) xdg-open $URL || true;;
  *) open $URL || true;;
esac
