use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

/// --- Error Handling ---
/// We use `thiserror` for idiomatic, type-safe error handling.
/// This allows us to map various serialization errors to a common domain error.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Serialization failed: {0}")]
    SerializationError(String),

    #[error("Deserialization failed: {0}")]
    DeserializationError(String),

    #[error("Storage is empty: no data to load")]
    EmptyStorage,
}

/// --- Serializer Trait ---
/// A generic trait for serialization formats.
/// It is generic over `T`, the type being serialized.
pub trait Serializer<T> {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, StorageError>;
}

/// --- Borsh Implementation ---
/// Borsh is a binary format optimized for consistency and speed.
/// Commonly used in Solana development.
pub struct BorshSerializer;

impl<T> Serializer<T> for BorshSerializer
where
    T: BorshSerialize + BorshDeserialize,
{
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError> {
        borsh::to_vec(value).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, StorageError> {
        borsh::from_slice(bytes).map_err(|e| StorageError::DeserializationError(e.to_string()))
    }
}

/// --- Bincode Implementation ---
/// Bincode (referenced as Wincode in the prompt) is a compact binary format
/// widely used in the Rust ecosystem for internal IPC and storage.
pub struct BincodeSerializer;

impl<T> Serializer<T> for BincodeSerializer
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError> {
        bincode::serialize(value).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, StorageError> {
        bincode::deserialize(bytes).map_err(|e| StorageError::DeserializationError(e.to_string()))
    }
}

/// --- JSON Implementation ---
/// JSON is human-readable and standard for web APIs, though less space-efficient.
pub struct JsonSerializer;

impl<T> Serializer<T> for JsonSerializer
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(value).map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<T, StorageError> {
        serde_json::from_slice(bytes).map_err(|e| StorageError::DeserializationError(e.to_string()))
    }
}

/// --- Generic Storage Container ---
/// `Storage` handles the lifecycle of data persistence.
///
/// ### Why PhantomData?
/// `PhantomData<T>` is required because the struct is generic over `T`,
/// but `T` is not actually stored as a field (only its serialized bytes are).
/// Rust requires all generic parameters to be "used" to determine variance and drop check.
pub struct Storage<T, S> {
    data: Option<Vec<u8>>,
    serializer: S,
    _marker: PhantomData<T>,
}

impl<T, S> Storage<T, S>
where
    S: Serializer<T>,
{
    pub fn new(serializer: S) -> Self {
        Self {
            data: None,
            serializer,
            _marker: PhantomData,
        }
    }

    /// Serializes and stores the value.
    pub fn save(&mut self, value: &T) -> Result<(), StorageError> {
        let bytes = self.serializer.to_bytes(value)?;
        self.data = Some(bytes);
        Ok(())
    }

    /// Deserializes and returns the stored value.
    pub fn load(&self) -> Result<T, StorageError> {
        match &self.data {
            Some(bytes) => self.serializer.from_bytes(bytes),
            None => Err(StorageError::EmptyStorage),
        }
    }

    /// Checks if storage contains any data.
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    /// Bonus: Convert storage to a different serialization format.
    /// This consumes the current storage, deserializes the data using the old format,
    /// and re-serializes it using the new format.
    pub fn convert<NewS>(self, new_serializer: NewS) -> Result<Storage<T, NewS>, StorageError>
    where
        NewS: Serializer<T>,
    {
        let data = match self.data {
            Some(bytes) => {
                let value = self.serializer.from_bytes(&bytes)?;
                Some(new_serializer.to_bytes(&value)?)
            }
            None => None,
        };

        Ok(Storage {
            data,
            serializer: new_serializer,
            _marker: PhantomData,
        })
    }
}

/// --- Test Data Type ---
#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct Person {
    pub name: String,
    pub age: u32,
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_person() -> Person {
        Person {
            name: "Alice".to_string(),
            age: 30,
        }
    }

    #[test]
    fn test_borsh_storage() {
        let person = create_test_person();
        let mut storage = Storage::new(BorshSerializer);

        assert!(!storage.has_data());
        assert!(storage.load().is_err());

        storage.save(&person).expect("Save failed");
        assert!(storage.has_data());

        let loaded = storage.load().expect("Load failed");
        assert_eq!(person, loaded);
    }

    #[test]
    fn test_bincode_storage() {
        let person = create_test_person();
        let mut storage = Storage::new(BincodeSerializer);

        storage.save(&person).expect("Save failed");
        let loaded = storage.load().expect("Load failed");
        assert_eq!(person, loaded);
    }

    #[test]
    fn test_json_storage() {
        let person = create_test_person();
        let mut storage = Storage::new(JsonSerializer);

        storage.save(&person).expect("Save failed");
        let loaded = storage.load().expect("Load failed");
        assert_eq!(person, loaded);
    }

    #[test]
    fn test_empty_storage() {
        let storage: Storage<Person, JsonSerializer> = Storage::new(JsonSerializer);
        match storage.load() {
            Err(StorageError::EmptyStorage) => (),
            _ => panic!("Expected EmptyStorage error"),
        }
    }

    #[test]
    fn test_conversion() {
        let person = create_test_person();
        let mut borsh_storage = Storage::new(BorshSerializer);
        borsh_storage.save(&person).unwrap();

        // Convert Borsh storage to JSON storage
        let json_storage = borsh_storage.convert(JsonSerializer).expect("Conversion failed");
        
        // Verify data is still correct
        let loaded = json_storage.load().expect("Load from JSON failed");
        assert_eq!(person, loaded);

        // Verify it's actually JSON internally
        let json_bytes = json_storage.data.as_ref().unwrap();
        let json_str = std::str::from_utf8(json_bytes).unwrap();
        assert!(json_str.contains("Alice"));
    }
}
