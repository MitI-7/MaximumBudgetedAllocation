use crate::assignment::Assignment;
use crate::campaigns::Campaigns;
use num::ToPrimitive;
use num_traits::{FromPrimitive, NumAssign};
use ordered_float::OrderedFloat;
use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::fmt::Display;

pub struct PrimalDual<N: NumAssign + ToPrimitive + FromPrimitive + Copy + PartialOrd + Display> {
    agents: Campaigns<N>,

    num_agents: usize,
    num_items: usize,
    epsilon: f64,

    beta: f64,
    item_agent: Vec<BinaryHeap<(OrderedFloat<f64>, u64, usize)>>, // item_agent[item_id] = [(price, num_update, agent_id), ...]
    alpha: Vec<f64>,
    assignment_temp: Vec<VecDeque<usize>>, // assignment_temp[agent_id] = [item_id, ...]
    max_agent: Vec<i64>,
    data: Vec<Vec<N>>, // data[agent_id][item_id] = bid
    num_update: Vec<u64>,
}

// (1 - beta / 4)(1 - epsilon)-approximation algorithm for budgeted allocation
// n : num of agents
// m : num of items
impl<N: NumAssign + ToPrimitive + FromPrimitive + Copy + PartialOrd + Display> PrimalDual<N> {
    pub fn new(num_agents: usize, num_items: usize, epsilon: f64) -> Self {
        PrimalDual {
            agents: Campaigns::new(num_agents),
            num_agents: num_agents,
            num_items: num_items,
            epsilon: epsilon,
            beta: 0.0,
            item_agent: vec![BinaryHeap::new(); num_items],
            alpha: vec![0.0; num_agents],
            assignment_temp: vec![VecDeque::new(); num_agents],
            max_agent: vec![-1; num_items],
            data: vec![vec![N::zero(); num_items]; num_agents],
            num_update: vec![0; num_agents],
        }
    }

    pub fn set_budget(&mut self, agent_id: usize, budget: N) {
        self.agents.set_budget(agent_id, budget);
    }

    pub fn set_bid(&mut self, agent_id: usize, item_id: usize, bid: N) {
        debug_assert!(bid > N::zero());
        debug_assert!(bid <= self.agents.budgets[agent_id]);

        if bid <= N::zero() {
            return;
        }
        if bid > self.agents.budgets[agent_id] {
            return;
        }

        self.data[agent_id][item_id] = bid;
        self.item_agent[item_id].push((OrderedFloat(bid.to_f64().unwrap()), 0, agent_id));

        if self.max_agent[item_id] == -1 {
            self.max_agent[item_id] = agent_id as i64;
        } else {
            let max_bid = self.data[self.max_agent[item_id] as usize][item_id];
            if bid > max_bid {
                self.max_agent[item_id] = agent_id as i64;
            }
        }

        let b = bid.to_f64().unwrap() / self.agents.budgets[agent_id].to_f64().unwrap();
        if b > self.beta {
            self.beta = b;
        }
    }

    pub fn solve(&mut self) {
        self.initialize();

        let mut have_not_paid_for_agent = true;
        while have_not_paid_for_agent {
            have_not_paid_for_agent = false;
            for agent_id in 0..self.num_agents {
                if self.is_paid_for(agent_id) {
                    continue;
                }

                have_not_paid_for_agent = true;
                while !self.is_paid_for(agent_id) {
                    let mut num_unique = 0;
                    let num: u64 = self.assignment_temp[agent_id].len() as u64;
                    // erase wrongly allocated items
                    for _i in 0..num {
                        let item_id = self.assignment_temp[agent_id].pop_front().unwrap();
                        let max_agent_id = self.max_price_agent(item_id);

                        // item_id is rightly allocated
                        if max_agent_id == agent_id {
                            num_unique += (self.item_agent[item_id].len() == 1) as u64;
                            self.assignment_temp[agent_id].push_front(item_id);
                            continue;
                        }

                        // erase item_id from agent_id
                        self.agents.consumptions[agent_id] -= self.data[agent_id][item_id];

                        // insert item_id to max_agent_id
                        self.assignment_temp[max_agent_id].push_front(item_id);
                        self.agents.consumptions[max_agent_id] += self.data[max_agent_id][item_id];

                        if self.is_paid_for(agent_id) {
                            break;
                        }
                    }

                    if num_unique == num {
                        while !self.assignment_temp[agent_id].is_empty() && !self.is_paid_for(agent_id) {
                            let item_id = self.assignment_temp[agent_id].pop_front().unwrap();
                            self.agents.consumptions[agent_id] -= self.data[agent_id][item_id];
                        }
                    }

                    // update alpha
                    if !self.is_paid_for(agent_id) {
                        if self.num_update[agent_id] == 0 {
                            self.alpha[agent_id] = self.epsilon;
                        } else {
                            self.alpha[agent_id] = self.alpha[agent_id] * (1.0 + self.calc_epsilon(agent_id));
                        }
                        self.num_update[agent_id] += 1;
                    }
                }
            }
        }
    }

