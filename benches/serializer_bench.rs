use criterion::{black_box, criterion_group, criterion_main, Criterion};
use turbine_rust::{BincodeSerializer, BorshSerializer, JsonSerializer, Person, Serializer};

fn bench_serialization(c: &mut Criterion) {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let mut group = c.benchmark_group("Serialization");

    group.bench_function("Borsh", |b| {
        let serializer = BorshSerializer;
        b.iter(|| serializer.to_bytes(black_box(&person)).unwrap())
    });

    group.bench_function("Bincode", |b| {
        let serializer = BincodeSerializer;
        b.iter(|| serializer.to_bytes(black_box(&person)).unwrap())
    });

    group.bench_function("JSON", |b| {
        let serializer = JsonSerializer;
        b.iter(|| serializer.to_bytes(black_box(&person)).unwrap())
    });

    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let borsh_bytes = BorshSerializer.to_bytes(&person).unwrap();
    let bincode_bytes = BincodeSerializer.to_bytes(&person).unwrap();
    let json_bytes = JsonSerializer.to_bytes(&person).unwrap();

    let mut group = c.benchmark_group("Deserialization");

    group.bench_function("Borsh", |b| {
        let serializer = BorshSerializer;
        b.iter(|| {
            let _: Person = serializer.from_bytes(black_box(&borsh_bytes)).unwrap();
        })
    });

    group.bench_function("Bincode", |b| {
        let serializer = BincodeSerializer;
        b.iter(|| {
            let _: Person = serializer.from_bytes(black_box(&bincode_bytes)).unwrap();
        })
    });

    group.bench_function("JSON", |b| {
        let serializer = JsonSerializer;
        b.iter(|| {
            let _: Person = serializer.from_bytes(black_box(&json_bytes)).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_serialization, bench_deserialization);
criterion_main!(benches);
