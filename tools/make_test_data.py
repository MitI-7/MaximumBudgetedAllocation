import pathlib
import random
from maximum_budgeted_allocation import MaximumBudgetedAllocation
random.seed(722)


def make_test_data(num_agents: int, num_items: int, input_data_file_path: pathlib.Path, output_data_file_path: pathlib.Path):

    budgets = [0.0] * num_agents
    for agent_id in range(num_agents):
        budgets[agent_id] = random.randint(1, 500)

    mba = MaximumBudgetedAllocation(num_agents, num_items, budgets)

    bid_list = []
    for agent_id in range(num_agents):
        for item_id in range(num_items):
            if random.random() < 0.2:
                bid = random.uniform(0, 100) + 0.01
                bid_list.append((agent_id, item_id, bid))
                mba.add(agent_id, item_id, bid)

    mba.solve()

    with input_data_file_path.open("w") as f:
        f.write(f"{num_agents},{num_items},{len(bid_list)}\n")

        f.write(f"{','.join(map(str, budgets))}\n")
        for agent_id, item_id, bid in bid_list:
            f.write(f"{agent_id},{item_id},{bid:.5}\n")

    with output_data_file_path.open("w") as f:
        f.write(f"{mba.objective}")


def main():
    test_directory_path = pathlib.Path("../test_cases/random")

    if not test_directory_path.exists():
        test_directory_path.mkdir(parents=True)

    for test_no in range(30):
        num_agents = random.randint(1, 50)
        num_items = random.randint(1, 1000)
        print(f"#test:{test_no}, #agents:{num_agents}, #items:{num_items}")
        input_file_name = f"{test_no}.in"
        output_file_name = f"{test_no}.out"
        make_test_data(num_agents, num_items, test_directory_path.joinpath(input_file_name), test_directory_path.joinpath(output_file_name))


if __name__ == "__main__":
    main()
