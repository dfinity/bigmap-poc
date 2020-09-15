#!/usr/bin/env node

const fs = require("fs");
const bigmap_fn = require('./bigmap_functions');

const strToArr = bigmap_fn.strToArr;
const arrToStr = bigmap_fn.arrToStr;

async function get(key) {
  console.time(`BigMap get ${key}`);
  let [value] = await bigmap_fn.bigMapGet(strToArr(key));
  console.timeEnd(`BigMap get ${key}`);
  if (value === undefined) {
    console.log(`BigMap key ${key} does not exist`);
  } else {
    console.log(`BigMap key ${key} ==> value ${arrToStr(value)}`);
  }
  return value;
}

async function getToFile(key, fileName) {
  console.time(`BigMap get ${key}`);
  let [value] = await bigmap_fn.bigMapGet(strToArr(key));
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
  let bytes_written = await bigmap_fn.bigMapPut(strToArr(key), strToArr(value));
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
  let bytes_written = await bigmap_fn.bigMapAppend(strToArr(key), strToArr(value));
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
  let bytes_deleted = await bigmap_fn.bigMapDelete(strToArr(key));
  console.timeEnd(`BigMap delete key ${key}`);

  if (bytes_deleted) {
    console.log(`BigMap delete key ${key} succeeded (${bytes_deleted} bytes)`);
  } else {
    console.log(`BigMap delete key ${key} failed`);
  }
  return bytes_deleted;
}

async function list(key_prefix) {
  let keys = await bigmap_fn.bigMapList(strToArr(key_prefix));
  let keys_str = keys.map(k => arrToStr(k));
  console.log(JSON.stringify(keys_str, null, 2));
  return keys_str;
}

async function setDataBucketWasmBinary(file_name_wasm_binary) {
  console.time(`BigMap set Data Canister wasm binary from file ${file_name_wasm_binary}`);
  let wasm_binary = fs.readFileSync(file_name_wasm_binary);
  let wasm_binary_array = Array.from(wasm_binary);
  await bigmap_fn.getBigMapActor().set_data_bucket_canister_wasm_binary(wasm_binary_array);
  console.timeEnd(`BigMap set Data Canister wasm binary from file ${file_name_wasm_binary}`);
}

async function setSearchWasmBinary(file_name_wasm_binary) {
  console.time(`BigMap set Search Canister wasm binary from file ${file_name_wasm_binary}`);
  let wasm_binary = fs.readFileSync(file_name_wasm_binary);
  let wasm_binary_array = Array.from(wasm_binary);
  await bigmap_fn.getBigMapActor().set_search_canister_wasm_binary(wasm_binary_array);
  console.timeEnd(`BigMap set Search Canister wasm binary from file ${file_name_wasm_binary}`);
}

async function maintenance() {
  console.time(`BigMap maintenance`);
  let res = (await bigmap_fn.getBigMapActor().maintenance());
  console.log(JSON.stringify(JSON.parse(res), null, 2));
  console.timeEnd(`BigMap maintenance`);
}

async function status() {
  let res = (await bigmap_fn.getBigMapActor().status());
  console.log(JSON.stringify(JSON.parse(res), null, 2));
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

async function put_and_fts_index(key, value) {
  console.time(`BigMap put_and_fts_index ${key}`);
  await bigmap_fn.getBigMapActor().put_and_fts_index(strToArr(key), value);
  console.timeEnd(`BigMap put_and_fts_index ${key}`);
}

async function remove_from_fts_index(key) {
  await bigmap_fn.getBigMapActor().remove_from_search_index(key);
}

async function search(search_query) {
  let results = await bigmap_fn.getBigMapActor().search(search_query.join(' '));
  let results_str = results.map(e => { return { key: arrToStr(e[0]), value: arrToStr(e[1]) } });
  console.log(JSON.stringify(results_str, null, 2));
  return results_str;
}

async function put_and_fts_index_file(filename) {
  let puts = [];
  let data = fs.readFileSync(filename, 'utf8').split(/\r?\n/);
  console.time(`BigMap put_and_fts_index_file ${filename}`);
  data.forEach(function (line) {
    if (!line) {
      return;
    };
    let key = line.split(/(\s)/, 1)[0].trim();
    let value = line.slice(key.length).trim();
    puts.push(bigmap_fn.getBigMapActor().put_and_fts_index(strToArr(key), value));
  });
  let result = Promise.all(puts);
  console.timeEnd(`BigMap put_and_fts_index_file ${filename}`);
  return result;
}

const getIndexActor = bigmap_fn.getBigMapActor;

module.exports = {
  get, getToFile, put, append, deleteKey, list, setDataBucketWasmBinary,
  setSearchWasmBinary, maintenance, status, callIndex, callData, getIndexActor,
  put_and_fts_index, put_and_fts_index_file, remove_from_fts_index, search,
  strToArr, arrToStr
};
