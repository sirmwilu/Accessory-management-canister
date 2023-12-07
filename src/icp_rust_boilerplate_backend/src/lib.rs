// Import necessary external crates and modules
#[macro_use]
extern crate serde;

use candid::{Decode, Encode};
use ic_cdk::api::{time, caller};
use validator::Validate;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use std::borrow::Borrow;

// Define type aliases for better readability
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define the structure representing an accessory
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Accessory {
    id: u64,
    seller: String,
    name: String,
    description: String,
    category: String,
    price: u64,  // New field: price
    created_at: u64,
    updated_at: Option<u64>,
    is_available: bool,
    inventory_count: u64,
}

// Implement trait for serializing and deserializing the accessory
impl Storable for Accessory {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement trait for specifying storage characteristics of the accessory
impl BoundedStorable for Accessory {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
// Define the Review struct
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Review {
    id: u64,
    accessory_id: u64,
    user_id: u64,
    rating: u8,
    comment: String,
    created_at: u64,
}

// Implement the Storable trait for Review
impl Storable for Review {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the BoundedStorable trait for Review
impl BoundedStorable for Review {
    const MAX_SIZE: u32 = 1024; // Maximum size for the serialized data
    const IS_FIXED_SIZE: bool = false; // Data size is not fixed
}
// Define thread-local variables for managing memory, ID counter, and accessory storage
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
    static REVIEW_STORAGE: RefCell<StableBTreeMap<u64, Review, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))) // Using a new MemoryId for reviews
        )
    );
}

// Define a payload structure for adding or updating an accessory
#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct AccessoryPayload {
    #[validate(length(min = 1))]
    name: String,
    #[validate(length(min = 10))]
    description: String,
    category: String,
    price: u64,  // New field: price
    is_available: bool,
    inventory_count: u64,
}

// Define a structure for recording transaction history
#[derive(candid::CandidType, Deserialize, Serialize)]
struct TransactionRecord {
    timestamp: u64,
    change_type: String,
    transaction_type: String,
}
// Define a payload structure for adding or updating a review
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ReviewPayload {
    accessory_id: u64,
    user_id: u64,
    rating: u8,
    comment: String,
}
// Function to insert an accessory into the storage
fn do_insert_accessory(accessory: &Accessory) {
    ACCESSORY_STORAGE.with(|service| {
        service.borrow_mut().insert(accessory.id, accessory.clone());
    });
}

// Query function to get an accessory by ID
#[ic_cdk::query]
fn get_accessory(id: u64) -> Result<Accessory, Error> {
    match _get_accessory(&id) {
        Some(accessory) => Ok(accessory),
        None => Err(Error::NotFound {
            msg: format!("an accessory with id={} not found", id),
        }),
    }
}

// Query function to get accessories by category
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

// Query function to get available accessories
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

// Query function to search for accessories based on a query string
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

// Query function to get transaction history of an accessory by ID
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
// Update function to adjust the stock level for an accessory
// Update function to adjust the stock level for an accessory
#[ic_cdk::update]
fn update_inventory(id: u64, new_inventory_count: u64) -> Result<Accessory, Error> {
    ACCESSORY_STORAGE.with(|service| {
        let mut accessories = service.borrow_mut();
        if let Some(accessory) = accessories.get(&id) {
            // Clone the accessory to update it, since we can't modify it in place
            let mut updated_accessory = accessory.clone();
            updated_accessory.inventory_count = new_inventory_count;
            
            // Insert the updated accessory back into the map
            accessories.insert(id, updated_accessory.clone());
            Ok(updated_accessory)
        } else {
            Err(Error::NotFound {
                msg: format!("Accessory with id={} not found", id),
            })
        }
    })
}



// Query function to return accessories with low stock levels
#[ic_cdk::query]
fn check_inventory_levels(threshold: u64) -> Vec<Accessory> {
    ACCESSORY_STORAGE.with(|service| {
        service.borrow()
            .iter()
            .filter(|(_, accessory)| accessory.inventory_count <= threshold)
            .map(|(_, accessory)| accessory.clone())
            .collect()
    })
}
// Update function to add a new accessory
#[ic_cdk::update]
fn add_accessory(accessory_payload: AccessoryPayload) -> Result<Accessory, Error> {
    let is_payload_valid = _check_input(&accessory_payload);
    if is_payload_valid.is_err() {
        return Err(is_payload_valid.unwrap_err())
    }
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let accessory = Accessory {
        id,
        seller: caller().to_string(),
        name: accessory_payload.name,
        description: accessory_payload.description,
        category: accessory_payload.category,
        price: accessory_payload.price,
        created_at: time(),
        updated_at: None,
        is_available: accessory_payload.is_available,
        inventory_count: accessory_payload.inventory_count,
    };

    do_insert_accessory(&accessory);
    Ok(accessory)
}

