//index.js:
 import { createRequire } from 'module';var require = createRequire(import.meta.url);(function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
}());import * as __farm_external_module_jquery from "jquery";
global['__farm_default_namespace__'].m.se({
    "jquery": __farm_external_module_jquery
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(global["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_jquery = farmRequire.i(farmRequire('jquery'));
        console.log({
            xxx: farmRequire.f(_f_jquery)
        });
    }
});
var __farm_ms__ = global['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");