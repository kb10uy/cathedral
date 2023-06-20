use std::cmp::min;

pub struct Lyricism<I, D, R, S> {
    /// Insertion that transforms query string to target string.
    insert_cost_function: I,

    /// Deletion that transforms query string to target string.
    delete_cost_function: D,

    /// Replacement that transforms query string to target string.
    replace_cost_function: R,

    /// Substring bonus.
    substring_bonus_function: S,
}

impl<
        I: Fn(char) -> usize,
        D: Fn(char) -> usize,
        R: Fn(char, char) -> usize,
        S: Fn(&str, usize) -> isize,
    > Lyricism<I, D, R, S>
{
    pub fn new(insert: I, delete: D, replace: R, substring: S) -> Lyricism<I, D, R, S> {
        Lyricism {
            insert_cost_function: insert,
            delete_cost_function: delete,
            replace_cost_function: replace,
            substring_bonus_function: substring,
        }
    }

    pub fn distance(&self, query: &str, target: &str) -> isize {
        if target.is_empty() {
            return query.chars().map(&self.delete_cost_function).sum::<usize>() as isize;
        }

        let target_chars: Vec<_> = target.chars().enumerate().collect();
        let query_distances: Vec<_> = query
            .chars()
            .scan(0, |sum, c| {
                *sum += (self.delete_cost_function)(c);
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

        let mut result_distance = target.chars().map(&self.insert_cost_function).sum();

        for (qi, qchar) in query.chars().enumerate() {
            let mut replace_base = if qi == 0 { 0 } else { query_distances[qi - 1] };
            result_distance = query_distances[qi];

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
                        replace_base + (self.replace_cost_function)(qchar, tchar),
                    )
                };
                replace_base = delete_base;
                target_distances[ti] = result_distance;
            }
        }

        if let Some(position) = target.find(query) {
            result_distance as isize + (self.substring_bonus_function)(query, position)
        } else {
            result_distance as isize
        }
    }
}
