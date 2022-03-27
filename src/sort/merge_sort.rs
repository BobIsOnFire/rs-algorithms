use alloc::vec::Vec;
use core::mem::MaybeUninit;

fn do_merge<T>(
    slice: &mut [MaybeUninit<T>],
    scratch: &mut [MaybeUninit<T>],
    is_less: &impl Fn(&T, &T) -> bool,
) {
    debug_assert_eq!(slice.len(), scratch.len());

    {
        let (mut left, mut right) = slice.split_at_mut(slice.len() / 2);

        for elem in scratch.iter_mut() {
            let use_left = match (left.first(), right.first()) {
                (Some(l), Some(r)) => unsafe { is_less(l.assume_init_ref(), r.assume_init_ref()) },
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (None, None) => unreachable!(
                    "Source slices are exhausted, while target is not yet fully initialized"
                ),
            };

            // match expr above does not allow me to return mut references to slices... well ok
            let source = if use_left { &mut left } else { &mut right };
            elem.write(unsafe { source.take_first_mut().unwrap().assume_init_read() });
        }
    }

    slice.iter_mut().zip(scratch.iter()).for_each(|(s, a)| {
        s.write(unsafe { a.assume_init_read() });
    });
}

fn do_merge_sort<T>(
    slice: &mut [MaybeUninit<T>],
    scratch: &mut [MaybeUninit<T>],
    is_less: &impl Fn(&T, &T) -> bool,
) {
    if slice.len() > 1 {
        let mid = slice.len() / 2;
        do_merge_sort(&mut slice[..mid], &mut scratch[..mid], is_less);
        do_merge_sort(&mut slice[mid..], &mut scratch[mid..], is_less);
        do_merge(slice, scratch, is_less);
    }
}

pub fn merge_sort_by<T>(slice: &mut [T], is_less: impl Fn(&T, &T) -> bool) {
    // Allow ourselves to move elements out of `slice` (because merge-sort needs auxiliary memory for merge operation).
    // SAFETY: all `slice` elements are initialized and safe to read from, by now.
    let slice: &mut [MaybeUninit<T>] = unsafe { core::mem::transmute(slice) };

    // Auxiliary memory required for merge operation (eugh, allocation!)
    // `vec![MaybeUninit::uninit(); slice.len()]` does not work - it requires `T: Clone`
    // SAFETY: `scratch` elements are uninitialized and unsafe to read from.
    let mut scratch = (0..slice.len())
        .map(|_| MaybeUninit::uninit())
        .collect::<Vec<_>>();

    do_merge_sort(slice, &mut scratch, &is_less);
}

pub fn merge_sort<T: PartialOrd>(slice: &mut [T]) {
    merge_sort_by(slice, <T as PartialOrd>::le)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_trait() {
        let mut arr = [1, 5, 2, 3, 7];
        merge_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 5, 7]);
    }

    #[test]
    fn empty_trait() {
        let mut arr: [i32; 0] = [];
        merge_sort(&mut arr);
        assert_eq!(arr, []);
    }

    #[test]
    fn stability() {
        // Assertion does not work otherwise
        #[derive(Debug, PartialEq)]
        struct Pair(i32, i32);

        // Ordering only for first element
        impl PartialOrd for Pair {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        let mut arr = [Pair(5, 1), Pair(2, 3), Pair(4, 1), Pair(2, 1), Pair(2, 5)];
        merge_sort(&mut arr);
        assert_eq!(
            arr,
            [Pair(2, 3), Pair(2, 1), Pair(2, 5), Pair(4, 1), Pair(5, 1)]
        );
    }

    #[test]
    fn basic_comparator() {
        // Assertion does not work otherwise
        #[derive(Debug, PartialEq)]
        struct NoOrd(i32);

        let mut arr = [NoOrd(1), NoOrd(5), NoOrd(2), NoOrd(3), NoOrd(7)];
        // Should not compile - the trait `PartialOrd` is not implemented for `NoOrd`
        // insertion_sort(&mut arr);
        merge_sort_by(&mut arr, |a, b| a.0 < b.0);
        assert_eq!(arr, [NoOrd(1), NoOrd(2), NoOrd(3), NoOrd(5), NoOrd(7)]);
    }
}
