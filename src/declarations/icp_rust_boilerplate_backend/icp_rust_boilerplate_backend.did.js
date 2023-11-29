export const idlFactory = ({ IDL }) => {
  const AccessoryPayload = IDL.Record({
    'name' : IDL.Text,
    'description' : IDL.Text,
    'category' : IDL.Text,
    'is_available' : IDL.Bool,
    'price' : IDL.Nat64,
  });
  const Accessory = IDL.Record({
    'id' : IDL.Nat64,
    'updated_at' : IDL.Opt(IDL.Nat64),
    'name' : IDL.Text,
    'description' : IDL.Text,
    'created_at' : IDL.Nat64,
    'category' : IDL.Text,
    'is_available' : IDL.Bool,
    'price' : IDL.Nat64,
  });
  const Error = IDL.Variant({ 'NotFound' : IDL.Record({ 'msg' : IDL.Text }) });
  const Result = IDL.Variant({ 'Ok' : Accessory, 'Err' : Error });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : Error });
  const TransactionRecord = IDL.Record({
    'transaction_type' : IDL.Text,
    'change_type' : IDL.Text,
    'timestamp' : IDL.Nat64,
  });
  return IDL.Service({
    'add_accessory' : IDL.Func([AccessoryPayload], [IDL.Opt(Accessory)], []),
    'bulk_update_accessories' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Nat64, AccessoryPayload))],
        [IDL.Vec(Result)],
        [],
      ),
    'delete_accessory' : IDL.Func([IDL.Nat64], [Result], []),
    'get_accessories_by_category' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(Accessory)],
        ['query'],
      ),
    'get_accessory' : IDL.Func([IDL.Nat64], [Result], ['query']),
    'get_accessory_price' : IDL.Func([IDL.Nat64], [Result_1], ['query']),
    'get_accessory_transaction_history' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(TransactionRecord)],
        ['query'],
      ),
    'get_available_accessories' : IDL.Func([], [IDL.Vec(Accessory)], ['query']),
    'mark_accessory_as_available' : IDL.Func([IDL.Nat64], [Result], []),
    'mark_accessory_as_unavailable' : IDL.Func([IDL.Nat64], [Result], []),
    'search_accessories' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(Accessory)],
        ['query'],
      ),
    'update_accessory' : IDL.Func([IDL.Nat64, AccessoryPayload], [Result], []),
  });
};
export const init = ({ IDL }) => { return []; };
