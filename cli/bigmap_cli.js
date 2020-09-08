#!/usr/bin/env node

const fs = require("fs");
const bigmap_fn = require('./bigmap_functions');

async function get(key) {
  console.time(`BigMap get ${key}`);
  let [value] = await bigmap_fn.bigMapGet(bigmap_fn.strToArr(key));
  console.timeEnd(`BigMap get ${key}`);
  if (value === undefined) {
    console.log(`BigMap key ${key} does not exist`);
  } else {
    console.log(`BigMap key ${key} ==> value ${bigmap_fn.arrToStr(value)}`);
  }
  return value;
}

async function getToFile(key, fileName) {
  console.time(`BigMap get ${key}`);
  let [value] = await bigmap_fn.bigMapGet(bigmap_fn.strToArr(key));
  console.timeEnd(`BigMap get ${key}`);
  if (value === undefined) {
    console.log(`BigMap key ${key} does not exist`);
  } else {
    fs.writeFileSync(fileName, new Uint8Array(value));
    console.log(`BigMap key ${key} ==> value written to file ${fileName}`);
  }
}

async function put(key, value) {
  console.time(`BigMap put ${key}`);
  let bytes_written = await bigmap_fn.bigMapPut(bigmap_fn.strToArr(key), bigmap_fn.strToArr(value));
  console.timeEnd(`BigMap put ${key}`);
  if (bytes_written) {
    console.log(`BigMap put ${key} succeeded (${bytes_written} bytes)`);
  } else {
    console.log(`BigMap put ${key} failed`);
  }
  return bytes_written;
}

async function append(key, value) {
  console.time(`BigMap append ${key}`);
  let bytes_written = await bigmap_fn.bigMapAppend(bigmap_fn.strToArr(key), bigmap_fn.strToArr(value));
  console.timeEnd(`BigMap append ${key}`);
  if (bytes_written) {
    console.log(`BigMap append ${key} succeeded (${bytes_written} bytes)`);
  } else {
    console.log(`BigMap append ${key} failed`);
  }
  return bytes_written;
}

async function deleteKey(key) {
  console.time(`BigMap delete key ${key}`);
  let bytes_deleted = await bigmap_fn.bigMapDelete(bigmap_fn.strToArr(key));
  console.timeEnd(`BigMap delete key ${key}`);

  if (bytes_deleted) {
    console.log(`BigMap delete key ${key} succeeded (${bytes_deleted} bytes)`);
  } else {
    console.log(`BigMap delete key ${key} failed`);
  }
  return bytes_deleted;
}

async function setDataBucketWasmBinary(file_name_wasm_binary) {
  console.time(`BigMap set Data Canister wasm binary from file ${file_name_wasm_binary}`);
  let wasm_binary = fs.readFileSync(file_name_wasm_binary);
  let wasm_binary_array = Array.from(wasm_binary);
  await bigmap_fn.getBigMapActor().set_data_bucket_canister_wasm_binary(wasm_binary_array);
  console.timeEnd(`BigMap set Data Canister wasm binary from file ${file_name_wasm_binary}`);
}

async function maintenance() {
  console.time(`BigMap maintenance`);
  let res = (await bigmap_fn.getBigMapActor().maintenance());
  console.log(JSON.parse(res));
  console.timeEnd(`BigMap maintenance`);
}

async function callIndex(functionName, ...args) {
  args = args[0];
  console.time(`BigMap Call Index Canister ${functionName}(${args})`);
  let res = (await bigmap_fn.getBigMapActor()[functionName](...args));
  console.log(res);
  console.timeEnd(`BigMap Call Index Canister ${functionName}(${args})`);
}

async function callData(canisterId, functionName, ...args) {
  args = args[0];
  console.time(`BigMap Call Data Canister ${functionName}(${args})`);
  let res = (await bigmap_fn.getBigMapDataActor(canisterId)[functionName](...args));
  console.log(res);
  console.timeEnd(`BigMap Call Data Canister ${functionName}(${args})`);
}

const getIndexActor = bigmap_fn.getBigMapActor;
const strToArr = bigmap_fn.strToArr;
const arrToStr = bigmap_fn.arrToStr;

module.exports = { get, getToFile, put, append, deleteKey, setDataBucketWasmBinary, maintenance, callIndex, callData, getIndexActor, strToArr, arrToStr };
