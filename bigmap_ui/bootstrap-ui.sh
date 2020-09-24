#!/bin/bash

set -e
echo "Bootstraping BigMap..."

# Support bootstrapping from any folder
BASEDIR=$(cd "$(dirname "$0")"; pwd)

"$BASEDIR/../bootstrap.sh" "$@"

if [[ "$1" == "tungsten" ]]; then
  echo "Bootstraping on the Tungsten network"
  NETWORK="--network tungsten"
  if [[ -z "$DFX_CREDS_USER" || -z "$DFX_CREDS_PASS" || -z "$DFX_NETWORK" ]]; then
    echo "Please make sure you set the following environment variables:"
    echo "export DFX_CREDS_USER={username}"
    echo "export DFX_CREDS_PASS={password}"
    echo "export DFX_NETWORK=tungsten"
  fi
else
  echo "Bootstraping on the local instance"
fi

cd "$BASEDIR/.."
address=$(dfx config defaults.start.address | tr -d '"')
port=$(dfx config defaults.start.port)

cd "$BASEDIR"

mkdir -p .dfx
test -d ../.dfx/local && rsync -aP ../.dfx/local/ .dfx/local/
test -d ../.dfx/tungsten && rsync -aP ../.dfx/tungsten/ .dfx/tungsten/
test -f ../canister_ids.json && rsync -aP ../canister_ids.json ./

echo "Bootstraping BigMap UI..."

npm install

echo "dfx build $NETWORK"
dfx build $NETWORK
echo "dfx canister $NETWORK install bigmap_ui --mode=reinstall"
dfx canister $NETWORK install bigmap_ui --mode=reinstall


if [[ "$1" == "tungsten" ]]; then
  URL="http://$(dfx canister $NETWORK id bigmap_ui).ic0.app/"
else
  URL="http://127.0.0.1:$port/?canisterId=$(dfx canister id bigmap_ui)"
fi

echo "Opening $URL"

case $(uname) in
  Linux) xdg-open $URL || true;;
  *) open $URL || true;;
esac
