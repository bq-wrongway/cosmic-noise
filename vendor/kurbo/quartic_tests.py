def vieta(x1, x2, x3, x4, case):
    a = -(x1 + x2 + x3 + x4)
    b = x1 * (x2 + x3) + x2 * (x3 + x4) + x4 * (x1 + x3)
    c = -x1 * x2 * (x3 + x4) - x3 * x4 * (x1 + x2)
    d = x1 * x2 * x3 * x4
    print(f'case {case}: {a.real}, {b.real}, {c.real}, {d.real}')

vieta(1e7, -1e6, 1 + 1j, 1 - 1j, 6)
vieta(-7, -4, -1e6 + 1e5j, -1e6 - 1e5j, 7)
vieta(1e8, 11, 1e3 + 1j, 1e3 - 1j, 8)
