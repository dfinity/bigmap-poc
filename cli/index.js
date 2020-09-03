#!/usr/bin/env node

const fetch = require("node-fetch");
const { TextDecoder, TextEncoder } = require("util");
const { Crypto } = require("node-webcrypto-ossl");
const fs = require("fs");
const path = require("path");
const { defaults, networks } = require("../dfx.json");
global.btoa = require('btoa')
const {
  generateKeyPair,
  HttpAgent,
  makeActorFactory,
  makeAuthTransform,
  makeNonceTransform,
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
const outputRoot = path.join(
  __dirname,
  '..',
  ".dfx",
  networkName
);

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
const DATA_CANISTER_IDS = [];
const DATA_CANISTER_ACTORS = new Map(); // A map of CanisterId => DataCanisterActor

const bigMapAddDataBuckets = async (dataCanisterNames) => {
  dataCanisterNames.forEach(can_name => {
    let can_id = getCanisterId(can_name);
    let can_actor = getBigMapDataActor(can_id);
    DATA_CANISTER_IDS.push(can_id);
    DATA_CANISTER_ACTORS.set(can_id, can_actor);
  });
  await bigMap.add_data_buckets(DATA_CANISTER_IDS);
}

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
  agent.addTransform(makeAuthTransform(keypair));

  return makeActorFactory(candid)({ canisterId, agent });
};

async function bigMapPut(encodedKey, encodedValue) {
  const key = arrToStr(encodedKey).substr(0, 100);
  let data_can_id = String(await bigMap.lookup_data_bucket_for_put(encodedKey));
  if (data_can_id == "") {
    console.error(`BigMap no Data Canister available to put key "${key}"`);
    return false;
  }
  let dataCan = await bigMapDataCanisterIdToActor(data_can_id);
  // console.time(`BigMap Data Canister put ${key}`);
  let res = dataCan.put(encodedKey, encodedValue);
  // console.timeEnd(`BigMap Data Canister put ${key}`);

  if (!res) {
    console.error(`BigMap Data Canister ${data_can_id}: Error putting key "${key}"`);
  }
  return res;
}

async function bigMapGet(encodedKey) {
  const key = arrToStr(encodedKey).substr(0, 100);
  let data_can_id = String(await bigMap.lookup_data_bucket_for_get(encodedKey));
  if (data_can_id == "") {
    console.error(`BigMap no Data Canister available to get key "${key}"`);
    return false;
  }
  let dataCan = await bigMapDataCanisterIdToActor(data_can_id);
  let res = bigMap.get(encodedKey);

  if (!res) {
    console.error(`BigMap Data Canister ${data_can_id}: Error getting key "${key}"`);
  }
  return res;
}

async function cliGet(key) {
  console.time(`BigMap Data Canister get ${key}`);
  let value = await bigMapGet(strToArr(key));
  console.timeEnd(`BigMap Data Canister get ${key}`);
  if (value === undefined) {
    console.log(`BigMap key ${key} does not exist`);
  } else {
    console.log(`BigMap key ${key} ==> value ${arrToStr(value)}`);
  }
  return value;
}

async function cliPut(key, value) {
  console.time(`BigMap Data Canister put ${key}`);
  let success = await bigMapPut(strToArr(key), strToArr(value));
  console.timeEnd(`BigMap Data Canister put ${key}`);
  if (success) {
    console.log(`Put ${key} succeeded`);
  } else {
    console.log(`Put ${key} failed ${success}`);
  }
}

// Helpers

module.exports = { getCanister, getCanisterId, bigMapPut, bigMapGet, bigMapAddDataBuckets, getBigMapDataActor };

const yargs = require("yargs");

const options = yargs
  .usage("Usage: $0 <cmd> [args]")
  .option("add-data-buckets", { describe: "Add data buckets to BigMap", type: "array", demandOption: false })
  .option("get", { describe: "Get key, start with @ to load from file", type: "string", demandOption: false })
  .option("put", { describe: "Put value for given key", type: "array", demandOption: false })
  .argv;

if (options.addDataBuckets) {
  let data_buckets = options.addDataBuckets;
  console.log(`Add data buckets ${data_buckets}`);
  bigMapAddDataBuckets(data_buckets);
}

if (options.get) {
  let key = options.get;
  if (key[0] == '@') {
    key = fs.readFileSync(key.substring(1));
  }
  cliGet(key);
}

if (options.put) {
  if (options.put.length == 2) {
    let key = options.put[0];
    let value = options.put[1];
    if (key[0] == '@') {
      key = fs.readFileSync(key.substring(1));
    }
    if (value[0] == '@') {
      value = fs.readFileSync(value.substring(1));
    }
    cliPut(key, value);
  } else {
    console.log("Put requires exactly two arguments: key value");
  }
}
