import BigMap from 'ic:canisters/bigmap';
import BigMapData from 'ic:idl/bigmap_data';

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export const encodeArrayBuffer = (file: ArrayBuffer): number[] => Array.from(new Uint8Array(file));
export const encodeJSON = (json: any): number[] => encode(JSON.stringify(json));
export const encodeStr = (str: string): number[] => Array.from(encoder.encode(str));
export const encode = (val: any): number[] => {
  switch (typeof val) {
    case 'string':
      return encodeStr(val);
    case 'object':
      return encodeJSON(val);
    default:
      throw Error('unhandled type for encoding');
  }
};

export const arrToStr = (arr: number[]) => decoder.decode(new Uint8Array(arr));
export function strToJson<T>(str: string): T {
  return JSON.parse(str) as T;
}
export function arrToJson<T extends object>(arr: number[]) {
  return strToJson<T>(arrToStr(arr));
}

export function decode<T extends object | string>(obj: number[], fallback?: T): T {
  let s = arrToStr(obj);
  try {
    return strToJson<T>(s);
  } catch (error) {
    console.error(error);
    if (!fallback) {
      throw Error('failure in strToJson, with no fallback provided');
    }
    return fallback;
  }
}


// const getBigMapActor = () => {
//   // @ts-ignore
//   const factory = makeActorFactory(BigMapData);
//   const actor = factory({ canister_ids });
//   return actor;
// };

// const getBigMapDataActor = (canisterId: string) => {
//   // @ts-ignore
//   const factory = makeActorFactory(BigMapData);
//   const actor = factory({ canisterId });
//   return actor;
// };

export async function bigMapGet(key: string): Promise<number[]> {
  const encodedKey = encode(key);

  console.time(`GET "${key}"`);
  const res = await BigMap.get(encodedKey);
  console.timeEnd(`GET "${key}"`);

  if (res) {
    return res && res[0];
  } else {
    console.error(`Error getting key "${key}"`);
    return [];
  }
}

export async function bigMapPut(key: string, encodedValue: number[]): Promise<number[]> {
  const encodedKey = encode(key);

  console.time(`PUT "${key}"`);
  const res = await BigMap.put(encodedKey, encodedValue);
  console.time(`PUT "${key}"`);

  if (res) {
    return res && res[0];
  } else {
    console.error(`Error putting key "${key}"`);
    return [];
  }
}

export async function getBigMapStatus(): Promise<string> {

  console.time("BigMap status");
  const res = await BigMap.status();
  console.timeEnd("BigMap status");

  if (res) {
    console.log("status", res);
    return res;
  } else {
    console.error("BigMap status get failed");
    return "ERROR retrieving";
  }
}
