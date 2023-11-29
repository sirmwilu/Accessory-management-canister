import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Accessory {
  'id' : bigint,
  'updated_at' : [] | [bigint],
  'name' : string,
  'description' : string,
  'created_at' : bigint,
  'category' : string,
  'is_available' : boolean,
  'price' : bigint,
}
export interface AccessoryPayload {
  'name' : string,
  'description' : string,
  'category' : string,
  'is_available' : boolean,
  'price' : bigint,
}
export type Error = { 'NotFound' : { 'msg' : string } };
export type Result = { 'Ok' : Accessory } |
  { 'Err' : Error };
export type Result_1 = { 'Ok' : bigint } |
  { 'Err' : Error };
export interface TransactionRecord {
  'transaction_type' : string,
  'change_type' : string,
  'timestamp' : bigint,
}
export interface _SERVICE {
  'add_accessory' : ActorMethod<[AccessoryPayload], [] | [Accessory]>,
  'bulk_update_accessories' : ActorMethod<
    [Array<[bigint, AccessoryPayload]>],
    Array<Result>
  >,
  'delete_accessory' : ActorMethod<[bigint], Result>,
  'get_accessories_by_category' : ActorMethod<[string], Array<Accessory>>,
  'get_accessory' : ActorMethod<[bigint], Result>,
  'get_accessory_price' : ActorMethod<[bigint], Result_1>,
  'get_accessory_transaction_history' : ActorMethod<
    [bigint],
    Array<TransactionRecord>
  >,
  'get_available_accessories' : ActorMethod<[], Array<Accessory>>,
  'mark_accessory_as_available' : ActorMethod<[bigint], Result>,
  'mark_accessory_as_unavailable' : ActorMethod<[bigint], Result>,
  'search_accessories' : ActorMethod<[string], Array<Accessory>>,
  'update_accessory' : ActorMethod<[bigint, AccessoryPayload], Result>,
}
