use std::time::Instant;

use anathema_values::*;

enum Value {
    Num(usize),
    List(Vec<ValueRef<Self>>)
}

impl From<usize> for Value {
    fn from(num: usize) -> Self {
        Self::Num(num)
    }
}

fn main() {
    let mut bucket = Bucket::<Value>::with_capacity(10_000);

    {
        let mut bucket_mut = bucket.write();

        // -----------------------------------------------------------------------------
        //   - Insert -
        // -----------------------------------------------------------------------------
        let data = (0..10_000).map(|i: usize| (i, Value::from(i))).collect::<Vec<_>>();
        let now = Instant::now();
        bucket_mut.bulk_insert(data);
        // for i in 0..10_000 {
        //     bucket_mut.insert(i, Value::Num(i));
        // }
        eprintln!("Insert (bulk) {:?}", now.elapsed());

        let now = Instant::now();
        for i in 0..10_000 {
            bucket_mut.insert(i, Value::Num(i));
        }
        eprintln!("Insert (individual) {:?}", now.elapsed());

        // -----------------------------------------------------------------------------
        //   - Fetch mut -
        // -----------------------------------------------------------------------------
        let now = Instant::now();
        for i in 0..10_000 {
            bucket_mut.get_by_ref(ValueRef::new(i, 0)).unwrap();
        }
        eprintln!("Fetch mut by key {:?}", now.elapsed());

        let now = Instant::now();
        for i in 0..10_000 {
            bucket_mut.get(i).unwrap();
        }
        eprintln!("Fetch mut by path {:?}", now.elapsed());
    }

    // -----------------------------------------------------------------------------
    //   - Fetch -
    // -----------------------------------------------------------------------------
    let bucket = bucket.read();
    let mut count = 0usize;
    let now = Instant::now();
    for i in 0..10_000 {
        match **bucket.get(ValueRef::new(i, 0)).unwrap() {
            Value::Num(n) => count += 1,
             _ => {}
        }
    }
    eprintln!("Fetch ref {:?} | {count}", now.elapsed());
}
