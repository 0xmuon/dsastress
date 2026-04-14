import os
import random


def init_rng():
    # Reproducible if dsastress provides DSASTRESS_SEED and DSASTRESS_TEST.
    seed = int(os.environ.get("DSASTRESS_SEED", "0"))
    t = int(os.environ.get("DSASTRESS_TEST", "0"))
    random.seed((seed << 20) ^ t)


def gen_one():
    # Count pairs (i<j) such that a[i] + a[j] == K.
    n = random.randint(1, 60)
    k = random.randint(-60, 60)
    a = [random.randint(-60, 60) for _ in range(n)]

    # Many solutions use sorting + two pointers; keep it sorted to match typical assumptions.
    a.sort()

    print(n, k)
    print(" ".join(map(str, a)))


if __name__ == "__main__":
    init_rng()
    gen_one()

