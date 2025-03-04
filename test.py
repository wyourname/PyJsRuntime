from py_js_runtime import JsRuntime  # 我们的新库
import execjs
import time
import tracemalloc

# # # 读取 JS 代码
with open('sig356.js', 'r', encoding='utf-8') as f:
    code = f.read()

with open('test.js', 'r', encoding='utf-8') as f:
    code2 = f.read()

# 公共执行函数，计算初始化时间和执行时间，同时监控内存使用
def execute_js(ctx, func_name, *args):
    tracemalloc.start()  # 开始跟踪内存分配
    init_time = time.time()
    ctx_compiled = ctx.compile(code)
    end_init_time = time.time()
    init_time_cost = end_init_time - init_time
    start_time = time.time()
    for i in range(1):
        result = ctx_compiled.call(func_name, *args)
    end_time = time.time()
    execution_time = end_time - start_time

    # 获取当前内存分配的快照
    current, peak = tracemalloc.get_traced_memory()

    tracemalloc.stop()  # 停止内存跟踪

    return init_time_cost, execution_time, result, current, peak

def rt_execute_js(ctx, func_name, *args):
    tracemalloc.start()  # 开始跟踪内存分配
    init_time = time.time()
    # ctx_compiled = ctx.compile("./test.js")
    code = """
import { getSig3 } from './sig3_56.js';

function test() {
    console.log('test');
    return getSig3('{"reportCount":1,"subBizId":6428,"taskId":26035}');
}
"""
    ctx_compiled = ctx.compile_code(code)
    end_init_time = time.time()
    init_time_cost = end_init_time - init_time
    start_time = time.time()
    for i in range(1):
        result = ctx_compiled.call_function(func_name, *args)
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
    print(f"pyexecjs---> {result}")
    # print(f"pyexecjs---> {result[:12]}")
    print(f"Execution Time: {execution_time:.6f} seconds; total time {execution_time + init_time_cost:.6f}")
    print(f"Memory Usage: {current / 10**6:.2f} MB; Peak Memory: {peak / 10**6:.2f} MB")

# 测试 PyJsRuntime 执行时间
def test_pyjsruntime_fibonacci(n, func_name='fibonacci'):
    init_time_cost, execution_time, result, current, peak = rt_execute_js(JsRuntime(), func_name, [n])
    print(f"PyJsRuntime Initialization Time: {init_time_cost:.6f} seconds")
    print(f"PyJsRuntime---> {result}")
    # print(f"PyJsRuntime---> {result[:12]}")
    print(f"Execution Time: {execution_time:.6f} seconds; total time {execution_time + init_time_cost:.6f}")
    print(f"Memory Usage: {current / 10**6:.2f} MB; Peak Memory: {peak / 10**6:.2f} MB")


def test_pyjsruntime_eval(code):
    ctx = JsRuntime().compile(code)
    result = ctx.get_property('localStorage')
    print(result)

# 性能对比
if __name__ == "__main__":
    # 使用固定的函数名和参数进行测试
    # pyexecjs_start_time = time.time()
    # test_pyexecjs_fibonacci('/rest/wd/cny2025/warmup/richtree/luckShake/drawsigCatVer=1{"entrySource":"ks_cny_158"}', func_name='getSig3')
    # pyexecjs_end_time = time.time()

    pyjsruntime_start_time = time.time()
    test_pyjsruntime_fibonacci('/rest/wd/cny2025/warmup/richtree/luckShake/drawsigCatVer=1{"entrySource":"ks_cny_158"}', func_name='test')
    pyjsruntime_end_time = time.time()

    # pyexecjs_total_time = pyexecjs_end_time - pyexecjs_start_time
    # pyjsruntime_total_time = pyjsruntime_end_time - pyjsruntime_start_time

    # # 计算性能提升比例
    # performance_improvement = (pyexecjs_total_time - pyjsruntime_total_time) / pyexecjs_total_time * 100
    # print(f"Performance Improvement: {performance_improvement:.2f}%")
    data = r"""
    localStorage = {
    setItem: function(key, value) {
        // 在这里存储数据
        this[key] = value;
    },
    getItem: function(key) {
        // 从这里获取数据
        return this[key] || null;
    },
    removeItem: function(key) {
        // 删除存储的数据
        delete this[key];
    }
};
    localStorage.setItem("a", '1')
"""
    # test_pyjsruntime_eval(data)
# from py_js_runtime import JsRuntime
# import asyncio
# async def test_promise_call():
#     code = """\n
# async function greet() {
#         Deno.core.ops.op_print('Hello from JavaScript!');
#         return 'Greetings from JS';
#     }
# greet().then(result => Deno.core.ops.op_resolve_promise(result));
# """
#     ctx = JsRuntime().call_async(code)
#     print(ctx)
#     ctx = JsRuntime().compile(code2)
#     result = ctx.call_function('getSig3', '{"reportCount":1,"subBizId":6426,"taskId":26021}')
#     print(result)

# # test_promise_call()
# if __name__ == "__main__":
#     asyncio.run(test_promise_call())





