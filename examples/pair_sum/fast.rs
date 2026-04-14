use std::io::{self, Read};

fn main() {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).unwrap();
    let mut it = s.split_whitespace();

    let n: usize = it.next().unwrap().parse().unwrap();
    let k: i64 = it.next().unwrap().parse().unwrap();
    let mut a: Vec<i64> = (0..n)
        .map(|_| it.next().unwrap().parse::<i64>().unwrap())
        .collect();

    // Two pointers on sorted input.
    a.sort_unstable();
    let mut i = 0usize;
    let mut j = n.saturating_sub(1);
    let mut ans: i64 = 0;

    while i < j {
        let sum = a[i] + a[j];
        if sum == k {
            if a[i] == a[j] {
                let m = (j - i + 1) as i64;
                ans += m * (m - 1) / 2;
                break;
            }
            let left_val = a[i];
            let right_val = a[j];
            let mut c1 = 0i64;
            let mut c2 = 0i64;
            while i <= j && a[i] == left_val {
                c1 += 1;
                i += 1;
            }
            while j >= i && a[j] == right_val {
                c2 += 1;
                j -= 1;
            }
            ans += c1 * c2;
        } else if sum < k {
            i += 1;
        } else {
            j -= 1;
        }
    }

    println!("{ans}");
}

