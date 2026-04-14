use std::io::{self, Read};

fn main() {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).unwrap();
    let mut it = s.split_whitespace();

    let n: usize = it.next().unwrap().parse().unwrap();
    let k: i64 = it.next().unwrap().parse().unwrap();
    let a: Vec<i64> = (0..n)
        .map(|_| it.next().unwrap().parse::<i64>().unwrap())
        .collect();

    let mut ans: i64 = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            if a[i] + a[j] == k {
                ans += 1;
            }
        }
    }
    println!("{ans}");
}

