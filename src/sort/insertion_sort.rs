fn find_slot<T>(slice: &[T], element: &T, cmp: &impl Fn(&T, &T) -> bool) -> usize {
    let mut lo = 0usize;
    let mut hi = slice.len();

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if cmp(&slice[mid], element) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    lo
}

pub fn insertion_sort_by<T>(slice: &mut [T], cmp: impl Fn(&T, &T) -> bool) {
    let len = slice.len();
    for i in 0..len {
        let slot = find_slot(&slice[..i], &slice[i], &cmp);
        for k in slot..i {
            slice.swap(k, i);
        }
    }
}

pub fn insertion_sort<T: PartialOrd>(slice: &mut [T]) {
    insertion_sort_by(slice, <T as PartialOrd>::le)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_trait() {
        let mut arr = [1, 5, 2, 3, 7];
        insertion_sort(&mut arr);
        assert_eq!(arr, [1, 2, 3, 5, 7]);
    }

    #[test]
    fn empty_trait() {
        let mut arr: [i32; 0] = [];
        insertion_sort(&mut arr);
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
        insertion_sort(&mut arr);
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
        insertion_sort_by(&mut arr, |a, b| a.0 < b.0);
        assert_eq!(arr, [NoOrd(1), NoOrd(2), NoOrd(3), NoOrd(5), NoOrd(7)]);
    }
}
