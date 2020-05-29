extern crate smallstring;

use smallstring::SmallString;
use std::str::FromStr;

#[test]
pub fn basic_construction() {
    let s = SmallString::<[u8; 0]>::default();
    assert_eq!(format!("{}", s), "");

    let s: SmallString = "Hello".into();
    assert_eq!(format!("{}", s), "Hello");

    // TODO: In the future it might be possible to infer this?
    let s = SmallString::<[u8; 5]>::from_str("Hello");
    assert_eq!(format!("{}", s), "Hello");
}

#[test]
pub fn can_be_sorted() {
    let mut unsorted_strings: Vec<SmallString<[u8; 5]>> = (0..5)
        .rev()
        .map(|x: u32| SmallString::<[u8; 5]>::from_str(x.to_string().as_str()))
        .collect();

    // Sort as SmallString...
    unsorted_strings.sort();

    // Map to Strings for easier comparison.
    // We might wanna introduce Eq for `[u8;N]` in the future.
    let sorted_strings: Vec<String> = unsorted_strings
        .iter()
        .map(|x: &SmallString<[u8; 5]>| String::from_str(x).unwrap())
        .collect();

    assert_eq!(vec!["0", "1", "2", "3", "4"], sorted_strings);
}