// Update function to update an existing accessory
#[ic_cdk::update]
fn update_accessory(id: u64, payload: AccessoryPayload) -> Result<Accessory, Error> {

    match ACCESSORY_STORAGE.with(|service| service.borrow_mut().get(&id)) {
        Some(mut accessory) => {
            let can_update = _check_if_seller(&accessory);
            if can_update.is_err(){
                return Err(can_update.unwrap_err())
            }
            let is_payload_valid = _check_input(&payload);
            if is_payload_valid.is_err() {
                return Err(is_payload_valid.unwrap_err())
            }
            accessory.name = payload.name;
            accessory.description = payload.description;
            accessory.category = payload.category;
            accessory.price = payload.price;
            accessory.updated_at = Some(time());
            accessory.is_available = payload.is_available;
            do_insert_accessory(&accessory);
            Ok(accessory.clone())
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update an accessory with id={}. accessory not found", id),
        }),
    }
}

// Update function to toggle an accessory's availability
#[ic_cdk::update]
fn toggle_accessory_availability(id: u64) -> Result<Accessory, Error> {
    match ACCESSORY_STORAGE.with(|service| service.borrow_mut().get(&id)) {
        Some(mut accessory) => {
            let can_toggle = _check_if_seller(&accessory);
            if can_toggle.is_err(){
                return Err(can_toggle.unwrap_err())
            }
            accessory.is_available = !accessory.is_available;
            do_insert_accessory(&accessory);
            Ok(accessory.clone())
        }
        None => Err(Error::NotFound {
            msg: format!("an accessory with id={} not found", id),
        }),
    }
}


// Function to insert a review into the storage
fn do_insert_review(review: &Review) {
    REVIEW_STORAGE.with(|service| {
        service.borrow_mut().insert(review.id, review.clone());
    });
}

// Update function to add a new review
#[ic_cdk::update]
fn add_review(review_payload: ReviewPayload) -> Result<Review, Error> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let review = Review {
        id,
        accessory_id: review_payload.accessory_id,
        user_id: review_payload.user_id,
        rating: review_payload.rating,
        comment: review_payload.comment,
        created_at: time(),
    };

    do_insert_review(&review);
    Ok(review)
}
// Query function to get all reviews for a specific accessory
#[ic_cdk::query]
fn get_reviews(accessory_id: u64) -> Vec<Review> {
    REVIEW_STORAGE.with(|service| {
        service.borrow()
            .iter()
            .filter(|(_, review)| review.accessory_id == accessory_id)
            .map(|(_, review)| review.clone())
            .collect()
    })
}
// Update function to delete an accessory
#[ic_cdk::update]
fn delete_accessory(id: u64) -> Result<Accessory, Error> {
    let accessory = _get_accessory(&id);
    if accessory.is_none() {
        return Err(Error::NotFound { msg: format!("an accessory with id={} not found", id) })
    }
    let can_toggle = _check_if_seller(&accessory.unwrap());
    if can_toggle.is_err(){
        return Err(can_toggle.unwrap_err())
    }
    match ACCESSORY_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(accessory) => Ok(accessory)
        ,
        None => Err(Error::NotFound {
            msg: format!("couldn't delete an accessory with id={}. accessory not found.", id),
        }),
    }
}

// Update function to bulk update accessories
#[ic_cdk::update]
fn bulk_update_accessories(updates: Vec<(u64, AccessoryPayload)>) -> Vec<Result<Accessory, Error>> {
    let mut results = Vec::new();
    for (id, payload) in updates {
        let result = update_accessory(id, payload);
        results.push(result);
    }
    results
}

// Enum to represent possible error types
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    ValidationFailed {msg: String},
    AuthenticationFailed {msg: String}
}

// Internal function to get an accessory by ID
fn _get_accessory(id: &u64) -> Option<Accessory> {
    let accessory_storage = MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)));
    StableBTreeMap::<u64, Accessory, Memory>::init(accessory_storage)
        .borrow()
        .get(id)
}

// Query function to get the price of an accessory by ID
#[ic_cdk::query]
fn get_accessory_price(id: u64) -> Result<u64, Error> {
    match _get_accessory(&id) {
        Some(accessory) => Ok(accessory.price),
        None => Err(Error::NotFound {
            msg: format!("an accessory with id={} not found", id),
        }),
    }
}

// Helper function to check the input data of the payload
fn _check_input(payload: &AccessoryPayload) -> Result<(), Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error:: ValidationFailed{ msg: check_payload.err().unwrap().to_string()})
    }else{
        Ok(())
    }
}

// Helper function to check whether the caller is the seller of a accessory
fn _check_if_seller(accessory: &Accessory) -> Result<(), Error> {
    if accessory.seller.to_string() != caller().to_string(){
        return Err(Error:: AuthenticationFailed{ msg: format!("Caller={} isn't the seller of the accessory with id={}", caller(), accessory.id) })  
    }else{
        Ok(())
    }
}

// Export the canister interface definition
ic_cdk::export_candid!();
