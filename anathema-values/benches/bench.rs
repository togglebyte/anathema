use anathema_values::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const COUNT: usize = 10_000;

#[derive(Clone)]
enum Value {
    Num(usize),
    List(Vec<ValueRef<Self>>),
}

impl From<usize> for Value {
    fn from(num: usize) -> Self {
        Self::Num(num)
    }
}

fn loaded_bucket() -> Store<Value> {
    let mut bucket = Store::<Value>::with_capacity(COUNT);
    let data = (0..COUNT)
        .map(|i: usize| (i, Value::from(i)))
        .collect::<Vec<_>>();
    {
        let mut bucket_mut = bucket.write();
        for (key, value) in data {
            bucket_mut.insert_at_path(key, value);
        }
    }
    bucket
}

fn mut_bucket_insert_individual(c: &mut Criterion) {
    let mut bucket = black_box(Store::<Value>::with_capacity(COUNT));
    c.bench_function("mut bucket: insert individual", |b| {
        b.iter(|| {
            let mut bucket_mut = bucket.write();
            for i in 0..COUNT {
                bucket_mut.insert_at_path(i, Value::Num(i));
            }
        });
    });
}

fn mut_bucket_fetch_by_value_ref(c: &mut Criterion) {
    let mut bucket = loaded_bucket();

    c.bench_function("mut bucket: fetch by value", |b| {
        b.iter(|| {
            let mut bucket_mut = bucket.write();
            for i in 0..COUNT {
                bucket_mut.by_ref(ValueRef::new(i, 0)).unwrap();
            }
        });
    });
}

fn mut_bucket_fetch_by_path(c: &mut Criterion) {
    let mut bucket = loaded_bucket();

    c.bench_function("mut bucket: fetch by path", |b| {
        b.iter(|| {
            let mut bucket_mut = bucket.write();
            for i in 0..COUNT {
                bucket_mut.get(i).unwrap();
            }
        });
    });
}

fn bucket_fetch_by_value_ref(c: &mut Criterion) {
    let mut bucket = loaded_bucket();

    c.bench_function("bucket: fetch by value ref", |b| {
        b.iter(|| {
            let bucket = (&bucket).read();
            for i in 0..COUNT {
                bucket.get(ValueRef::new(i, 0)).unwrap();
            }
        });
    });
}

criterion_group!(
    benches,
    mut_bucket_insert_individual,
    mut_bucket_fetch_by_value_ref,
    mut_bucket_fetch_by_path,
    bucket_fetch_by_value_ref,
);
criterion_main!(benches);
