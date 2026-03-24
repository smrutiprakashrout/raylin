use nucleo::{Config, Nucleo};

/// A thin, reusable wrapper around `nucleo` fuzzy matcher.
///
/// Provides a simple interface for case-insensitive, smart-normalized search
/// against a list of strings, returning indices and scores of matching items.
pub struct FuzzyMatcher {
    _private: (),
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Search `items` with `query`.
    ///
    /// Returns a sorted vec of `(original_index, score)` for matching items.
    /// When `query` is empty, all items are returned in their original order with score 0.
    pub fn search(&self, query: &str, items: &[String]) -> Vec<(usize, i32)> {
        if query.is_empty() {
            return items.iter().enumerate().map(|(i, _)| (i, 0)).collect();
        }

        let mut nucleo = Nucleo::<usize>::new(
            Config::DEFAULT,
            std::sync::Arc::new(|| {}),
            None,
            1,
        );

        let injector = nucleo.injector();
        for (idx, item) in items.iter().enumerate() {
            let s = item.clone();
            injector.push(idx, move |i, cols| {
                cols[0] = s.clone().into();
                let _ = i;
            });
        }

        while nucleo.tick(10).running {}

        nucleo.pattern.reparse(
            0,
            query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
            false,
        );

        while nucleo.tick(10).running {}

        let snapshot = nucleo.snapshot();
        let count = snapshot.matched_item_count();
        let mut results = Vec::with_capacity(count as usize);

        for i in 0..count {
            if let Some(item) = snapshot.get_matched_item(i) {
                let score = snapshot.get_item(i)
                    .map(|_| i as i32) // use rank position as score proxy
                    .unwrap_or(i as i32);
                results.push((*item.data, score));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_returns_all() {
        let matcher = FuzzyMatcher::new();
        let items = vec!["firefox".to_string(), "chrome".to_string(), "vim".to_string()];
        let results = matcher.search("", &items);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn no_crash_on_empty_items() {
        let matcher = FuzzyMatcher::new();
        let results = matcher.search("test", &[]);
        assert!(results.is_empty());
    }
}
