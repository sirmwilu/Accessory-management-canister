#[macro_use]
extern crate serde;

use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use std::borrow::Borrow;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Accessory {
    id: u64,
    name: String,
    description: String,
    category: String,
    price: u64,  // New field: price
    created_at: u64,
    updated_at: Option<u64>,
    is_available: bool,
}

impl Storable for Accessory {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Accessory {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static ACCESSORY_STORAGE: RefCell<StableBTreeMap<u64, Accessory, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct AccessoryPayload {
    name: String,
    description: String,
    category: String,
    price: u64,  // New field: price
    is_available: bool,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct TransactionRecord {
    timestamp: u64,
    change_type: String,
    transaction_type: String,
}

fn do_insert_accessory(accessory: &Accessory) {
    ACCESSORY_STORAGE.with(|service| {
        service.borrow_mut().insert(accessory.id, accessory.clone());
    });
}

#[ic_cdk::query]
fn get_accessory(id: u64) -> Result<Accessory, Error> {
    match _get_accessory(&id) {
        Some(accessory) => Ok(accessory),
        None => Err(Error::NotFound {
            msg: format!("an accessory with id={} not found", id),
        }),
    }
}

#[ic_cdk::query]
fn get_accessories_by_category(category: String) -> Vec<Accessory> {
    ACCESSORY_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .filter(|(_, accessory)| accessory.category == category)
            .map(|(_, accessory)| accessory.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn get_available_accessories() -> Vec<Accessory> {
    ACCESSORY_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .filter(|(_, accessory)| accessory.is_available)
            .map(|(_, accessory)| accessory.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn search_accessories(query: String) -> Vec<Accessory> {
    ACCESSORY_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .filter(|(_, accessory)| accessory.name.contains(&query) || accessory.description.contains(&query))
            .map(|(_, accessory)| accessory.clone())
            .collect()
    })
}

#[ic_cdk::query]
fn get_accessory_transaction_history(id: u64) -> Vec<TransactionRecord> {
    match _get_accessory(&id) {
        Some(accessory) => {
            let mut history = Vec::new();
            if let Some(updated_at) = accessory.updated_at {
                history.push(TransactionRecord {
                    timestamp: updated_at,
                    change_type: "Update".to_string(),
                    transaction_type: "Update".to_string(),
                });
            }
            history.push(TransactionRecord {
                timestamp: accessory.created_at,
                change_type: "Creation".to_string(),
                transaction_type: "Creation".to_string(),
            });
            history
        }
        None => Vec::new(),
    }
}

#[ic_cdk::update]
fn add_accessory(accessory_payload: AccessoryPayload) -> Option<Accessory> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let accessory = Accessory {
        id,
        name: accessory_payload.name,
        description: accessory_payload.description,
        category: accessory_payload.category,
        price: accessory_payload.price,
        created_at: time(),
        updated_at: None,
        is_available: accessory_payload.is_available,
    };

    do_insert_accessory(&accessory);
    Some(accessory)
}

#[ic_cdk::update]
fn update_accessory(id: u64, payload: AccessoryPayload) -> Result<Accessory, Error> {
    match ACCESSORY_STORAGE.with(|service| service.borrow_mut().get(&id)) {
        Some(mut accessory) => {
            accessory.name = payload.name;
            accessory.description = payload.description;
            accessory.category = payload.category;
            accessory.price = payload.price;
            accessory.updated_at = Some(time());
            accessory.is_available = payload.is_available;

            // No need to call do_insert_accessory as the accessory is modified in place

            Ok(accessory.clone())
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update an accessory with id={}. accessory not found", id),
        }),
    }
}

#[ic_cdk::update]
fn mark_accessory_as_available(id: u64) -> Result<Accessory, Error> {
    match ACCESSORY_STORAGE.with(|service| service.borrow_mut().get(&id)) {
        Some(mut accessory) => {
            accessory.is_available = true;
            do_insert_accessory(&accessory);
            Ok(accessory.clone())
        }
        None => Err(Error::NotFound {
            msg: format!("an accessory with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn mark_accessory_as_unavailable(id: u64) -> Result<Accessory, Error> {
    if let Some(mut accessory) = ACCESSORY_STORAGE.with(|service| service.borrow_mut().get(&id)) {
        accessory.is_available = false;
        do_insert_accessory(&accessory);
        Ok(accessory.clone())
    } else {
        Err(Error::NotFound {
            msg: format!("an accessory with id={} not found", id),
        })
    }
}

#[ic_cdk::update]
fn delete_accessory(id: u64) -> Result<Accessory, Error> {
    match ACCESSORY_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(accessory) => Ok(accessory),
        None => Err(Error::NotFound {
            msg: format!("couldn't delete an accessory with id={}. accessory not found.", id),
        }),
    }
}

#[ic_cdk::update]
fn bulk_update_accessories(updates: Vec<(u64, AccessoryPayload)>) -> Vec<Result<Accessory, Error>> {
    let mut results = Vec::new();
    for (id, payload) in updates {
        let result = update_accessory(id, payload);
        results.push(result);
    }
    results
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

fn _get_accessory(id: &u64) -> Option<Accessory> {
    let accessory_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)));
    StableBTreeMap::<u64, Accessory, Memory>::init(accessory_storage)
        .borrow()
        .get(id)
}
#[ic_cdk::query]
fn get_accessory_price(id: u64) -> Result<u64, Error> {
    match _get_accessory(&id) {
        Some(accessory) => Ok(accessory.price),
        None => Err(Error::NotFound {
            msg: format!("an accessory with id={} not found", id),
        }),
    }
}

ic_cdk::export_candid!();