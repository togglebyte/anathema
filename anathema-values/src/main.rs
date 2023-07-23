use std::time::Instant;

use anathema_values::*;

const COUNT: usize = 30_000;

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
    for _ in 0..1 {
        run();
    }
}

fn run() {
    let mut bucket = Bucket::<Value>::with_capacity(COUNT);

    {
        let mut bucket_mut = bucket.write();

        // -----------------------------------------------------------------------------
        //   - Insert -
        // -----------------------------------------------------------------------------
        let data = (0..COUNT).map(|i: usize| (i, Value::from(i))).collect::<Vec<_>>();
        let now = Instant::now();
        bucket_mut.bulk_insert(data);
        eprintln!("Insert (bulk) {:?}", now.elapsed());

        let now = Instant::now();
        for i in 0..COUNT {
            bucket_mut.insert(i, Value::Num(i));
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
            bucket_mut.get(i);//.unwrap();
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
        match **bucket.get(ValueRef::new(i, 0)).unwrap() {
            Value::Num(n) => count += 1,
             _ => {}
        }
    }
    eprintln!("Fetch ref {:?} | {count}", now.elapsed());
}
