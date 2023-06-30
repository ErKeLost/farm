//index.js:
 var entry = function() {
    var __farm_global_this__ = {
        __FARM_TARGET_ENV__: "browser"
    };
    (function(modules, entryModule) {
        var cache = {};
        function require(id) {
            if (cache[id]) return cache[id].exports;
            var module = {
                id: id,
                exports: {}
            };
            modules[id](module, module.exports, require);
            cache[id] = module;
            return module.exports;
        }
        require(entryModule);
    })({
        "d2214aaa": function(module, exports, farmRequire, dynamicRequire) {
            "use strict";
            console.log("runtime/index.js");
            __farm_global_this__.__farm_module_system__.setPlugins([]);
        }
    }, "d2214aaa");
    (function(modules) {
        for(var key in modules){
            __farm_global_this__.__farm_module_system__.register(key, modules[key]);
        }
    })({
        "05ee5ec7": function(module, exports, farmRequire, dynamicRequire) {
            "use strict";
            Object.defineProperty(exports, "__esModule", {
                value: true
            });
            Object.defineProperty(exports, "a", {
                enumerable: true,
                get: function() {
                    return a;
                }
            });
            const a = "1";
        },
        "b5d64806": function(module, exports, farmRequire, dynamicRequire) {
            "use strict";
            Object.defineProperty(exports, "__esModule", {
                value: true
            });
            var _dep = farmRequire("05ee5ec7");
            console.log(_dep.a);
        }
    });
    var farmModuleSystem = __farm_global_this__.__farm_module_system__;
    farmModuleSystem.bootstrap();
    return farmModuleSystem.require("b5d64806");
}();
