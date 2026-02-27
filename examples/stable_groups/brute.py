def cost_to_bridge(diff, x):
    if diff <= x:
        return 0
    # How many new students needed so that each neighboring difference <= x
    # (m + 1) * x >= diff  =>  m >= diff / x - 1  =>  m = ceil(diff / x) - 1
    from math import ceil
    return max(0, ceil(diff / x) - 1)


def brute_solve(n, k, x, a):
    a = sorted(a)
    if n <= 1:
        return 1

    diffs = []
    for i in range(n - 1):
        d = a[i + 1] - a[i]
        if d > x:
            diffs.append(d)

    # If no "big" gaps, everything is already in one stable group.
    if not diffs:
        return 1

    g = len(diffs)
    costs = [cost_to_bridge(d, x) for d in diffs]

    # Enumerate all subsets of gaps to keep as actual group boundaries.
    # For a subset S of indices of gaps that remain, the number of groups is |S| + 1.
    # We must ensure that for each removed gap (i not in S), the total cost does not exceed k.
    best = g + 1  # maximum possible groups before merging
    for mask in range(1 << g):
        groups = 1
        total_cost = 0
        for i in range(g):
            if mask & (1 << i):
                # keep this gap as a boundary
                groups += 1
            else:
                # bridge this gap
                total_cost += costs[i]
                if total_cost > k:
                    break
        if total_cost <= k:
            best = min(best, groups)
    return best


def main():
    import sys

    data = list(map(int, sys.stdin.read().split()))
    it = iter(data)
    n = next(it)
    k = next(it)
    x = next(it)
    a = [next(it) for _ in range(n)]

    ans = brute_solve(n, k, x, a)
    print(ans)


if __name__ == "__main__":
    main()

