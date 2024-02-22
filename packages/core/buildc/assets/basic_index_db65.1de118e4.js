(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'basic_index_db65.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"e9860c88": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
const i18n = {
    "en-US": {
        "menu.profile": "Profile",
        "menu.profile.basic": "Basic Profile",
        "basicProfile.title.form": "Parameter Approval Process Table",
        "basicProfile.steps.commit": "Commit",
        "basicProfile.steps.approval": "Approval",
        "basicProfile.steps.finish": "Finish",
        "basicProfile.title.currentVideo": "Current Video Parameters",
        "basicProfile.title.currentAudio": "Current Audio Parameters",
        "basicProfile.title.originVideo": "Origin Video Parameters",
        "basicProfile.title.originAudio": "Origin Audio Parameters",
        "basicProfile.label.video.mode": "Config Mode",
        "basicProfile.label.video.acquisition.resolution": "Acquisition Resolution",
        "basicProfile.label.video.acquisition.frameRate": "Acquisition Frame Rate",
        "basicProfile.label.video.encoding.resolution": "Encoding Resolution",
        "basicProfile.label.video.encoding.rate.min": "Encoding Min Rate",
        "basicProfile.label.video.encoding.rate.max": "Encoding Max Rate",
        "basicProfile.label.video.encoding.rate.default": "Encoding Default Rate",
        "basicProfile.label.video.encoding.frameRate": "Encoding Frame Rate",
        "basicProfile.label.video.encoding.profile": "Encoding Profile",
        "basicProfile.label.audio.mode": "Config Mode",
        "basicProfile.label.audio.acquisition.channels": "Acquisition Channels",
        "basicProfile.label.audio.encoding.channels": "Encoding Channels",
        "basicProfile.label.audio.encoding.rate": "Encoding Rate",
        "basicProfile.label.audio.encoding.profile": "Encoding Profile",
        "basicProfile.unit.audio.channels": "channels",
        "basicProfile.goBack": "GoBack",
        "basicProfile.cancel": "Cancel Process",
        "basicProfile.adjustment.record": "Parameter adjustment record",
        "basicProfile.adjustment.contentId": "Content number",
        "basicProfile.adjustment.content": "Adjust content",
        "basicProfile.adjustment.status": "Current state",
        "basicProfile.adjustment.updatedTime": "Change the time",
        "basicProfile.adjustment.operation": "Operation",
        "basicProfile.adjustment.success": "passed",
        "basicProfile.adjustment.waiting": "under review",
        "basicProfile.adjustment.operation.view": "view"
    },
    "zh-CN": {
        "menu.profile": "详情页",
        "menu.profile.basic": "基础详情页",
        "basicProfile.title.form": "参数审批流程表",
        "basicProfile.steps.commit": "提交修改",
        "basicProfile.steps.approval": "审批中",
        "basicProfile.steps.finish": "修改完成",
        "basicProfile.title.currentVideo": "现视频参数",
        "basicProfile.title.currentAudio": "现音频参数",
        "basicProfile.title.originVideo": "原视频参数",
        "basicProfile.title.originAudio": "原音频参数",
        "basicProfile.label.video.mode": "配置模式",
        "basicProfile.label.video.acquisition.resolution": "采集分辨率",
        "basicProfile.label.video.acquisition.frameRate": "采集帧率",
        "basicProfile.label.video.encoding.resolution": "编码分辨率",
        "basicProfile.label.video.encoding.rate.min": "编码码率最小值",
        "basicProfile.label.video.encoding.rate.max": "编码码率最大值",
        "basicProfile.label.video.encoding.rate.default": "编码码率默认值",
        "basicProfile.label.video.encoding.frameRate": "编码帧率",
        "basicProfile.label.video.encoding.profile": "编码profile",
        "basicProfile.label.audio.mode": "配置模式",
        "basicProfile.label.audio.acquisition.channels": "采集声道数",
        "basicProfile.label.audio.encoding.channels": "编码声道数",
        "basicProfile.label.audio.encoding.rate": "编码码率",
        "basicProfile.label.audio.encoding.profile": "编码 profile",
        "basicProfile.unit.audio.channels": "声道",
        "basicProfile.goBack": "返回",
        "basicProfile.cancel": "取消流程",
        "basicProfile.adjustment.record": "参数调整记录",
        "basicProfile.adjustment.contentId": "内容编号",
        "basicProfile.adjustment.content": "调整内容",
        "basicProfile.adjustment.status": "当前状态",
        "basicProfile.adjustment.updatedTime": "修改时间",
        "basicProfile.adjustment.operation": "操作",
        "basicProfile.adjustment.success": "已通过",
        "basicProfile.adjustment.waiting": "审核中",
        "basicProfile.adjustment.operation.view": "查看"
    }
};
const _default = i18n;

},});