use std::cmp::Ordering;

/// Sorting algorithms
pub struct SortingAlgorithms;

impl SortingAlgorithms {
    /// Quick sort implementation
    pub fn quick_sort<T: Ord + Clone>(arr: &mut [T]) {
        if arr.len() <= 1 {
            return;
        }

        let pivot_idx = Self::partition(arr);
        let (left, right) = arr.split_at_mut(pivot_idx);
        Self::quick_sort(left);
        Self::quick_sort(&mut right[1..]);
    }

    fn partition<T: Ord + Clone>(arr: &mut [T]) -> usize {
        let pivot_idx = arr.len() - 1;
        let mut i = 0;

        for j in 0..pivot_idx {
            if arr[j] <= arr[pivot_idx] {
                arr.swap(i, j);
                i += 1;
            }
        }

        arr.swap(i, pivot_idx);
        i
    }

    /// Merge sort implementation
    pub fn merge_sort<T: Ord + Clone + Default>(arr: &mut [T]) {
        if arr.len() <= 1 {
            return;
        }

        let mid = arr.len() / 2;
        {
            let (left, right) = arr.split_at_mut(mid);
            Self::merge_sort(left);
            Self::merge_sort(right);
        }

        Self::merge(arr, mid);
    }

    fn merge<T: Ord + Clone + Default>(arr: &mut [T], mid: usize) {
        let mut left_arr = Vec::with_capacity(mid);
        let mut right_arr = Vec::with_capacity(arr.len() - mid);

        left_arr.extend_from_slice(&arr[..mid]);
        right_arr.extend_from_slice(&arr[mid..]);

        let mut i = 0; // left index
        let mut j = 0; // right index
        let mut k = 0; // merged index

        while i < left_arr.len() && j < right_arr.len() {
            if left_arr[i] <= right_arr[j] {
                arr[k] = left_arr[i].clone();
                i += 1;
            } else {
                arr[k] = right_arr[j].clone();
                j += 1;
            }
            k += 1;
        }

        while i < left_arr.len() {
            arr[k] = left_arr[i].clone();
            i += 1;
            k += 1;
        }

        while j < right_arr.len() {
            arr[k] = right_arr[j].clone();
            j += 1;
            k += 1;
        }
    }

    /// Binary search in a sorted array
    pub fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
        let mut left = 0;
        let mut right = arr.len();

        while left < right {
            let mid = left + (right - left) / 2;

            match arr[mid].cmp(target) {
                Ordering::Equal => return Some(mid),
                Ordering::Greater => right = mid,
                Ordering::Less => left = mid + 1,
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_sort() {
        let mut arr = [64, 34, 25, 12, 22, 11, 90];
        SortingAlgorithms::quick_sort(&mut arr);
        assert_eq!(arr, [11, 12, 22, 25, 34, 64, 90]);
    }

    #[test]
    fn test_merge_sort() {
        let mut arr = [64, 34, 25, 12, 22, 11, 90];
        SortingAlgorithms::merge_sort(&mut arr);
        assert_eq!(arr, [11, 12, 22, 25, 34, 64, 90]);
    }

    #[test]
    fn test_binary_search() {
        let arr = [1, 3, 5, 7, 9, 11, 13];
        assert_eq!(SortingAlgorithms::binary_search(&arr, &7), Some(3));
        assert_eq!(SortingAlgorithms::binary_search(&arr, &4), None);
    }
}
