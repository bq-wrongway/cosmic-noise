import sympy as sp
from sympy.utilities.codegen import codegen
from sympy.simplify.cse_main import opt_cse

t = sp.Symbol('t')
x0 = sp.Symbol('x0')
x1 = sp.Symbol('x1')
x2 = sp.Symbol('x2')
x3 = sp.Symbol('x3')
y0 = sp.Symbol('y0')
y1 = sp.Symbol('y1')
y2 = sp.Symbol('y2')
y3 = sp.Symbol('y3')

#x = x0 * (1 - t) ** 3 + 3 * x1 * t * (1 - t) ** 2 + 3 * x2 * t**2 * (1 - t) + x3 * t**3
#y = y0 * (1 - t) ** 3 + 3 * y1 * t * (1 - t) ** 2 + 3 * y2 * t**2 * (1 - t) + y3 * t**3
x = 3 * x1 * t * (1 - t) ** 2 + 3 * x2 * t**2 * (1 - t) + x3 * t**3
y = 3 * y1 * t * (1 - t) ** 2 + 3 * y2 * t**2 * (1 - t) + y3 * t**3
a =  20 * sp.integrate(y * sp.diff(x, t), (t, 0, 1))
xm = 840 * sp.integrate(x * y * sp.diff(x, t), (t, 0, 1))
ym = 420 * sp.integrate(y**2 * sp.diff(x, t), (t, 0, 1))


subs, simplified = sp.cse([a, xm, ym], symbols=sp.numbered_symbols('r'), optimizations='basic')
for var, sub in subs:
    print(var, '=', sub)
print(simplified)
print(codegen(('moment_integrals', simplified), "Rust")[0][1])
xx = sp.integrate(x * sp.diff(x, t), (t, 0, 1))
print(codegen(('xx', xx), "Rust")[0][1])
