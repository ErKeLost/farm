//dep-8b00.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "dep", function() {
            return dep;
        });
        var _f_dep1 = farmRequire.i(farmRequire("ef0c4c9d"));
        var dep = 'dep';
        exports.default = function() {
            return farmRequire.f(_f_dep1)();
        };
        console.log('side effect in dep.ts');
    },
    "ef0c4c9d": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = function() {
            console.log('1111');
        };
    }
});


//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('dynamic-import.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
initModuleSystem$1(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmDynamicRequire("05ee5ec7").then((dep)=>{
            console.log(dep);
        });
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.si([]);__farm_ms__.sd([{ path: 'dep-8b00.js', type: 0 }],{ '05ee5ec7': [0] });__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");