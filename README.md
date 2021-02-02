Maximum Budgeted Allocation
====

# Overview
Implementation of maximum budgeted allocation problem.
1. PrimalDual((1 - beta / 4)(1 - epsilon)-approximation)
    * beta is the ratio of bid to budget(0 < beta <= 1)

# Maximum Budgeted Allocation Problem
There is a set of agents A and a set of items Q.  
Each agent i willing to pay b_ij on item j; each agent i also has a budget B_i.    
The goal is to allocate items to agents to maximize revenue.

![mba](https://user-images.githubusercontent.com/9996150/65564622-6edc7a00-df88-11e9-942e-d27da3544894.gif)
# Usage
```rust
let num_agents = 2;
let num_items = 3;

let mut solver = primal_dual::PrimalDual::<u64>::new(num_agents, num_items, 0.01);

// set budget
// solver.set_budget(agent_id, budget)
solver.set_budget(0, 100);
solver.set_budget(1, 200);

// set bid
// solver.set_bid(agent_id, item_id, bid)
solver.set_bid(0, 0, 50);
solver.set_bid(0, 1, 60);
solver.set_bid(0, 2, 60);
solver.set_bid(1, 0, 90);
solver.set_bid(1, 1, 10);
solver.set_bid(1, 2, 20);

// solve
solver.solve();

// show result
let assignment = solver.make_valid_assignment();
println!("total consumption: {}", assignment.total_consumption);
```

# References
* [On the Approximability of Budgeted Allocations and Improved Lower Bounds for Submodular Welfare Maximization and GAP](https://ieeexplore.ieee.org/abstract/document/4691001)
