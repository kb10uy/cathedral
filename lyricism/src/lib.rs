use std::cmp::min;

pub struct Lyricism<I, D, R> {
    /// Insertion that transforms query string to target string.
    insert_cost_function: I,

    /// Deletion that transforms query string to target string.
    delete_cost_function: D,

    /// Replacement that transforms query string to target string.
    replace_cost_function: R,
}

impl<I: Fn(char) -> usize, D: Fn(char) -> usize, R: Fn(char, char) -> usize> Lyricism<I, D, R> {
    pub fn new(insert: I, delete: D, replace: R) -> Lyricism<I, D, R> {
        Lyricism {
            insert_cost_function: insert,
            delete_cost_function: delete,
            replace_cost_function: replace,
        }
    }

    pub fn distance(&self, query: &str, target: &str) -> usize {
        if target.is_empty() {
            return query.chars().map(&self.delete_cost_function).sum();
        }

        let target_chars: Vec<_> = target.chars().enumerate().collect();
        let query_distances: Vec<_> = query
            .chars()
            .scan(0, |sum, c| {
                *sum += (self.insert_cost_function)(c);
                Some(*sum)
            })
            .collect();
        let mut target_distances: Vec<_> = target
            .chars()
            .scan(0, |sum, c| {
                *sum += (self.insert_cost_function)(c);
                Some(*sum)
            })
            .collect();
        let mut result_distance = target.chars().map(&self.delete_cost_function).sum();

        for (qi, qchar) in query.chars().enumerate() {
            let mut replace_base = query_distances[qi];
            result_distance = replace_base;

            for &(ti, tchar) in &target_chars {
                let delete_base = target_distances[ti];
                let insert_cost = (self.insert_cost_function)(tchar);
                let delete_cost = (self.delete_cost_function)(tchar);
                result_distance = if qchar == tchar {
                    min(
                        min(result_distance + insert_cost, delete_base + delete_cost),
                        replace_base,
                    )
                } else {
                    min(
                        min(result_distance + insert_cost, delete_base + delete_cost),
                        (self.replace_cost_function)(qchar, tchar),
                    )
                };
                replace_base = delete_base;
                target_distances[ti] = result_distance;
            }
        }

        result_distance
    }
}
