# import time
# import execjs  # 原始的 pyexecjs
from PyJsRuntime import JsRuntime  # 我们的新库

# def benchmark(name, func, iterations=1000):
#     start_time = time.time()
#     for _ in range(iterations):
#         func()
#     end_time = time.time()
#     duration = end_time - start_time
#     ops_per_sec = iterations / duration
#     print(f"{name}: {duration:.3f} seconds, {ops_per_sec:.2f} ops/sec")

# # 测试数据
# test_cases = [
#     ("Simple Math", "1 + 2 * 3"),
#     # ("Object Operation", "({a:1, b:2, c:3})"),
#     ("Array Operation", "[1,2,3,4,5].map(x => x * 2)"),
#     ("String Operation", "'hello' + ' world'.toUpperCase()"),
#     ("Complex Calculation", """
#         function fib(n) {
#             if (n <= 1) return n;
#             return fib(n-1) + fib(n-2);
#         }
#         fib(10)
#     """)
# ]

# # 初始化两个运行时
# pyexecjs_runtime = execjs.get()
# our_runtime = JsRuntime()

# # 运行比较测试
# for test_name, code in test_cases:
#     print(f"\nTesting: {test_name}")
    
#     # Our Runtime
#     def our_test():
#         our_runtime.execute(code)
#     benchmark("Our Runtime", our_test)
#     # PyExecJS
#     # def pyexecjs_test():
#     #     pyexecjs_runtime.eval(code)
#     # benchmark("PyExecJS", pyexecjs_test)
    
    
#     # 验证结果一致性
#     # result1 = pyexecjs_runtime.eval(code)
#     result2 = our_runtime.execute(code)
#     # print(f"Results match: {result1 == result2}")
#     # print(f"PyExecJS result: {result1}")
#     print(f"Our result: {result2}")

import execjs
import time

# 创建一个 JavaScript 代码执行上下文

with open('test.js', 'r', encoding='utf-8') as f:
    code = f.read()

ctx = execjs.compile(code)

# 测试 Fibonacci 函数的执行时间
def test_pyexecjs_fibonacci(n):
    start_time = time.time()
    # 调用 JavaScript 中的 fibonacci 函数
    result = ctx.call("fibonacci", n)
    end_time = time.time()
    print(f"fibonacci({n}) = {result}")
    print(f"Execution Time: {end_time - start_time:.6f} seconds")


def test_pyjsruntime_fibonacci(n):
    
    # result = JsRuntime().eval(code)
    # result = JsRuntime().eval("new Date('2023-07-20')")
    
    ctx = JsRuntime().compile(code)
    print(type(ctx))
    start_time = time.time()
    result = ctx.call("fibonacci", [n])
    end_time = time.time()
    print(f"result = {result}")
    print(f"Execution Time: {end_time - start_time:.6f} seconds")

# 测试性能
if __name__ == "__main__":
    test_pyexecjs_fibonacci(40)  # 你可以调整数字来测试不同的性能
    test_pyjsruntime_fibonacci(40)  # 你可以调整数字来测试不同的性能
