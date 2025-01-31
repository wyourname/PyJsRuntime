from PyJsRuntime import JsRuntime  # 我们的新库
import execjs
import time
import tracemalloc

# 读取 JS 代码
with open('sig356.js', 'r', encoding='utf-8') as f:
    code = f.read()

# 公共执行函数，计算初始化时间和执行时间，同时监控内存使用
def execute_js(ctx, func_name, *args):
    tracemalloc.start()  # 开始跟踪内存分配

    init_time = time.time()
    ctx_compiled = ctx.compile(code)
    end_init_time = time.time()
    init_time_cost = end_init_time - init_time

    start_time = time.time()
    result = ctx_compiled.call(func_name, *args)
    end_time = time.time()
    execution_time = end_time - start_time

    # 获取当前内存分配的快照
    current, peak = tracemalloc.get_traced_memory()

    tracemalloc.stop()  # 停止内存跟踪

    return init_time_cost, execution_time, result, current, peak

# 测试 pyexecjs 执行时间
def test_pyexecjs_fibonacci(n, func_name='fibonacci'):
    init_time_cost, execution_time, result, current, peak = execute_js(execjs, func_name, n)
    print(f"pyexecjs Initialization Time: {init_time_cost:.6f} seconds")
    print(f"pyexecjs---> {result[:12]}")
    print(f"Execution Time: {execution_time:.6f} seconds; total time {execution_time + init_time_cost:.6f}")
    print(f"Memory Usage: {current / 10**6:.2f} MB; Peak Memory: {peak / 10**6:.2f} MB")

# 测试 PyJsRuntime 执行时间
def test_pyjsruntime_fibonacci(n, func_name='fibonacci'):
    init_time_cost, execution_time, result, current, peak = execute_js(JsRuntime(), func_name, [n])
    print(f"PyJsRuntime Initialization Time: {init_time_cost:.6f} seconds")
    print(f"PyJsRuntime---> {result[:12]}")
    print(f"Execution Time: {execution_time:.6f} seconds; total time {execution_time + init_time_cost:.6f}")
    print(f"Memory Usage: {current / 10**6:.2f} MB; Peak Memory: {peak / 10**6:.2f} MB")

# 性能对比
if __name__ == "__main__":
    # 使用固定的函数名和参数进行测试
    pyexecjs_start_time = time.time()
    test_pyexecjs_fibonacci('/rest/wd/cny2025/warmup/richtree/luckShake/drawsigCatVer=1{"entrySource":"ks_cny_158"}', func_name='getSig3')
    pyexecjs_end_time = time.time()

    pyjsruntime_start_time = time.time()
    test_pyjsruntime_fibonacci('/rest/wd/cny2025/warmup/richtree/luckShake/drawsigCatVer=1{"entrySource":"ks_cny_158"}', func_name='getSig3')
    pyjsruntime_end_time = time.time()

    pyexecjs_total_time = pyexecjs_end_time - pyexecjs_start_time
    pyjsruntime_total_time = pyjsruntime_end_time - pyjsruntime_start_time

    # 计算性能提升比例
    performance_improvement = (pyexecjs_total_time - pyjsruntime_total_time) / pyexecjs_total_time * 100
    print(f"Performance Improvement: {performance_improvement:.2f}%")
