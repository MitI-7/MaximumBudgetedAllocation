use ordered_float::NotNan;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

pub struct MaximumBudgetedAllocation {
    num_agents: usize,
    num_items: usize,
    epsilon: f64,
    budgets: Vec<f64>,
    bid: Vec<Vec<f64>>,

    alpha: Vec<f64>,
    beta: f64,
    consumptions: Vec<f64>,
    gamma: Vec<VecDeque<usize>>,                            // gamma[agent_id] = [item_id, ...]
    item_agent: Vec<BinaryHeap<(NotNan<f64>, u32, usize)>>, // item_agent[item_id] = [(price, num_update, agent_id), ...]
    num_update: Vec<u32>,
    max_bid_agent: Vec<usize>,
}

// On the Approximability of Budgeted Allocations and Improved Lower Bounds for Submodular Welfare Maximization and GAP
// (1 - beta / 4)(1 - epsilon)-approximation algorithm for maximum budgeted allocation
// n : num of agents, m : num of items
// Õ(nm / epsilon)
#[allow(dead_code)]
impl MaximumBudgetedAllocation {
    pub fn new(num_agents: usize, num_items: usize, epsilon: f64) -> Self {
        MaximumBudgetedAllocation {
            num_agents,
            num_items,
            epsilon,
            budgets: vec![0.0; num_agents],
            bid: vec![vec![0.0; num_items]; num_agents],
            alpha: vec![0.0; num_agents],
            beta: 0.0,
            consumptions: vec![0.0; num_agents],
            gamma: vec![VecDeque::new(); num_agents],
            item_agent: vec![BinaryHeap::new(); num_items],
            num_update: vec![0; num_agents],
            max_bid_agent: vec![num_agents; num_items],
        }
    }

    pub fn set_budget(&mut self, agent_id: usize, budget: f64) {
        debug_assert!(budget >= 0.0);
        self.budgets[agent_id] = budget;
    }

    pub fn get_budget(&self, agent_id: usize) -> f64 {
        self.budgets[agent_id]
    }

    pub fn set_bid(&mut self, agent_id: usize, item_id: usize, bid: f64) {
        debug_assert!(bid > 0.0);

        if bid > self.budgets[agent_id] {
            return;
        }

        self.bid[agent_id][item_id] = bid;
        let price = self.price(agent_id, item_id);
        self.item_agent[item_id].push((NotNan::new(price).unwrap(), 0, agent_id));

        let max_bid_agent_id = self.max_bid_agent[item_id];
        if max_bid_agent_id >= self.num_agents {
            self.max_bid_agent[item_id] = agent_id;
        } else {
            let max_bid = self.bid[max_bid_agent_id][item_id];
            if bid > max_bid {
                self.max_bid_agent[item_id] = agent_id;
            }
        }

        self.beta = self.beta.max(bid / self.budgets[agent_id]);
    }

    pub fn get_bid(&self, agent_id: usize, item_id: usize) -> f64 {
        self.bid[agent_id][item_id]
    }

    pub fn get_approximation_ratio(&self) -> f64 {
        (1.0 - self.beta / 4.0) * (1.0 - self.epsilon)
    }

    pub fn solve(&mut self) {
        self.initialize();

        let mut all_agents_are_paid_for = false;

        while !all_agents_are_paid_for {
            all_agents_are_paid_for = true;

            for agent_id in 0..self.num_agents {
                while !self.is_paid_for(agent_id) {
                    all_agents_are_paid_for = false;

                    let mut num_unique = 0;
                    let num = self.gamma[agent_id].len();
                    // erase wrongly allocated items
                    for _ in 0..num {
                        let item_id = self.gamma[agent_id].pop_front().unwrap();
                        let max_agent_id = self.max_price_agent(item_id);

                        // item_id is rightly allocated
                        if max_agent_id == agent_id {
                            num_unique += if self.item_agent.len() == 1 { 1 } else { 0 };
                            self.gamma[agent_id].push_back(item_id);
                        }
                        // item_id is wrongly allocated
                        else {
                            // erase item_id from agent_id
                            self.consumptions[agent_id] -= self.bid[agent_id][item_id];

                            // insert item_id to max_agent_id
                            self.gamma[max_agent_id].push_back(item_id);
                            self.consumptions[max_agent_id] += self.bid[max_agent_id][item_id];

                            if self.is_paid_for(agent_id) {
                                break;
                            }
                        }
                    }

                    if num_unique == num {
                        for _ in 0..self.num_items {
                            if self.is_paid_for(agent_id) {
                                break;
                            }
                            self.update_alpha(agent_id);
                        }
                    }

                    // update alpha
                    if !self.is_paid_for(agent_id) {
                        self.update_alpha(agent_id);
                    }
                }
            }
        }
    }

    pub fn get_dual_objective_value(&self) -> f64 {
        (0..self.num_agents)
            .map(|agent_id| {
                self.budgets[agent_id] * self.alpha[agent_id] + self.consumptions[agent_id] * (1.0 - self.alpha[agent_id])
            })
            .sum()
    }

    pub fn get_primal_objective_value(&self) -> f64 {
        (0..self.num_agents)
            .map(|agent_id| self.consumptions[agent_id].min(self.budgets[agent_id]))
            .sum()
    }

    pub fn get_assignment(&mut self) -> &Vec<VecDeque<usize>> {
        self.gamma.as_mut()
    }

