import random


def gen_one():
    # Small constraints for stress testing with a brute-force reference.
    n = random.randint(1, 20)
    k = random.randint(0, 10)
    x = random.randint(1, 10)

    a = [random.randint(1, 50) for _ in range(n)]

    print(n, k, x)
    print(" ".join(map(str, a)))


if __name__ == "__main__":
    gen_one()

