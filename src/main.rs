mod assignment;
mod primal_dual;

fn main() {
    let num_agents = 2;
    let num_items = 3;

    let mut solver = primal_dual::PrimalDual::<i64>::new(num_agents, num_items, 0.01);

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
}
