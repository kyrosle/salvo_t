use std::{
    cmp::{max, min},
    ptr::eq,
};
#[derive(Debug)]
struct Message {
    diff_pos: usize,
    status: bool,
    l_pos_diff: String,
    r_pos_diff: String,
}

fn diff(left: &mut String, right: &mut String) -> Message {
    let left_c = left.clone();
    let right_c = right.clone();
    let iter = left_c.chars().zip(right_c.chars()).into_iter().enumerate();
    for (pos, (l, r)) in iter {
        if l.eq(&r) {
            continue;
        } else {
            println!("pos: {} left : {:?} || right : {:?}", pos, l, r);
            left.insert(pos - 1, '!');
            right.insert(pos - 1, '!');

            return Message {
                diff_pos: pos,
                status: false,
                l_pos_diff: left[max(0, pos - 20)..min(pos + 20, left.len())].to_string(),
                r_pos_diff: right[max(0, pos - 20)..min(pos + 20, right.len())].to_string(),
            };
        }
    }
    Message {
        diff_pos: left.len(),
        status: true,
        l_pos_diff: "".to_string(),
        r_pos_diff: "".to_string(),
    }
}

fn main() {
    let file = include_str!("../file.txt").to_string();
    let matches = file.split('\n').collect::<Vec<_>>();
    let (left, right) = (matches[0].to_owned(), matches[1].to_owned());
    let mut left = left.trim().strip_prefix("left: ").unwrap().to_string();
    let mut right = right.trim().strip_prefix("right: ").unwrap().to_string();
    println!("{:#?}", diff(&mut left, &mut right));
}
