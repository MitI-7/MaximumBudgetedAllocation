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

        x_ij = pulp.LpVariable(f"{agent_id}_{item_id}", 0.0, 1.0, pulp.LpContinuous)  # for Liner programming
        # x_ij = pulp.LpVariable(f"{agent_id}_{item_id}", 0.0, 1.0, pulp.LpBinary)
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

        status = self.problem.solve(pulp.PULP_CBC_CMD(msg=0))
        self.status = pulp.LpStatus[status]
        self.objective = pulp.value(self.problem.objective)

        for k, v in self.variables.items():
            if pulp.value(v) != 0 and pulp.value(v) is not None:
                self.assignment[k] = pulp.value(v)


def sample():
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
        print(f"agent id: {k[0]}, item id:{k[1]}, value:{v:.3f}")


def main():
    sample()


if __name__ == '__main__':
    main()