    fn initialize(&mut self) {
        for (item_id, &agent_id) in self.max_bid_agent.iter().enumerate() {
            // no agent can assign item id
            if agent_id == self.num_agents {
                continue;
            }

            self.gamma[agent_id].push_back(item_id);
            self.consumptions[agent_id] += self.bid[agent_id][item_id];
        }
    }

    fn price(&self, agent_id: usize, item_id: usize) -> f64 {
        (1.0 - self.alpha[agent_id]) * self.bid[agent_id][item_id]
    }

    fn update_alpha(&mut self, agent_id: usize) {
        self.alpha[agent_id] = match self.num_update[agent_id] {
            0 => self.epsilon,
            _ => self.alpha[agent_id] * (1.0 + self.calc_epsilon(agent_id)),
        };
        self.num_update[agent_id] += 1;
    }

    fn calc_epsilon(&self, agent_id: usize) -> f64 {
        let a = self.alpha[agent_id];
        self.epsilon * ((1.0 - a) / a)
    }

    fn max_price_agent(&mut self, item_id: usize) -> usize {
        loop {
            let (price, num, agent_id) = *self.item_agent[item_id].peek().unwrap();

            // price is up-to-date
            if num == self.num_update[agent_id] {
                return agent_id;
            }

            let updated_price = NotNan::new(self.price(agent_id, item_id)).unwrap();
            debug_assert!(updated_price < price); // price is monotone decreasing

            let mut v = self.item_agent[item_id].peek_mut().unwrap();
            *v = (updated_price, self.num_update[agent_id], agent_id);
        }
    }

    fn is_paid_for(&self, agent_id: usize) -> bool {
        self.consumptions[agent_id] <= self.U(agent_id) * self.budgets[agent_id]
    }

    #[allow(non_snake_case)]
    // ((1.0 - a) * (4.0 - self.beta) + self.beta) / ((1.0 - a) * (4.0 - self.beta))
    fn U(&self, agent_id: usize) -> f64 {
        let denominator = (self.alpha[agent_id] - 1.0) * (self.beta - 4.0);
        self.beta / denominator + 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::MaximumBudgetedAllocation;
    use std::fs;
    use std::fs::read_to_string;
    use std::path::Path;

    #[test]
    fn sample() {
        let num_agents = 2;
        let num_items = 3;

        let mut solver = MaximumBudgetedAllocation::new(num_agents, num_items, 0.01);

        solver.set_budget(0, 300.0);
        solver.set_budget(1, 400.0);

        solver.set_bid(0, 0, 200.0);
        solver.set_bid(0, 1, 200.0);
        solver.set_bid(0, 2, 100.0);

        solver.set_bid(1, 0, 100.0);
        solver.set_bid(1, 1, 100.0);
        solver.set_bid(1, 2, 200.0);

        solver.solve();

        let primal_objective_value = solver.get_primal_objective_value();
        let dual_objective_value = solver.get_dual_objective_value();
        let approximate_rate = solver.get_approximation_ratio();
        assert_eq!(primal_objective_value, 500.0);
        assert!(primal_objective_value >= dual_objective_value * approximate_rate);
    }

    #[test]
    fn random() {
        let directory_path = Path::new("test_cases/random");
        let epsilon = 0.01;

        match fs::read_dir(directory_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".in") {
                                let file_name = file_name.replace(".in", "");
                                let input_file_path = format!("{}.in", file_name);
                                let output_file_path = format!("{}.out", file_name);

                                println!("{}/{}", input_file_path, output_file_path);

                                let (primal, ratio) = get_result(&(directory_path.join(Path::new(&input_file_path))), epsilon);
                                let opt = get_ans(&directory_path.join(Path::new(&output_file_path)));
                                println!("{:}/{:}", primal, opt);
                                assert!(primal >= opt * ratio);
                                println!("");
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading directory: {}", err);
            }
        }
    }

    fn get_result(file_path: &Path, epsilon: f64) -> (f64, f64) {
        let data = read_to_string(file_path);
        let xy = match data {
            Ok(content) => content,
            Err(error) => {
                panic!("Could not open or find file:{} {}", file_path.to_str().unwrap(), error);
            }
        };
        let xy_pairs: Vec<&str> = xy.trim().split("\n").collect();

        let v: Vec<&str> = xy_pairs[0].trim().split(",").collect();
        let num_agents = v[0].parse().unwrap();
        let num_items: usize = v[1].parse().unwrap();

        let mut solver = MaximumBudgetedAllocation::new(num_agents, num_items, epsilon);

        let v: Vec<&str> = xy_pairs[1].trim().split(",").collect();
        for agent_id in 0..num_agents {
            let budget = v[agent_id].parse().unwrap();
            solver.set_budget(agent_id, budget);
        }

        for (i, _pair) in xy_pairs.iter().enumerate() {
            if i <= 1 {
                continue;
            }
            let vv: Vec<&str> = xy_pairs[i].trim().split(",").collect();
            let agent_id: usize = vv[0].parse().unwrap();
            let item_id: usize = vv[1].parse().unwrap();
            let bid: f64 = vv[2].parse().unwrap();
            solver.set_bid(agent_id, item_id, bid);
        }

        solver.solve();
        (solver.get_primal_objective_value(), solver.get_approximation_ratio())
    }

    fn get_ans(file_path: &Path) -> f64 {
        let data = read_to_string(file_path).unwrap();
        data.trim().parse().unwrap()
    }
}
