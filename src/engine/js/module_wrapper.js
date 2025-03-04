// 检测是否是 ES 模块
var isESModule = MODULE_CODE.includes('export ') || MODULE_CODE.includes('import ');

if (isESModule) {
    // 对于 ES 模块，我们需要先处理导入语句，然后再执行代码
    try {
        // 预处理代码，替换 import/export 语句
        var processedCode = MODULE_CODE
            // 替换 import { x } from 'y' 语句
            .replace(/import\s+{\s*([^}]+)}\s+from\s+['"]([^'"]+)['"]\s*;?/g, 
                     function(_, imports, path) {
                         return `const { ${imports} } = require('${path}');`;
                     })
            // 替换 import x from 'y' 语句
            .replace(/import\s+([^\s{]+)\s+from\s+['"]([^'"]+)['"]\s*;?/g,
                     function(_, name, path) {
                         return `const ${name} = require('${path}').default || require('${path}');`;
                     })
            // 替换 export const x = y 语句
            .replace(/export\s+const\s+([^\s=]+)\s*=/g, 
                     'module.exports.$1 =')
            // 替换 export function x() 语句
            .replace(/export\s+function\s+([^\s(]+)/g,
                     'module.exports.$1 = function $1')
            // 替换 export default x 语句
            .replace(/export\s+default\s+/g, 
                     'module.exports.default = ');
        
        // 创建模块对象
        var module = { exports: {} };
        
        // 执行转换后的代码
        (new Function('module', 'exports', 'require', '__filename', '__dirname', processedCode))(
            module, 
            module.exports, 
            require, 
            MODULE_PATH, 
            MODULE_DIR
        );
        
        // 将模块导出添加到全局作用域
        for (var key in module.exports) {
            if (module.exports.hasOwnProperty(key)) {
                global[key] = module.exports[key];
            }
        }
        
        // 如果有默认导出，也添加到全局
        if (module.exports.default) {
            if (typeof module.exports.default === 'object') {
                for (var key in module.exports.default) {
                    if (module.exports.default.hasOwnProperty(key)) {
                        global[key] = module.exports.default[key];
                    }
                }
            } else {
                global.default = module.exports.default;
            }
        }
    } catch (e) {
        console.error('Error processing ES module:', e);
        throw e;
    }
} else {
    // 普通 CommonJS 模块
    try {
        var module = { exports: {} };
        (function(exports, require, module, __filename, __dirname) {
            MODULE_CODE
        })(module.exports, require, module, MODULE_PATH, MODULE_DIR);
        
        // 将模块导出添加到全局作用域
        for (var key in module.exports) {
            if (module.exports.hasOwnProperty(key)) {
                global[key] = module.exports[key];
            }
        }
    } catch (e) {
        console.error('Error processing CommonJS module:', e);
        throw e;
    }
} 