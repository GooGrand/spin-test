use rayon::prelude::*;
use std::{thread, marker::Sync};

const THERSHOLD: usize = 3;
const K_THERSHOLD: u64 = 8;

// fn computation_splitter_rayon<D, T, R>(data: D, f: fn(T) -> R) -> Vec<R> 
// where 
//     T: Copy, 
//     D: IntoIterator + IndexedParallelIterator
// {
//     if data.len() > THERSHOLD {
//         data.par_iter();
//         vec![]
//     } else {
//         data.iter().map(|item| {
//             f(*item)
//         }).collect()
//     }
// }

fn computation_splitter_std<T, R>(data: Vec<T>, f: fn(T) -> R) -> Vec<R> 
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

fn computation_splitter_std_test<T, R>(data: Vec<T>, f: fn(T) -> R) -> (Vec<R>, bool) 
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
    let res = computation_splitter_std(data, compute);
    println!("{:?}",  res);
}


mod test {
    use super::*;

    #[test]
    fn check_computation_splitter_std() {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98];
        let res = computation_splitter_std(data, compute);
        assert_eq!(res, vec![0, 1, 7, 2, 8, 2, 88, 64, 47, 14]);
    }

    #[test]
    fn check_generics() {

        fn add_str(name: &str) -> String {
            format!("{}_prefix", name)
        }

        let data = vec!["George", "Falcon", "Something"];
        let res = computation_splitter_std(data, add_str);
        assert_eq!(res, vec!["George_prefix", "Falcon_prefix", "Something_prefix"]);
    }

    #[test]
    fn check_threads_spawn() {
        let data = vec![1, 2, 3, 4, 6, 12, 100, 75, 55, 98]; // more than 3 so it's threading
        let (_, res) = computation_splitter_std_test(data, compute);
        assert_eq!(res, true);

        let data = vec![1, 2,]; // less than 3 - one thread
        let (_, res) = computation_splitter_std_test(data, compute);
        assert_eq!(res, false);

        let data = vec![1, 2, 3]; // exact 3 should be calculated in one thread
        let (_, res) = computation_splitter_std_test(data, compute);
        assert_eq!(res, false);

        let data = vec![1, 2, 3, 4]; // over thershold
        let (_, res) = computation_splitter_std_test(data, compute);
        assert_eq!(res, true);
    }
}
