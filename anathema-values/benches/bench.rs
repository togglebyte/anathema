#![feature(test)]
extern crate test;
use test::bench::{Bencher, black_box};
use anathema_values::*;

const COUNT: usize = 10_000;

#[derive(Clone)]
enum Value {
    Num(usize),
    List(Vec<ValueRef<Self>>)
}

impl From<usize> for Value {
    fn from(num: usize) -> Self {
        Self::Num(num)
    }
}

fn loaded_bucket() -> Bucket<Value> {
    let mut bucket = Bucket::<Value>::with_capacity(COUNT);
    let data = (0..COUNT).map(|i: usize| (i, Value::from(i))).collect::<Vec<_>>();
    {
    let mut bucket_mut = bucket.write();
    bucket_mut.bulk_insert(data);
    }
    bucket
}

#[bench]
fn mut_bucket_insert_bulk(b: &mut Bencher) {
    let mut bucket = black_box(Bucket::<Value>::with_capacity(COUNT));
    b.iter(|| {
        let data = (0..COUNT).map(|i: usize| (i, Value::from(i))).collect::<Vec<_>>();
        let mut bucket_mut = bucket.write();
        bucket_mut.bulk_insert(data);
    });
}

#[bench]
fn mut_bucket_insert_individual(b: &mut Bencher) {
    let mut bucket = black_box(Bucket::<Value>::with_capacity(COUNT));
    b.iter(|| {
        let mut bucket_mut = bucket.write();
        for i in 0..COUNT {
            bucket_mut.insert(i, Value::Num(i));
        }
    });
}

#[bench]
fn mut_bucket_fetch_by_value_ref(b: &mut Bencher) {
    let mut bucket = loaded_bucket();

    b.iter(|| {
        let mut bucket_mut = bucket.write();
        for i in 0..COUNT {
            bucket_mut.by_ref(ValueRef::new(i, 0)).unwrap();
        }
    });
}

#[bench]
fn mut_bucket_fetch_by_path(b: &mut Bencher) {
    let mut bucket = loaded_bucket();

    b.iter(|| {
        let mut bucket_mut = bucket.write();
        for i in 0..COUNT {
            bucket_mut.get(i).unwrap();
        }
    });
}


#[bench]
fn bucket_fetch_by_value_ref(b: &mut Bencher) {
    let mut bucket = loaded_bucket();

    b.iter(|| {
        let bucket = (&bucket).read();
        for i in 0..COUNT {
            bucket.get(ValueRef::new(i, 0)).unwrap();
        }
    });
}
