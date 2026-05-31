# poly-store-rs 🦀

A production-quality, generic storage system in Rust demonstrating advanced trait patterns, multiple serialization formats, and cross-format data migration.

## 🚀 Features

- **Generic Serializer Trait**: Abstract interface for plugging in any serialization format.
- **Multiple Implementations**:
  - **Borsh**: Binary Object Representation Serializer for Hashing (Solana favorite).
  - **Bincode**: Compact binary format for internal Rust IPC/storage.
  - **JSON**: Human-readable standard for web interoperability.
- **Type-Safe Storage**: Uses `PhantomData<T>` to manage type tracking without overhead.
- **Data Migration**: Built-in support for converting stored data from one format to another (e.g., Borsh -> JSON).
- **Error Handling**: Idiomatic error propagation using `thiserror`.
- **Benchmarking**: Integrated Criterion suite to compare performance.

## 🛠 Architecture

### The `Serializer` Trait
The core abstraction that decouples the storage logic from the underlying data format.

```rust
pub trait Serializer<T> {
    fn to_bytes(&self, value: &T) -> Result<Vec<u8>, StorageError>;
    fn from_bytes(&self, bytes: &[u8]) -> Result<T, StorageError>;
}
```

### The `Storage<T, S>` Container
A generic container that handles the lifecycle of serialized data.

```rust
pub struct Storage<T, S> {
    data: Option<Vec<u8>>,
    serializer: S,
    _marker: PhantomData<T>,
}
```

#### Why `PhantomData`?
Since `Storage` only stores raw bytes (`Vec<u8>`) and not the actual instance of `T`, Rust needs a way to track the type `T` for variance and drop-checking. `PhantomData` acts as a zero-sized marker to satisfy the compiler while maintaining type safety.

## 📊 Benchmarking

Compare the speed and efficiency of different formats:

```bash
cargo bench
```

### Format Comparison

| Format | Speed | Size | Best For |
| :--- | :--- | :--- | :--- |
| **Borsh** | High | Minimal | Deterministic blockchain state (Solana). |
| **Bincode** | Extreme | Small | Internal Rust services and IPC. |
| **JSON** | Low | Large | Web APIs and human debugging. |

## 🧪 Testing

The project includes comprehensive unit tests covering storage operations, empty states, and cross-format conversions.

```bash
cargo test
```

## 🌐 Blockchain Context

This pattern is fundamental in blockchain development:
- **Solana**: Uses **Borsh** for almost all on-chain account data to ensure determinism.
- **Ethereum/EVM**: Uses **ABI encoding** or **RLP**.
- **Sui/Aptos**: Use **BCS** (Binary Canonical Serialization).

## 🎓 Educational Insights

### Common Mistakes
- **Unnecessary Cloning**: Beginners often pass data by value into serializers. This implementation uses references (`&T`) to minimize allocations.
- **Ignoring HRTBs**: Deserializing generic types often requires Higher-Rank Trait Bounds (`for<'de> Deserialize<'de>`) to handle lifetimes correctly.

### Interview Questions
1. How does `PhantomData` affect the memory layout of a struct? (Answer: It doesn't, it's zero-sized).
2. Why is determinism important in blockchain serialization? (Answer: To ensure different nodes reach the same state hash).
3. What are the trade-offs between text-based and binary serialization?

## 📜 License

MIT / Apache 2.0
