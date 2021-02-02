import pulp
import random


class MaximumBudgetedAllocation:
    def __init__(self, num_agents, num_items, budgets):
        assert num_agents > 0
        assert num_items > 0
        assert len(budgets) == num_agents

        self.num_agents = num_agents
        self.num_items = num_items
        self.budgets = budgets

        self.problem = pulp.LpProblem("BudgetedAllocation", pulp.LpMaximize)
        self.variables = {}
        self.c = {}

        self.status = None
        self.objective = None
        self.assignment = {}

    def add(self, agent_id, item_id, bid):
        assert 0 <= agent_id < self.num_agents
        assert 0 <= item_id < self.num_items
        assert bid >= 0

        # x_ij = pulp.LpVariable(f"{agent_id}_{item_id}", 0.0, 1.0, pulp.LpContinuous)  // for Integer programming
        x_ij = pulp.LpVariable(f"{agent_id}_{item_id}", 0.0, 1.0, pulp.LpBinary)
        self.variables[(agent_id, item_id)] = x_ij
        self.c[(agent_id, item_id)] = bid

    def solve(self):
        v = []
        for agent_id in range(self.num_agents):
            for item_id in range(self.num_items):
                if (agent_id, item_id) in self.variables:
                    v.append(self.variables[(agent_id, item_id)] * self.c[(agent_id, item_id)])
        self.problem += pulp.lpSum(v)

        for agent_id in range(self.num_agents):
            v = []
            for item_id in range(self.num_items):
                if (agent_id, item_id) in self.variables:
                    v.append(self.variables[(agent_id, item_id)] * self.c[(agent_id, item_id)])

            if v:
                self.problem += pulp.lpSum(v) <= self.budgets[agent_id]

        for item_id in range(self.num_items):
            v = []
            for agent_id in range(self.num_agents):
                if (agent_id, item_id) in self.variables:
                    v.append(self.variables[(agent_id, item_id)])
            if v:
                self.problem += pulp.lpSum(v) <= 1.0

        status = self.problem.solve()
        self.status = pulp.LpStatus[status]
        self.objective = pulp.value(self.problem.objective)

        for k, v in self.variables.items():
            if pulp.value(v) != 0 and pulp.value(v) is not None:
                self.assignment[k] = pulp.value(v)


def test1():
    random.seed(7)

    num_agents = 3
    num_items = 7
    budgets = [0] * num_agents
    for i in range(num_agents):
        budgets[i] = random.randint(5, 10)

    mba = MaximumBudgetedAllocation(num_agents, num_items, budgets)

    for agent_id in range(num_agents):
        for item_id in range(num_items):
            cost = random.randint(1, 5)
            if random.randint(0, 2) == 0:
                continue
            mba.add(agent_id, item_id, cost)

    mba.solve()
    print(mba.status)
    print(int(mba.objective))
    for k, v in mba.assignment.items():
        print(f"agent: {k[0]} - items:{k[1]}, value:{v}")


def test2(file_path: str):
    with open(file_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    no = 0
    num_agents, num_items = map(int, lines[no].strip().split())
    no += 1

    budgets = [0] * num_agents
    for agent_id in range(num_agents):
        budgets[agent_id] = int(lines[no])
        no += 1

    mba = MaximumBudgetedAllocation(num_agents, num_items, budgets)

    data = [[0] * num_items for _ in range(num_agents)]
    for line in lines[no:]:
        agent_id, item_id, bid = map(int, line.strip().split())
        mba.add(agent_id, item_id, bid)
        data[agent_id][item_id] = bid

    mba.solve()

    print(f"status:{mba.status}")
    print(f"consumption: {int(mba.objective)}/{sum(budgets)}")

    total_consume = 0
    for k, v in mba.assignment.items():
        # print(k, v, data[k[0]][k[1]])
        total_consume += data[k[0]][k[1]] * v
    # assert(mba.objective == total_consume)

    return mba.objective, sum(budgets)


def main():
    import os
    test1()

    directory = "../test/data"
    for file_name in os.listdir(directory):
        if ".in" in file_name:
            print(file_name)
            file_path = os.path.join(directory, file_name)
            objective, total = test2(file_path)

            with open(file_path.replace(".in", ".out"), "w") as f:
                f.write("{0}\n".format(int(objective)))


if __name__ == '__main__':
    main()
