mod maximum_budgeted_allocation;

fn main() {
    let num_agents = 2;
    let num_items = 3;

    let mut solver = maximum_budgeted_allocation::MaximumBudgetedAllocation::new(num_agents, num_items, 0.01);

    // set budget
    solver.set_budget(0, 100.0);
    solver.set_budget(1, 200.0);

    // set bid
    solver.set_bid(0, 0, 50.0);
    solver.set_bid(0, 1, 60.0);
    solver.set_bid(0, 2, 60.0);
    solver.set_bid(1, 0, 90.0);
    solver.set_bid(1, 1, 10.0);
    solver.set_bid(1, 2, 20.0);

    solver.solve();

    // show result
    println!("primal objective value:{:.3}", solver.get_primal_objective_value());
    println!("dual objective value:{:.3}", solver.get_dual_objective_value());
    println!("approximate rate:{:.3}", solver.get_approximation_ratio());

    let assignment = solver.get_assignment();
    for agent_id in 0..num_agents {
        if assignment[agent_id].is_empty() {
            continue;
        }
        println!("agent id:{}, item ids:{:?}", agent_id, assignment[agent_id]);
    }
}
