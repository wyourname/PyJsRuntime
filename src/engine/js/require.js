// 确保 global 对象存在
var global = globalThis;

// 添加 ES 模块支持
var importCache = {};

function require(path) {
    // 已加载的模块缓存
    if (!global._modules) global._modules = {};
    
    // 解析路径
    function resolvePath(base, target) {
        if (target.startsWith('./') || target.startsWith('../')) {
            // 相对路径
            let basedir = base.split('/').slice(0, -1).join('/');
            if (basedir) basedir += '/';
            return basedir + target;
        }
        // 绝对路径或模块名
        return target;
    }
    
    // 当前文件路径
    let currentPath = global.__currentPath || '';
    let resolvedPath = resolvePath(currentPath, path);
    
    // 如果模块已加载，直接返回
    if (global._modules[resolvedPath]) {
        return global._modules[resolvedPath].exports;
    }
    
    // 通知 Rust 加载模块
    let moduleCode = _loadModule(resolvedPath);
    if (!moduleCode) {
        throw new Error(`Module not found: ${resolvedPath}`);
    }
    
    // 创建模块对象
    let module = { exports: {} };
    global._modules[resolvedPath] = module;
    
    // 保存当前路径并设置新路径
    let prevPath = global.__currentPath;
    global.__currentPath = resolvedPath;
    
    // 执行模块代码
    try {
        // 检测是否是 ES 模块 (包含 import/export 语句)
        if (moduleCode.includes('export ') || moduleCode.includes('import ')) {
            // 转换 ES 模块为 CommonJS
            let transformedCode = moduleCode
                // 转换 import 语句
                .replace(/import\s+{\s*([^}]+)}\s+from\s+['"]([^'"]+)['"]/g, 
                         function(_, imports, path) {
                             return `const { ${imports} } = require('${path}');`;
                         })
                // 转换默认导入
                .replace(/import\s+([^\s]+)\s+from\s+['"]([^'"]+)['"]/g,
                         function(_, name, path) {
                             return `const ${name} = require('${path}');`;
                         })
                // 转换命名导出
                .replace(/export\s+const\s+([^\s=]+)\s*=/g, 
                         'module.exports.$1 =')
                // 转换默认导出
                .replace(/export\s+default\s+/g, 
                         'module.exports.default = ');
            
            let moduleFunc = new Function('exports', 'require', 'module', '__filename', '__dirname', transformedCode);
            moduleFunc(module.exports, require, module, resolvedPath, resolvedPath.split('/').slice(0, -1).join('/'));
        } else {
            // 普通 CommonJS 模块
            let moduleFunc = new Function('exports', 'require', 'module', '__filename', '__dirname', moduleCode);
            moduleFunc(module.exports, require, module, resolvedPath, resolvedPath.split('/').slice(0, -1).join('/'));
        }
    } finally {
        // 恢复当前路径
        global.__currentPath = prevPath;
    }
    
    return module.exports;
}

// 添加 import 函数支持 ES 模块语法
global.import = function(path) {
    return Promise.resolve(require(path));
};

// 将 require 添加到全局
global.require = require; 