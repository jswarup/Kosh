"""Quick smoke test for the pykosh Rust plugin."""

import pykosh


def main():
    # Functions
    print("add(3, 4) =", pykosh.add(3, 4))
    print("greet('World') =", pykosh.greet("World"))
    print("fibonacci(10) =", pykosh.fibonacci(10))

    # Hardware Simulated Adder (Rube)
    print("\n--- Rube Hardware Simulation ---")
    print("add_via_rube(10, 20) =", pykosh.add_via_rube(10, 20))
    print("add_via_rube(100, 255) =", pykosh.add_via_rube(100, 255))
    print("add_via_rube(12345, 54321) =", pykosh.add_via_rube(12345, 54321))


    # Class
    c = pykosh.Counter(start=10)
    print(f"\n{c}")
    c.increment(5)
    print(f"After increment(5): {c}")
    c.reset()
    print(f"After reset():      {c}")


if __name__ == "__main__":
    main()

