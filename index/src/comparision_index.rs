use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{collation::Collatable, reactor::SubscriptionId};
use ankql::ast;

/// An index for a specific field and comparison operator
/// Used for storage engines that don't offer watchable indexes
/// This is a naive implementation that uses a BTreeMap for each operator
/// This is not efficient for large datasets. If this ends up being used in production
/// we should consider using a more efficient index structure like a B+ tree with subscription
/// registrations on intermediate nodes for range comparisons.
#[derive(Debug, Default)]
pub(crate) struct ComparisonIndex {
    pub(crate) eq: HashMap<Vec<u8>, Vec<SubscriptionId>>,
    pub(crate) gt: BTreeMap<Vec<u8>, Vec<SubscriptionId>>,
    pub(crate) lt: BTreeMap<Vec<u8>, Vec<SubscriptionId>>,
}

impl ComparisonIndex {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            eq: HashMap::new(),
            gt: BTreeMap::new(),
            lt: BTreeMap::new(),
        }
    }

    fn for_entry<F, V>(&mut self, value: V, op: ast::ComparisonOperator, f: F)
    where
        F: FnOnce(&mut Vec<SubscriptionId>),
        V: Collatable,
    {
        match op {
            ast::ComparisonOperator::Equal => {
                let entry = self.eq.entry(value.to_bytes()).or_default();
                f(entry);
            }
            ast::ComparisonOperator::GreaterThan => {
                let entry = self.gt.entry(value.to_bytes()).or_default();
                f(entry);
            }
            ast::ComparisonOperator::LessThan => {
                let entry = self.lt.entry(value.to_bytes()).or_default();
                f(entry);
            }
            ast::ComparisonOperator::GreaterThanOrEqual => {
                // x >= 5 is equivalent to x > 4
                if let Some(pred) = value.predecessor_bytes() {
                    let entry = self.gt.entry(pred).or_default();
                    f(entry);
                } else {
                    // If there is no predecessor (e.g., value is minimum), then this matches everything
                    let entry = self.gt.entry(Vec::new()).or_default();
                    f(entry);
                }
            }
            ast::ComparisonOperator::LessThanOrEqual => {
                // x <= 5 is equivalent to x < 6
                if let Some(succ) = value.successor_bytes() {
                    let entry = self.lt.entry(succ).or_default();
                    f(entry);
                }
            }
            _ => panic!("Unsupported operator: {:?}", op),
        }
    }

    pub fn add<V: Collatable>(
        &mut self,
        value: V,
        op: ast::ComparisonOperator,
        sub_id: SubscriptionId,
    ) {
        self.for_entry(value, op, |entries| entries.push(sub_id));
    }

    pub fn remove<V: Collatable>(
        &mut self,
        value: V,
        op: ast::ComparisonOperator,
        sub_id: SubscriptionId,
    ) {
        self.for_entry(value, op, |entries| {
            if let Some(pos) = entries.iter().position(|id| *id == sub_id) {
                entries.remove(pos);
            }
        });
    }
    pub fn find_matching<V: Collatable>(&self, value: V) -> Vec<SubscriptionId> {
        let mut result = BTreeSet::new();
        let bytes = value.to_bytes();

        println!("\nChecking value with bytes: {:?}", bytes);

        // Check exact matches
        if let Some(subs) = self.eq.get(&bytes) {
            println!("eq match - subs: {:?}", subs);
            result.extend(subs.iter().cloned());
        }

        // Check greater than matches (x > threshold)
        println!("\ngt index contents: {:?}", self.gt);
        println!("gt range query: ..{:?}", bytes);
        for (threshold, subs) in self.gt.range(..bytes.clone()) {
            println!("gt got threshold: {:?}", threshold);
            result.extend(subs.iter().cloned());
        }

        // Check less than matches (x < threshold)
        println!("lt index contents: {:?}", self.lt);
        if let Some(pred) = value.successor_bytes() {
            println!("lt range query: {:?}..", pred);
            for (threshold, subs) in self.lt.range(pred..) {
                println!("lt got threshold: {:?}", threshold);
                result.extend(subs.iter().cloned());
            }
        }

        // Should just return the BTreeSet but this sucks for test cases
        result.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ComparisonIndex;
    use crate::reactor::{SubscriptionId, Value};
    use ankql::ast;

    #[test]
    fn test_field_index() {
        let mut index = ComparisonIndex::new();

        // Less than 8 ------------------------------------------------------------
        let sub0 = SubscriptionId::from(0);
        index.add(
            ast::Literal::Integer(8),
            ast::ComparisonOperator::LessThan,
            sub0,
        );

        // 8 should match nothing
        assert!(index.find_matching(Value::Integer(8)).is_empty());

        // 7 should match sub0
        assert_eq!(index.find_matching(Value::Integer(7)), vec![sub0]);

        let sub1 = SubscriptionId::from(1);

        // Greater than 20 ------------------------------------------------------------
        index.add(
            ast::Literal::Integer(20),
            ast::ComparisonOperator::GreaterThan,
            sub1,
        );

        // 20 should match nothing
        assert!(index.find_matching(Value::Integer(20)).is_empty());

        // 21 should match sub1
        assert_eq!(index.find_matching(Value::Integer(21)), vec![sub1]);

        // // Add subscriptions for various numeric comparisons
        index.add(
            ast::Literal::Integer(5),
            ast::ComparisonOperator::Equal,
            sub0,
        );
        // println!("MARK 4: Added = 5 subscription");

        // // Test exact match (5)
        // println!("Testing exact match (5)");
        assert_eq!(index.find_matching(Value::Integer(5)), vec![sub0]);

        // Less than 25 ------------------------------------------------------------
        index.add(
            ast::Literal::Integer(25),
            ast::ComparisonOperator::LessThan,
            sub0,
        );

        // 22 should match sub0 and sub1 because > 20 and < 25
        assert_eq!(index.find_matching(Value::Integer(22)), vec![sub0, sub1]);

        // 25 should match sub1 because > 20 and !< 25
        assert_eq!(index.find_matching(Value::Integer(25)), vec![sub1]);

        // 26 should match sub0 and sub1 because > 20 and !< 25
        assert_eq!(index.find_matching(Value::Integer(26)), vec![sub1]);
    }
}
