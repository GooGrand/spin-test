#![feature(test)]

use rayon::prelude::*;
use std::{thread, marker::Sync, rc::Rc};

const THERSHOLD: usize = 3;
const K_THERSHOLD: u64 = 8;

fn splitter_rayon_chunks<T, R>(data: Vec<T>, f: fn(T) -> R) -> Vec<R> 
where 
    T: Send,
    R: Clone + Send
{
    if data.len() > THERSHOLD {
        data.into_par_iter().chunks(THERSHOLD)
        .map(|chunk| {
            chunk.into_iter()
            .map(|item| f(item))
            .collect::<Vec<R>>()
        }).collect::<Vec<Vec<R>>>().concat()
    } else {
        data.into_iter().map(|item| {
            f(item)
        }).collect()
    }
}

fn splitter_std_chunks<T, R>(data: Vec<T>, f: fn(T) -> R) -> Vec<R> 
where 
    T: Copy + Send + Sync + 'static, 
    R: Clone + Send + 'static,
{
    if data.len() > THERSHOLD {
        data.chunks(THERSHOLD)
        .map(|chunk| {
            let chunk = chunk.to_owned();
            thread::spawn(move || -> Vec<R> {
                chunk.iter()
                .map(|item| f(*item))
                .collect::<Vec<R>>()
            })
        })
        .map(|handle| {
            handle.join().unwrap()
        }).collect::<Vec<Vec<R>>>()
        .concat()
    } else {
        data.into_iter().map(|item| {
            f(item)
        }).collect()
    }
}

fn splitter_rayon<T, R>(data: Vec<T>, f: fn(T) -> R) -> Vec<R> 
where 
    T: Send + Sync, 
    R: Send 
{
    if data.len() > THERSHOLD {
        data.into_par_iter().map(|item| {
            f(item)
        }).collect::<Vec<R>>()
    } else {
        data.into_iter().map(|item| {
            f(item)
        }).collect()
    }
}

fn splitter_std<T, R>(data: Vec<T>, f: fn(T) -> R) -> Vec<R> 
where 
    T: Sync + Send + 'static, 
    R: Send + 'static
{
    if data.len() > THERSHOLD {
        data.into_iter().map(|item| {
            thread::spawn(move || -> R {
                f(item)
            })
        }).map(|handle| {
            handle.join().unwrap()
        }).collect()
    } else {
        data.into_iter().map(|item| {
            f(item)
        }).collect()
    }
}

fn splitter_std_test<T, R>(data: Vec<T>, f: fn(T) -> R) -> (Vec<R>, bool) 
where 
    T: Sync + Send + 'static, 
    R: Send + 'static
{
    if data.len() > THERSHOLD {
        (data.into_iter().map(|item| {
            thread::spawn(move || -> R {
                f(item)
            })
        }).map(|handle| {
            handle.join().unwrap()
        }).collect(), true)
    } else {
        (data.into_iter().map(|item| {
            f(item)
        }).collect(), false)
    }
}

fn compute(item: u64) -> u64 {
    let mut i = 0 as u64;
    let mut item = item;

    while i < K_THERSHOLD {
        if item == 1 {
            break;
        }
        if item % 2 == 0 {
            item = item / 2
        } else {
            item = item * 3 + 1
        }
        i += 1;
    };
    if item == 1 { i } else { item }
}

fn main() {
    let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98];
    let res = splitter_std_chunks(data, compute);
    println!("{:?}",  res);
}


mod test {
    extern crate test;
    use super::*;
    use test::Bencher;

    #[test]
    fn check_splitter_std() {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98];
        let res = splitter_std(data, compute);
        assert_eq!(res, vec![0, 1, 7, 2, 8, 2, 88, 64, 47, 14]);
    }

    #[test]
    fn check_splitter_rayon() {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98];
        let res = splitter_rayon(data, compute);
        assert_eq!(res, vec![0, 1, 7, 2, 8, 2, 88, 64, 47, 14]);
    }

    #[test]
    fn check_generics() {

        fn add_str(name: &str) -> String {
            format!("{}_prefix", name)
        }

        let data = vec!["George", "Falcon", "Something"];
        let res = splitter_std(data, add_str);
        assert_eq!(res, vec!["George_prefix", "Falcon_prefix", "Something_prefix"]);
    }

    #[test]
    fn check_threads_spawn() {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98]; // more than 3 so it's threading
        let (_, res) = splitter_std_test(data, compute);
        assert_eq!(res, true);

        let data = vec![1, 2,]; // less than 3 - one thread
        let (_, res) = splitter_std_test(data, compute);
        assert_eq!(res, false);

        let data = vec![1, 2, 3]; // exact 3 should be calculated in one thread
        let (_, res) = splitter_std_test(data, compute);
        assert_eq!(res, false);

        let data = vec![1, 2, 3, 4]; // over thershold
        let (_, res) = splitter_std_test(data, compute);
        assert_eq!(res, true);
    }

    #[bench]
    fn bench_rayon(b: &mut Bencher) {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98]; // more than 3 so it's threading
        b.iter(|| splitter_rayon(data.clone(), compute));
    }

    #[bench]
    fn bench_rayon_chunks(b: &mut Bencher) {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98]; // more than 3 so it's threading
        b.iter(|| splitter_rayon_chunks(data.clone(), compute));
    }

    #[bench]
    fn bench_std(b: &mut Bencher) {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98]; // more than 3 so it's threading
        b.iter(|| splitter_std(data.clone(), compute));
    }

    #[bench]
    fn bench_std_chunks(b: &mut Bencher) {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98]; // more than 3 so it's threading
        b.iter(|| splitter_std_chunks(data.clone(), compute));
    }

}
