import Prim "mo:prim";

// A bridge to the Rust Big Map
// import Log "Log";
import BigMapMain "canister:bigmap";
import BigMapData0 "canister:bigmap_data_0";
import BigMapData1 "canister:bigmap_data_1";
import BigMapData2 "canister:bigmap_data_2";

module {
  public func initialize() : async () {
    Prim.debugPrint("Motoko BigMap initialize()");
    await BigMapMain.initialize();
    // await BigMapMain.add_data_buckets([])
  };

  public func used_bytes() : async Nat {
    await BigMapMain.total_used_bytes()
  };

  public func get(key : [Nat8]) : async ? [Nat8] {
    await BigMapMain.get(key)
  };

  public func put(key : [Nat8], value : [Nat8]) : async Bool {
    await BigMapMain.put(key, value)
  };
};