    fn initialize(&mut self) {
        for item_id in 0..self.num_items {
            let agent_id = self.max_agent[item_id];
            if agent_id == -1 {
                continue;
            }

            let bid = self.data[agent_id as usize][item_id];
            self.assignment_temp[agent_id as usize].push_back(item_id);
            self.agents.consumptions[agent_id as usize] += bid;
        }
    }

    fn calc_epsilon(&self, agent_id: usize) -> f64 {
        let a = self.alpha[agent_id];
        self.epsilon * ((1.0 - a) / a)
    }

    fn max_price_agent(&mut self, item_id: usize) -> usize {
        loop {
            let (price, num, agent_id) = self.item_agent[item_id].pop().unwrap();

            if num == self.num_update[agent_id] {
                self.item_agent[item_id].push((price, num, agent_id));
                return agent_id;
            }

            let updated_price = self.calc_price(agent_id, item_id);
            self.item_agent[item_id].push((OrderedFloat(updated_price), self.num_update[agent_id], agent_id));
        }
    }

    fn calc_price(&self, agent_id: usize, item_id: usize) -> f64 {
        self.data[agent_id][item_id].to_f64().unwrap() * (1.0 - self.alpha[agent_id])
    }

    fn is_paid_for(&self, agent_id: usize) -> bool {
        self.agents.consumptions[agent_id].to_f64().unwrap() <= self.U(agent_id) * self.agents.budgets[agent_id].to_f64().unwrap()
    }

    #[allow(non_snake_case)]
    fn U(&self, agent_id: usize) -> f64 {
        let a = self.alpha[agent_id];
        ((1.0 - a) * (4.0 - self.beta) + self.beta) / ((1.0 - a) * (4.0 - self.beta))
    }

    pub fn make_valid_assignment(&mut self) -> Assignment<N> {
        let mut temp_agents = Campaigns::new(self.num_agents);
        let mut assignment = Assignment::new(self.num_agents);
        for agent_id in 0..self.num_agents {
            temp_agents.set_budget(agent_id, self.agents.budgets[agent_id]);
            assignment.set_budget(agent_id, self.agents.budgets[agent_id]);
        }

        // greedy
        let mut used_item = vec![false; self.num_items];
        for agent_id in 0..self.num_agents {
            let mut bid_items = Vec::new();
            while self.assignment_temp[agent_id].len() > 0 {
                let item_id = self.assignment_temp[agent_id].pop_front().unwrap();
                let bid = self.data[agent_id][item_id];
                bid_items.push((OrderedFloat(bid.to_f64().unwrap()), item_id));
            }
            bid_items.sort();
            bid_items.reverse();
            for (bid, item_id) in bid_items {
                if temp_agents.can_assignment(agent_id, N::from_f64(bid.to_f64().unwrap()).unwrap()) {
                    temp_agents.consume(agent_id, N::from_f64(bid.to_f64().unwrap()).unwrap());
                    assignment.assign(agent_id, item_id, N::from_f64(bid.to_f64().unwrap()).unwrap());
                    used_item[item_id] = true;
                }
            }
        }

        for item_id in 0..self.num_items {
            if used_item[item_id] {
                continue;
            }
            let mut v = Vec::new();
            while !self.item_agent[item_id].is_empty() {
                let (_price, _num, agent_id) = self.item_agent[item_id].pop().unwrap();
                let bid = self.data[agent_id][item_id];

                if temp_agents.can_assignment(agent_id, bid) {
                    v.push((OrderedFloat(bid.to_f64().unwrap()), agent_id));
                }
            }
            v.sort();
            v.reverse();

            for (bid, agent_id) in &v {
                if temp_agents.can_assignment(*agent_id, N::from_f64(bid.to_f64().unwrap()).unwrap()) {
                    temp_agents.consume(*agent_id, N::from_f64(bid.to_f64().unwrap()).unwrap());
                    assignment.assign(*agent_id, item_id, N::from_f64(bid.to_f64().unwrap()).unwrap());
                    used_item[item_id] = true;
                    break;
                }
            }
        }

        assignment
    }
}
