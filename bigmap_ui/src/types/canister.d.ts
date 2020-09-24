declare module 'ic:idl/bigmap_data' {
  // type BigMapData = IDL.InterfaceFactory;
  // const BigMapData: BigMapData;
  // export default BigMapData;
}

declare module 'ic:canisters/bigmap' {
  interface BigMap {
    /**
     * Store a value in the Big Map
     * Both `key` and `value` must be [Word8] (arrays of utf8 charcodes)
     */
    put: (key: number[], value: number[]) => Promise<any>;
    /**
     * Fetch a value from the Big Map
     */
    get: (arr: number[]) => Promise<number[][]>;
    status: () => Promise<string>;
    search: (query: string) => Promise<[BigNumber, [[number[], number[]]]]>;
  }
  const BigMap: BigMap;
  export default BigMap;
}

declare module 'ic:canisters/bigmap_ui';


interface SearchResultItem {
  key: string;
  value: string;
}

interface SearchResults {
  entries_count: number;
  entries: SearchResultItem[];
}
