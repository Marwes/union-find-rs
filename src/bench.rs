extern crate test;

use std::io::{BufRead, BufReader};
use std::fs::File;
use ::{UnionFind, UfValue};
use self::test::Bencher;

lazy_static! {
    static ref TINY: Input = Input::from_file("etc/tinyUF.txt");
    static ref MEDIUM: Input = Input::from_file("etc/mediumUF.txt");
    static ref LARGE: Input = Input::from_file("etc/largeUF.txt");
}

struct Cache<T> {
    input: &'static Input,
    init: Option<T>,
    union1: Option<T>,
    union2: Option<T>,
    find1: Option<T>,
    find2: Option<T>
}

impl<T> Cache<T> {
    fn new(input: &'static Input) -> Cache<T> {
        Cache {
            input: input,
            init: None,
            union1: None,
            union2: None,
            find1: None,
            find2: None
        }
    }

    fn init<V>(&mut self) -> &T
        where T: UnionFind<V>, V: UfValue
    {
        if let None = self.init {
            self.init = Some(self.input.init());
        }
        self.init.as_ref().unwrap()
    }

    fn union1<V>(&mut self) -> &T
        where T: UnionFind<V> + Clone, V: UfValue
    {
        if let None = self.union1 {
            let mut uf = self.init().clone();
            self.input.union(&mut uf);
            self.union1 = Some(uf);
        }
        self.union1.as_ref().unwrap()
    }

    fn union2<V>(&mut self) -> &T
        where T: UnionFind<V> + Clone, V: UfValue
    {
        if let None = self.union2 {
            let mut uf = self.union1().clone();
            self.input.union(&mut uf);
            self.union2 = Some(uf);
        }
        self.union2.as_ref().unwrap()
    }

    fn find1<V>(&mut self) -> &T
        where T: UnionFind<V> + Clone, V: UfValue
    {
        if let None = self.find1 {
            let mut uf = self.union1().clone();
            self.input.find_all(&mut uf);
            self.find1 = Some(uf);
        }
        self.find1.as_ref().unwrap()
    }

    fn find2<V>(&mut self) -> &T
        where T: UnionFind<V> + Clone, V: UfValue
    {
        if let None = self.find2 {
            let mut uf = self.union1().clone();
            self.input.find_all(&mut uf);
            self.find2 = Some(uf);
        }
        self.find2.as_ref().unwrap()
    }
}

#[derive(Clone, Debug)]
struct Input {
    size: usize,
    conn: Vec<(usize, usize)>
}

impl Input {
    fn from_file(name: &str) -> Input {
        let mut reader = BufReader::new(File::open(name).unwrap());
        let mut buf = String::new();

        let _ = reader.read_line(&mut buf).unwrap();
        let size = buf.trim().parse::<usize>().unwrap();
        buf.clear();

        let mut conn = vec![];

        while reader.read_line(&mut buf).unwrap() > 0 {
            {
                let mut sp = buf.trim().split_whitespace();
                let a = sp.next().unwrap().parse::<usize>().unwrap();
                let b = sp.next().unwrap().parse::<usize>().unwrap();
                conn.push((a, b));
            }

            buf.clear();
        }

        Input { size: size, conn: conn }
    }

    fn init<T, V>(&self) -> T
        where T: UnionFind<V>, V: UfValue
    {
        T::new(self.size)
    }

    fn union<T, V>(&self, uf: &mut T)
        where T: UnionFind<V>, V: UfValue
    {
        for &(p, q) in &self.conn {
            uf.union(p, q);
        }
    }
    fn find_all<T, V>(&self, uf: &mut T)
        where T: UnionFind<V>, V: UfValue
    {
        for i in 0..uf.size() {
            let _ = uf.find(i);
        }
    }

