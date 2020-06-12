use crate::dfn_core;
use candid::de::IDLDeserialize;
use candid::CandidType;
use serde::de::DeserializeOwned;

pub fn from_output<Output: CandidType>(payload: Output) -> Vec<u8> {
    candid::ser::IDLBuilder::new()
        .arg(&payload)
        .unwrap()
        .serialize_to_vec()
        .unwrap()
}

pub fn to_input<Input: DeserializeOwned>(inp: Vec<u8>) -> Input {
    let mut d = IDLDeserialize::new(&inp).unwrap();
    let v = d.get_value().unwrap();
    d.done().unwrap();
    v
}

fn extract_arg_1<A1: DeserializeOwned>(mut de: IDLDeserialize<'_>) -> ((A1,), IDLDeserialize<'_>) {
    let a1 = de.get_value().unwrap();
    ((a1,), de)
}

fn extract_arg_2<A1: DeserializeOwned, A2: DeserializeOwned>(
    de: IDLDeserialize<'_>,
) -> ((A1, A2), IDLDeserialize<'_>) {
    let ((a1,), mut de) = extract_arg_1(de);
    let a_new = de.get_value().unwrap();
    ((a1, a_new), de)
}

fn extract_arg_3<A1: DeserializeOwned, A2: DeserializeOwned, A3: DeserializeOwned>(
    de: IDLDeserialize<'_>,
) -> ((A1, A2, A3), IDLDeserialize<'_>) {
    let ((a1, a2), mut de) = extract_arg_2(de);
    let a_new = de.get_value().unwrap();
    ((a1, a2, a_new), de)
}

fn extract_arg_4<
    A1: DeserializeOwned,
    A2: DeserializeOwned,
    A3: DeserializeOwned,
    A4: DeserializeOwned,
>(
    de: IDLDeserialize<'_>,
) -> ((A1, A2, A3, A4), IDLDeserialize<'_>) {
    let ((a1, a2, a3), mut de) = extract_arg_3(de);
    let a_new = de.get_value().unwrap();
    ((a1, a2, a3, a_new), de)
}

pub trait OverCandid<Arguments, Output> {
    fn over_candid(self, _: IDLDeserialize<'_>) -> Vec<u8>;
}

impl<C: CandidType, F: FnOnce() -> C> OverCandid<(), C> for F {
    fn over_candid(self, _: IDLDeserialize<'_>) -> Vec<u8> {
        from_output(self())
    }
}

impl<A1: DeserializeOwned, C: CandidType, F: FnOnce(A1) -> C> OverCandid<(A1,), C> for F {
    fn over_candid(self, d: IDLDeserialize<'_>) -> Vec<u8> {
        let ((a1,), _) = extract_arg_1(d);
        from_output(self(a1))
    }
}

impl<A1: DeserializeOwned, A2: DeserializeOwned, C: CandidType, F: FnOnce(A1, A2) -> C>
    OverCandid<(A1, A2), C> for F
{
    fn over_candid(self, d: IDLDeserialize<'_>) -> Vec<u8> {
        let ((a1, a2), _) = extract_arg_2(d);
        from_output(self(a1, a2))
    }
}

impl<
        A1: DeserializeOwned,
        A2: DeserializeOwned,
        A3: DeserializeOwned,
        C: CandidType,
        F: FnOnce(A1, A2, A3) -> C,
    > OverCandid<(A1, A2, A3), C> for F
{
    fn over_candid(self, d: IDLDeserialize<'_>) -> Vec<u8> {
        let ((a1, a2, a3), _) = extract_arg_3(d);
        from_output(self(a1, a2, a3))
    }
}

impl<
        A1: DeserializeOwned,
        A2: DeserializeOwned,
        A3: DeserializeOwned,
        A4: DeserializeOwned,
        C: CandidType,
        F: FnOnce(A1, A2, A3, A4) -> C,
    > OverCandid<(A1, A2, A3, A4), C> for F
{
    fn over_candid(self, d: IDLDeserialize<'_>) -> Vec<u8> {
        let ((a1, a2, a3, a4), _) = extract_arg_4(d);
        from_output(self(a1, a2, a3, a4))
    }
}

/// This allows you to wrap regular rust functions and have them take and return
/// candid
/// Currently it will only return one argument
pub fn candid<A, O, F: OverCandid<A, O>>(f: F) {
    dfn_core::bytes(|bs| {
        let de = candid::de::IDLDeserialize::new(&bs).unwrap();
        f.over_candid(de)
    })
}

// TODO turn this into a trait with call_json, unless the type complexity is
// more trouble than it's worth
pub async fn call_candid<Input: DeserializeOwned, Output: CandidType>(
    canister_id: Vec<u8>,
    method_name: &str,
    payload: Output,
) -> Result<Input, (i32, String)> {
    let result = dfn_core::call(
        dfn_core::CanisterId(canister_id),
        method_name,
        &from_output(payload),
    )
    .await;
    result.map(to_input)
}
