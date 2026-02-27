def solve_fast(n, k, x, a):
    a.sort()
    if n <= 1:
        return 1

    # Collect "big" gaps that break stability.
    gaps = []
    for i in range(n - 1):
        d = a[i + 1] - a[i]
        if d > x:
            gaps.append(d)

    # If no big gaps, already one stable group.
    if not gaps:
        return 1

    # Each gap requires some number of new students to bridge.
    # For diff d, cost = ceil(d / x) - 1 = (d - 1) // x
    costs = [max(0, (d - 1) // x) for d in gaps]

    # Initially, each big gap splits groups.
    groups = 1 + len(costs)

    # Greedily bridge cheapest gaps first while we have budget k.
    for c in sorted(costs):
        if c <= k:
            k -= c
            groups -= 1
        else:
            break

    return groups


def main():
    import sys

    data = list(map(int, sys.stdin.read().split()))
    it = iter(data)
    n = next(it)
    k = next(it)
    x = next(it)
    a = [next(it) for _ in range(n)]

    ans = solve_fast(n, k, x, a)
    print(ans)


if __name__ == "__main__":
    main()