    fn bench_union1<T, V>(&self, bencher: &mut Bencher, cache: &mut Cache<T>)
        where T: UnionFind<V> + Clone, V: UfValue
    {
        let uf = cache.init();
        bencher.bytes = self.conn.len() as u64;
        bencher.iter(|| {
            let mut uf = uf.clone();
            self.union(&mut uf);
            uf
        });
    }
    fn bench_union2<T, V>(&self, bencher: &mut Bencher, cache: &mut Cache<T>)
        where T: UnionFind<V> + Clone, V: UfValue
    {
        let uf = cache.union1();
        bencher.bytes = self.conn.len() as u64;
        bencher.iter(|| {
            let mut uf = uf.clone();
            self.union(&mut uf);
            uf
        });
    }
    fn bench_union3<T, V>(&self, bencher: &mut Bencher, cache: &mut Cache<T>)
        where T: UnionFind<V> + Clone, V: UfValue
    {
        let uf = cache.union2();
        bencher.bytes = self.conn.len() as u64;
        bencher.iter(|| {
            let mut uf = uf.clone();
            self.union(&mut uf);
            uf
        });
    }

    fn bench_find1<T, V>(&self, bencher: &mut Bencher, cache: &mut Cache<T>)
        where T: UnionFind<V> + Clone, V: UfValue
    {
        let uf = cache.union1();
        bencher.bytes = self.size as u64;
        bencher.iter(|| {
            let mut uf = uf.clone();
            self.find_all(&mut uf);
            uf
        });
    }
    fn bench_find2<T, V>(&self, bencher: &mut Bencher, cache: &mut Cache<T>)
        where T: UnionFind<V> + Clone, V: UfValue
    {
        let uf = cache.find1();
        bencher.bytes = self.size as u64;
        bencher.iter(|| {
            let mut uf = uf.clone();
            self.find_all(&mut uf);
            uf
        });
    }
    fn bench_find3<T, V>(&self, bencher: &mut Bencher, cache: &mut Cache<T>)
        where T: UnionFind<V> + Clone, V: UfValue
    {
        let uf = cache.find2();
        bencher.bytes = self.size as u64;
        bencher.iter(|| {
            let mut uf = uf.clone();
            self.find_all(&mut uf);
            uf
        });
    }
}

macro_rules! bench_fns_for_type_with_input {
    ($ty:ty, $input:path) => {
        use std::sync::Mutex;
        use ::bench::Cache;

        lazy_static!{
            static ref CACHE: Mutex<Cache<$ty>> = Mutex::new(Cache::new(&$input));
        }

        #[bench]
        fn union1(bencher: &mut ::bench::test::Bencher) {
            $input.bench_union1::<$ty, _>(bencher, &mut CACHE.lock().unwrap());
        }
        #[bench]
        fn union2(bencher: &mut ::bench::test::Bencher) {
            $input.bench_union2::<$ty, _>(bencher, &mut CACHE.lock().unwrap());
        }
        #[bench]
        fn union3(bencher: &mut ::bench::test::Bencher) {
            $input.bench_union3::<$ty, _>(bencher, &mut CACHE.lock().unwrap());
        }
        #[bench]
        fn find1(bencher: &mut ::bench::test::Bencher) {
            $input.bench_find1::<$ty, _>(bencher, &mut CACHE.lock().unwrap());
        }
        #[bench]
        fn find2(bencher: &mut ::bench::test::Bencher) {
            $input.bench_find2::<$ty, _>(bencher, &mut CACHE.lock().unwrap());
        }
        #[bench]
        fn find3(bencher: &mut ::bench::test::Bencher) {
            $input.bench_find3::<$ty, _>(bencher, &mut CACHE.lock().unwrap());
        }
    }
}

macro_rules! bench_fns_for_type {
    ($ty:ty) => {
        mod tiny { bench_fns_for_type_with_input!($ty, ::bench::TINY); }
        mod medium { bench_fns_for_type_with_input!($ty, ::bench::MEDIUM); }
        mod large { bench_fns_for_type_with_input!($ty, ::bench::LARGE); }
    }
}

mod quick_union { bench_fns_for_type!(::QuickUnionUf<::Size>); }
mod quick_find {
    // bench_fns_for_type!(::QuickFindUf<::Size>);
    mod tiny { bench_fns_for_type_with_input!(::QuickFindUf<::Size>, ::bench::TINY); }
    mod medium { bench_fns_for_type_with_input!(::QuickFindUf<::Size>, ::bench::MEDIUM); }
    // large is too large to execute
    // mod large { bench_fns_for_type_with_input!(::QuickFindUf<::Size>, ::bench::LARGE); }
}