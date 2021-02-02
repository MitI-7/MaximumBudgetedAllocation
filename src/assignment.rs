use num::ToPrimitive;
use num_traits::NumAssign;
use std::collections::HashMap;
use std::fmt::Display;

pub struct Assignment<N: NumAssign + ToPrimitive + Copy + PartialOrd + Display> {
    pub total_consumption: N,
    pub budgets: Vec<N>,
    pub assignment: HashMap<usize, Vec<usize>>,
}

impl<N: NumAssign + ToPrimitive + Copy + PartialOrd + Display> Assignment<N> {
    pub fn new(num_agents: usize) -> Self {
        Assignment { total_consumption: N::zero(), budgets: vec![N::zero(); num_agents], assignment: HashMap::new() }
    }

    pub fn set_budget(&mut self, agent_id: usize, budget: N) {
        self.budgets[agent_id] = budget;
    }

    pub fn assign(&mut self, agent_id: usize, item_id: usize, bid: N) {
        self.total_consumption += bid;
        match self.assignment.get_mut(&agent_id) {
            Some(v) => v.push(item_id),
            None => {
                self.assignment.insert(agent_id, vec![item_id]);
            }
        };
    }
}
