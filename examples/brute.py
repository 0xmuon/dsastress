def solve():
    import sys

    data = list(map(int, sys.stdin.read().split()))
    it = iter(data)
    n = next(it)
    K = next(it)
    a = [next(it) for _ in range(n)]

    cnt = 0
    for i in range(n):
        for j in range(i + 1, n):
            if a[i] + a[j] == K:
                cnt += 1
    print(cnt)


if __name__ == "__main__":
    solve()

