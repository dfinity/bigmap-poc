actor Counter {
    var cell : Nat = 0;

    public query func read() : async Nat {
        cell
    };

    public func increment() : async Nat {
        cell += 1;
        cell
    };

    public func decrement() : async Nat {
        cell -= 1;
        cell
    };

    public func write(n: Nat) : async () {
        cell := n;
    };
}
