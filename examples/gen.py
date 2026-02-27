import random


def gen_one():
    # Size of the array
    n = random.randint(1, 50)
    # Target sum
    K = random.randint(-50, 50)
    # Elements (can be negative)
    a = [random.randint(-50, 50) for _ in range(n)]
    a.sort()  # ensure sorted for two-pointer solution

    print(n, K)
    print(" ".join(map(str, a)))


if __name__ == "__main__":
    gen_one()

