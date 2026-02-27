def solve():
    import sys

    data = list(map(int, sys.stdin.read().split()))
    it = iter(data)
    n = next(it)
    K = next(it)
    a = [next(it) for _ in range(n)]

    i, j = 0, n - 1
    cnt = 0
    while i < j:
        s = a[i] + a[j]
        if s == K:
            # Handle duplicates correctly
            if a[i] == a[j]:
                m = j - i + 1
                cnt += m * (m - 1) // 2
                break
            left_val, right_val = a[i], a[j]
            c1 = c2 = 0
            while i <= j and a[i] == left_val:
                c1 += 1
                i += 1
            while j >= i and a[j] == right_val:
                c2 += 1
                j -= 1
            cnt += c1 * c2
        elif s < K:
            i += 1
        else:
            j -= 1
    print(cnt)


if __name__ == "__main__":
    solve()

