use std::time::Instant;

use anathema_values::*;

const COUNT: usize = 30_000;

enum TestValue {
    Num(usize),
    S(String),
}

impl<'a> TryFrom<&'a TestValue> for &'a usize {
    type Error = ();

    fn try_from(lark: &'a TestValue) -> Result<Self, Self::Error> {
        match lark {
            TestValue::Num(n) => Ok(n),
            _ => Err(())
        }
    }
}

fn main() {
    for _ in 0..1 {
        run();
    }
}

fn run() {
    let mut bucket = Bucket::<TestValue>::with_capacity(COUNT);

    {
        let mut bucket_mut = bucket.write();
        // TODO: get::<u64>() should work on the writable one,
        //       probably on readable too
        //
        //       inserting a Map<K, V> where V: impl Into<Value2>

        // -----------------------------------------------------------------------------
        //   - Insert -
        // -----------------------------------------------------------------------------
        let data = (0..COUNT).map(|i| (i, TestValue::Num(i))).collect::<Vec<_>>();
        let now = Instant::now();
        bucket_mut.bulk_insert(data);
        eprintln!("Insert (bulk) {:?}", now.elapsed());

        let now = Instant::now();
        for i in 0..COUNT {
            bucket_mut.insert(i, TestValue::Num(i));
        }
        eprintln!("Insert (individual) {:?}", now.elapsed());

        // -----------------------------------------------------------------------------
        //   - Fetch mut -
        // -----------------------------------------------------------------------------
        let now = Instant::now();
        for i in 0..COUNT {
            bucket_mut.by_ref(ValueRef::new(i, 0)).unwrap();
        }
        eprintln!("Fetch mut by key {:?}", now.elapsed());

        let now = Instant::now();
        for i in 0..COUNT {
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
    for i in 0..COUNT {
        bucket.get(ValueRef::new(i, 0)).unwrap();
        count += 1;
    }
    eprintln!("Fetch ref {:?} | {count}", now.elapsed());
}
