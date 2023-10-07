# Architecture Design Record

## ADR-14, KeyValue Store

### Flash Storage
Must IoT devices use flash type of storage from esp32, rpi, etc.
It is then important that a database or store must be designed for that type of storage. 
LSM algorithm are perfect for flash and RAM storage. 

- [RocksDb](https://github.com/facebook/rocksdb)
- [rs-RocksDb](https://github.com/rust-rocksdb/rust-rocksdb) with rust binding is then a good fit.

### Search for keys

RocksDb maintains a sorted list of keys. Then you can create Iterators that start from a specific key, 
prefix, beginning or end of storage.
You can also choose the direction of the Iterator (forward or backward).
The prefix will give you an iterator with the first hit and then the rest of the storage from that point.
You must stop the iterator using a prefix extractor and compare each key with a stop condition.

You can use prefix_iterator to have the stop condition included.

### Optimization

So many things. prefix_extractor, column families, etc.
Before drowning into optimization, we need some usage!

[seek](https://github.com/facebook/rocksdb/wiki/Prefix-Seek)

### Snippet

```rust
use rocksdb::{DB, Direction, IteratorMode, Error, SliceTransform, Options};

fn main() {
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);
    //db_opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(3));
    
    let db = DB::open(&db_opts, "./db").unwrap();
    db.put(b"hello", b"world").unwrap();
    db.put(b"1", b"1").unwrap();
    db.put(b"2", b"2").unwrap();
    db.put(b"3", b"3").unwrap();
    db.put(b"4", b"4").unwrap();
    db.put(b"5", b"5").unwrap();
    db.put(b"6", b"6").unwrap();

    db.put(b"cd-1", b"1").unwrap();
    db.put(b"cd-2", b"2").unwrap();
    db.put(b"cd-3", b"3").unwrap();
    db.put(b"cd-4", b"4").unwrap();
    db.put(b"cd-5", b"5").unwrap();
    db.put(b"cd-6", b"6").unwrap();

    db.put(b"dvd-1", b"1").unwrap();
    db.put(b"dvd-2", b"2").unwrap();
    db.put(b"dvd-3", b"3").unwrap();
    db.put(b"dvd-4", b"4").unwrap();
    db.put(b"dvd-5", b"5").unwrap();
    db.put(b"dvd-6", b"6").unwrap();
    match db.get(b"hello") {
        Ok(Some(v)) => {
            println!("{}", String::from_utf8(v).unwrap());
        }
        Ok(None) => {

        }
        Err(_) => {}
    }

    println!("=====");
    // world
    // 1-1
    // 2-2
    // 3-3
    // 4-4
    // 5-5
    // 6-6
    // cd-1-1
    // cd-2-2
    // cd-3-3
    // cd-4-4
    // cd-5-5
    // cd-6-6
    // dvd-1-1
    // dvd-2-2
    // dvd-3-3
    // dvd-4-4
    // dvd-5-5
    // dvd-6-6
    // hello-world
    let mut iter = db.iterator(IteratorMode::Start); // Always iterates forward
    for item in iter {
        print_item(item)
    }

    println!("=====");
    // cd-1-1
    // cd-2-2
    // cd-3-3
    // cd-4-4
    // cd-5-5
    // cd-6-6
    // dvd-1-1
    // dvd-2-2
    // dvd-3-3
    // dvd-4-4
    // dvd-5-5
    // dvd-6-6
    // hello-world
    iter = db.iterator(IteratorMode::From("cd".as_ref(), Direction::Forward)); // Always iterates forward
    for item in iter {
        print_item(item)
    }

    
    println!("=====");
    // dvd-1-1
    // dvd-2-2
    // dvd-3-3
    // dvd-4-4
    // dvd-5-5
    // dvd-6-6
    iter = db.prefix_iterator(b"dvd");
    for item in iter {
        print_item(item)
    }

}

fn print_item(item:  Result<(Box<[u8]>, Box<[u8]>), Error>) {
    match item {
        Ok(i) => {
            let (k, v) = i;
            println!("{}-{}", String::from_utf8(k.into()).unwrap(), String::from_utf8(v.into()).unwrap());
        }
        Err(e) => {
            println!("error {}", e)
        }
    }
}
```


```csharp
mir/
├─ list/
│  ├─ devices/      // can specify a device id
│     ├─ all        // not a command, its devices
│     ├─ meta     
│     ├─ desired
│     ├─ reported
├─ update/
│  ├─ device/       // must specify a device id
│     ├─ tag     
│     ├─ desired
├─ create/          // must specify a device id
│  ├─ device
├─ delete/          // must specify a device id
│  ├─ device
├─ stream/          // stream data. add filter based on queue type.
│  ├─ telemetry
├─ listen/          // listen to all queue at the same time. add filter based on queue type.
│  ├─ hearthbeat/
│  ├─ desired/
│  ├─ reported/
│  ├─ telemetry/
│  ├─ .../
```
