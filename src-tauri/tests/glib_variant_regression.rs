#![cfg(target_os = "linux")]

use glib::variant::ToVariant;

#[test]
fn string_variant_iterators_preserve_values_in_both_directions() {
    let variant = ["primeiro", "meio", "último"].to_variant();
    assert_eq!(variant.array_iter_str().unwrap().collect::<Vec<_>>(), ["primeiro", "meio", "último"]);
    assert_eq!(variant.array_iter_str().unwrap().last(), Some("último"));
    assert_eq!(variant.array_iter_str().unwrap().nth(1), Some("meio"));
    assert_eq!(variant.array_iter_str().unwrap().nth_back(1), Some("meio"));
    let mut iter = variant.array_iter_str().unwrap();
    assert_eq!(iter.next_back(), Some("último"));
    assert_eq!(iter.next(), Some("primeiro"));
    assert_eq!(iter.next_back(), Some("meio"));
    assert_eq!(iter.next(), None);
}
