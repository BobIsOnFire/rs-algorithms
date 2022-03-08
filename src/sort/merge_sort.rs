use alloc::vec::Vec;
use core::mem::MaybeUninit;

unsafe fn assume_init_move<T>(init_ref: &mut MaybeUninit<T>) -> T {
    let val: T = core::ptr::read(init_ref.as_mut_ptr());
    *init_ref = MaybeUninit::uninit();
    val
}

unsafe fn assume_init_is_less<T>(
    a: &MaybeUninit<T>,
    b: &MaybeUninit<T>,
    is_less: &impl Fn(&T, &T) -> bool,
) -> bool {
    is_less(a.assume_init_ref(), b.assume_init_ref())
}

fn do_merge<T>(
    slice: &mut [MaybeUninit<T>],
    aux: &mut [MaybeUninit<T>],
    is_less: &impl Fn(&T, &T) -> bool,
) {
    let mut left = (0usize, slice.len() / 2);
    let mut right = (slice.len() / 2, slice.len());

    // Merge elements from `left` and `right` ranges by moving them into `aux`.
    // SAFETY guarantees for the loop:
    // All elements in `slice[left.0..left.1]` are initialized and safe to read.
    // All elements in `slice[right.0..right.1]` are initialized and safe to read.
    // By the end of the loop, both left.0 == left.1 and right.0 == right.1, thus `slice` is unsafe to read.
    // Also, by the end of the loop, all elements are moved into `aux`, now it is safe to read.
    for elem in aux.iter_mut() {
        let index = match 0 {
            _ if left.0 == left.1 => &mut right.0,
            _ if right.0 == right.1 => &mut left.0,
            _ if unsafe { assume_init_is_less(&slice[left.0], &slice[right.0], is_less) } => {
                &mut left.0
            }
            _ => &mut right.0,
        };

        elem.write(unsafe { assume_init_move(&mut slice[*index]) });
        *index += 1;
    }

    // Move elements from `aux` back to `slice`.
    // SAFETY: once this loop is finished, `aux` is moved-from and unsafe to read, `slice` is moved-into and safe to read
    for i in 0..aux.len() {
        slice[i].write(unsafe { assume_init_move(&mut aux[i]) });
    }
}

fn do_merge_sort<T>(
    slice: &mut [MaybeUninit<T>],
    aux: &mut [MaybeUninit<T>],
    is_less: &impl Fn(&T, &T) -> bool,
) {
    if slice.len() > 1 {
        let mid = slice.len() / 2;
        do_merge_sort(&mut slice[..mid], &mut aux[..mid], is_less);
        do_merge_sort(&mut slice[mid..], &mut aux[mid..], is_less);
        do_merge(slice, aux, is_less);
    }
}

pub fn merge_sort_with_comparator<T>(slice: &mut [T], is_less: impl Fn(&T, &T) -> bool) {
    // Allow ourselves to move elements out of `slice` (because merge-sort needs auxiliary memory for merge operation).
    // SAFETY: all `slice` elements are initialized and safe to read from, by now.
    let slice: &mut [MaybeUninit<T>] = unsafe { core::mem::transmute(slice) };

    // Auxiliary memory required for merge operation (eugh, allocation!)
    // SAFETY: `aux` elements are uninitialized and unsafe to read from.
    let mut aux = Vec::with_capacity(slice.len());
    for _ in 0..slice.len() {
        aux.push(MaybeUninit::uninit());
    }

    do_merge_sort(slice, &mut aux, &is_less);
}

pub fn merge_sort<T: PartialOrd>(slice: &mut [T]) {
    merge_sort_with_comparator(slice, <T as PartialOrd>::le)
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
        merge_sort_with_comparator(&mut arr, |a, b| a.0 < b.0);
        assert_eq!(arr, [NoOrd(1), NoOrd(2), NoOrd(3), NoOrd(5), NoOrd(7)]);
    }
}
