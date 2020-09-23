#!/usr/bin/env node

const fetch = require("node-fetch");
const { TextDecoder, TextEncoder } = require("util");
const { Crypto } = require("node-webcrypto-ossl");
const fs = require("fs");
const path = require("path");
const execSync = require('child_process').execSync;
const topLevelPath = execSync('git rev-parse --show-toplevel', { encoding: 'utf8' }).trimEnd();
const { defaults, networks } = require(path.join(topLevelPath, "dfx.json"));
global.btoa = require('btoa')
const {
  generateKeyPair,
  HttpAgent,
  makeActorFactory,
  makeAuthTransform,
  makeNonceTransform,
  makeExpiryTransform,
  Principal,
} = require("@dfinity/agent");

global.TextDecoder = TextDecoder;
global.TextEncoder = TextEncoder;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

const strToArr = (str) => Array.from(encoder.encode(str));
const arrToStr = (arr) => decoder.decode(new Uint8Array(arr));

global.crypto = new Crypto();

const networkName = process.env["DFX_NETWORK"] || "local";
const DEFAULT_HOST = networkName === 'local' ? `http://${defaults.start.address}:${defaults.start.port}` : networks[networkName].providers[0];
const outputRoot = path.join(topLevelPath, ".dfx", networkName);

const credentials = { name: process.env['DFX_CREDS_USER'], password: process.env['DFX_CREDS_PASS'] };
// Exports
const getCanister = (
  canisterName,
  host = DEFAULT_HOST,
  keypair = generateKeyPair()
) => {
  const candid = eval(getCandid(canisterName));
  const canisterId = getCanisterId(canisterName);
  const principal = Principal.selfAuthenticating(keypair.publicKey);
  const config = { fetch, host, principal };
  if (credentials.name && credentials.password) {
    config.credentials = credentials;
  }
  const agent = new HttpAgent(config);
  agent.addTransform(makeNonceTransform());
  agent.addTransform(makeExpiryTransform(5 * 60 * 1000));
  agent.addTransform(makeAuthTransform(keypair));

  return makeActorFactory(candid)({ canisterId, agent });
};

const getCanisterPath = (canisterName) => {
  return path.join(
    outputRoot,
    "canisters",
    canisterName
  );
}

const getCandid = (canisterName) =>
  fs
    .readFileSync(`${getCanisterPath(canisterName)}/${canisterName}.did.js`)
    .toString()
    .replace("export default ", "");

const getCanisterId = (canisterName) => {
  const canisterIdsPath = networkName === 'local' ? outputRoot : '.';
  let manifest = JSON.parse(
    fs.readFileSync(path.resolve(canisterIdsPath, 'canister_ids.json'))
  );
  return manifest[canisterName][networkName];
};

// Big Map
const bigMap = getCanister("bigmap");
const DATA_CANISTER_ACTORS = new Map(); // A map of CanisterId => DataCanisterActor

const bigMapDataCanisterIdToActor = async (canisterId) => {
  let cacheLookup = DATA_CANISTER_ACTORS.get(canisterId);
  if (cacheLookup) {
    return cacheLookup;
  } else {
    let canisterActor = getBigMapDataActor(canisterId);
    DATA_CANISTER_ACTORS.set(canisterId, canisterActor);
    return canisterActor;
  }
}

const getBigMapActor = () => {
  return bigMap;
}

const getBigMapDataActor = (canisterId) => {
  const host = DEFAULT_HOST;
  const keypair = generateKeyPair();
  const canisterName = "bigmap_data";
  const candid = eval(getCandid(canisterName));
  const principal = Principal.selfAuthenticating(keypair.publicKey);
  const config = { fetch, host, principal };
  if (credentials.name && credentials.password) {
    config.credentials = credentials;
  }
  const agent = new HttpAgent(config);
  agent.addTransform(makeNonceTransform());
  agent.addTransform(makeExpiryTransform(5 * 60 * 1000));
  agent.addTransform(makeAuthTransform(keypair));

  return makeActorFactory(candid)({ canisterId, agent });
};

async function bigMapPut(encodedKey, encodedValue) {

  let res = bigMap.put(encodedKey, encodedValue);

  if (!res) {
    const key = arrToStr(encodedKey).substr(0, 100);
    console.error(`BigMap: Error putting key "${key}"`);
  }
  return res;
}

async function bigMapPutSync(encodedKey) {
  return await bigMapPut(encodedKey);
}

async function bigMapAppend(encodedKey, encodedValue) {

  let res = bigMap.append(encodedKey, encodedValue);

  if (!res) {
    const key = arrToStr(encodedKey).substr(0, 100);
    console.error(`BigMap: Error appending key "${key}"`);
  }
  return res;
}

async function bigMapDelete(encodedKey) {
  let res = bigMap.delete(encodedKey);

  if (!res) {
    const key = arrToStr(encodedKey).substr(0, 100);
    console.error(`BigMap: Error deleting key "${key}"`);
  }
  return res;
}

async function bigMapGet(encodedKey) {
  let res = bigMap.get(encodedKey);

  if (!res) {
    const key = arrToStr(encodedKey).substr(0, 100);
    console.error(`BigMap: Error getting key "${key}"`);
  }
  return res;
}

async function bigMapGetSync(encodedKey) {
  return await bigMapGet(encodedKey);
}

async function bigMapList(encodedKeyPrefix) {
  let res = bigMap.list(encodedKeyPrefix);

  if (!res) {
    const key = arrToStr(encodedKeyPrefix).substr(0, 100);
    console.error(`BigMap: Error listing with key_prefix "${key}"`);
  }
  return res;
}

async function bigMapInit() {
  let data_wasm = path.join(
    __dirname,
    '..',
    "/target/wasm32-unknown-unknown/release/bigmap_data.wasm"
  )
  let wasm_binary = fs.readFileSync(data_wasm);
  let wasm_binary_array = Array.from(wasm_binary);
  await bigMap.set_data_bucket_canister_wasm_binary(wasm_binary_array);
}

async function bigMapInitWithSearch() {
  let data_wasm = path.join(
    __dirname,
    '..',
    "/target/wasm32-unknown-unknown/release/bigmap_data.wasm"
  )
  let search_wasm = path.join(
    __dirname,
    '..',
    "/target/wasm32-unknown-unknown/release/bigmap_search.wasm"
  )
  let wasm_binary1 = fs.readFileSync(data_wasm);
  let wasm_binary_array1 = Array.from(wasm_binary1);
  await bigMap.set_data_bucket_canister_wasm_binary(wasm_binary_array1);
  let wasm_binary2 = fs.readFileSync(search_wasm);
  let wasm_binary_array2 = Array.from(wasm_binary2);
  await bigMap.set_search_canister_wasm_binary(wasm_binary_array2);
}

module.exports = {
  getCanister, getCanisterId, getBigMapActor, bigMapPut, bigMapPutSync,
  bigMapAppend, bigMapDelete, bigMapGet, bigMapGetSync, bigMapList,
  bigMapInit, bigMapInitWithSearch, getBigMapDataActor, strToArr, arrToStr
};
