"""Quick smoke test for the pykosh Rust plugin."""

import pykosh


def main():
    # Functions
    print("add(3, 4) =", pykosh.add(3, 4))
    print("greet('World') =", pykosh.greet("World"))
    print("fibonacci(10) =", pykosh.fibonacci(10))

    # Class
    c = pykosh.Counter(start=10)
    print(f"\n{c}")
    c.increment(5)
    print(f"After increment(5): {c}")
    c.reset()
    print(f"After reset():      {c}")


if __name__ == "__main__":
    main()

