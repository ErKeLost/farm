(function (modules) {
            for (var key in modules) {
              modules[key].__farm_resource_pot__ = 'monitor_index_ec33.js';
                (globalThis || window || global)['8631a49814b6940e6ec3522bbe70b0e5'].__farm_module_system__.register(key, modules[key]);
            }
        })({"380fb846": function(module, exports, farmRequire, farmDynamicRequire) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return StudioStatus;
    }
});
const _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
const _useLocale = /*#__PURE__*/ _interop_require_default._(farmRequire("96146b66"));
const _webreact = farmRequire("050d455e");
const _react = /*#__PURE__*/ _interop_require_default._(farmRequire("a0fc9dfd"));
const _locale = /*#__PURE__*/ _interop_require_default._(farmRequire("84a52c12"));
function StudioStatus() {
    const t = (0, _useLocale.default)(_locale.default);
    const dataStatus = [
        {
            label: _react.default.createElement("span", null, _react.default.createElement(_webreact.Typography.Text, {
                style: {
                    paddingRight: 8
                }
            }, t['monitor.studioStatus.mainstream']), t['monitor.studioStatus.bitRate']),
            value: '6 Mbps'
        },
        {
            label: t['monitor.studioStatus.frameRate'],
            value: '60'
        },
        {
            label: _react.default.createElement("span", null, _react.default.createElement(_webreact.Typography.Text, {
                style: {
                    paddingRight: 8
                }
            }, t['monitor.studioStatus.hotStandby']), t['monitor.studioStatus.bitRate']),
            value: '6 Mbps'
        },
        {
            label: t['monitor.studioStatus.frameRate'],
            value: '60'
        },
        {
            label: _react.default.createElement("span", null, _react.default.createElement(_webreact.Typography.Text, {
                style: {
                    paddingRight: 8
                }
            }, t['monitor.studioStatus.coldStandby']), t['monitor.studioStatus.bitRate']),
            value: '6 Mbps'
        },
        {
            label: t['monitor.studioStatus.frameRate'],
            value: '60'
        }
    ];
    const dataPicture = [
        {
            label: t['monitor.studioStatus.line'],
            value: '热备'
        },
        {
            label: 'CDN',
            value: 'KS'
        },
        {
            label: t['monitor.studioStatus.play'],
            value: 'FLV'
        },
        {
            label: t['monitor.studioStatus.pictureQuality'],
            value: '原画'
        }
    ];
    return _react.default.createElement(_webreact.Card, null, _react.default.createElement(_webreact.Space, {
        align: "start"
    }, _react.default.createElement(_webreact.Typography.Title, {
        style: {
            marginTop: 0,
            marginBottom: 16
        },
        heading: 6
    }, t['monitor.studioStatus.title.studioStatus']), _react.default.createElement(_webreact.Tag, {
        color: "green"
    }, t['monitor.studioStatus.smooth'])), _react.default.createElement(_webreact.Descriptions, {
        colon: ": ",
        layout: "horizontal",
        data: dataStatus,
        column: 2
    }), _react.default.createElement(_webreact.Typography.Title, {
        style: {
            marginBottom: 16
        },
        heading: 6
    }, t['monitor.studioStatus.title.pictureInfo']), _react.default.createElement(_webreact.Descriptions, {
        colon: ": ",
        layout: "horizontal",
        data: dataPicture,
        column: 2
    }));
}

},
"84a52c12": function(module, exports, farmRequire, farmDynamicRequire) {
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
        "menu.dashboard": "Dashboard",
        "monitor.title.chatPanel": "Chat Window",
        "monitor.title.quickOperation": "Quick Operation",
        "monitor.title.studioInfo": "Studio Information",
        "monitor.title.studioPreview": "Studio Preview",
        "monitor.chat.options.all": "All",
        "monitor.chat.placeholder.searchCategory": "Search Category",
        "monitor.chat.update": "Update",
        "monitor.list.title.order": "Order",
        "monitor.list.title.cover": "Cover",
        "monitor.list.title.name": "Name",
        "monitor.list.title.duration": "Duration",
        "monitor.list.title.id": "ID",
        "monitor.list.tip.rotations": "Rotations ",
        "monitor.list.tip.rest": ", The program list is not visible to viewers",
        "monitor.list.tag.auditFailed": "Audit Failed",
        "monitor.tab.title.liveMethod": "Live Method",
        "monitor.tab.title.onlineUsers": "Online Users",
        "monitor.liveMethod.normal": "Normal Live",
        "monitor.liveMethod.flowControl": "Flow Control Live",
        "monitor.liveMethod.video": "Video Live",
        "monitor.liveMethod.web": "Web Live",
        "monitor.editCarousel": "Edit",
        "monitor.startCarousel": "Start",
        "monitor.quickOperation.changeClarity": "Change the Clarity",
        "monitor.quickOperation.switchStream": "Switch Stream",
        "monitor.quickOperation.removeClarity": "Remove the Clarity",
        "monitor.quickOperation.pushFlowGasket": "Push Flow Gasket",
        "monitor.studioInfo.label.studioTitle": "Studio Title",
        "monitor.studioInfo.label.onlineNotification": "Online Notification",
        "monitor.studioInfo.label.studioCategory": "Studio Category",
        "monitor.studioInfo.placeholder.studioTitle": "'s Studio",
        "monitor.studioStatus.title.studioStatus": "Studio Status",
        "monitor.studioStatus.title.pictureInfo": "Picture Information",
        "monitor.studioStatus.smooth": "Smooth",
        "monitor.studioStatus.frameRate": "Frame",
        "monitor.studioStatus.bitRate": "Bit",
        "monitor.studioStatus.mainstream": "Main",
        "monitor.studioStatus.hotStandby": "Hot",
        "monitor.studioStatus.coldStandby": "Cold",
        "monitor.studioStatus.line": "Line",
        "monitor.studioStatus.play": "Format",
        "monitor.studioStatus.pictureQuality": "Quality",
        "monitor.studioPreview.studio": "Studio",
        "monitor.studioPreview.watching": "watching"
    },
    "zh-CN": {
        "menu.dashboard": "仪表盘",
        "menu.dashboard.monitor": "实时监控",
        "monitor.title.chatPanel": "聊天窗口",
        "monitor.title.quickOperation": "快捷操作",
        "monitor.title.studioInfo": "直播信息",
        "monitor.title.studioPreview": "直播预览",
        "monitor.chat.options.all": "全部",
        "monitor.chat.placeholder.searchCategory": "搜索类目",
        "monitor.chat.update": "更新",
        "monitor.list.title.order": "序号",
        "monitor.list.title.cover": "封面",
        "monitor.list.title.name": "名称",
        "monitor.list.title.duration": "视频时长",
        "monitor.list.title.id": "视频Id",
        "monitor.list.tip.rotations": "轮播次数",
        "monitor.list.tip.rest": "，节目单观众不可见",
        "monitor.list.tag.auditFailed": "审核未通过",
        "monitor.tab.title.liveMethod": "直播方式",
        "monitor.tab.title.onlineUsers": "在线人数",
        "monitor.liveMethod.normal": "普通直播",
        "monitor.liveMethod.flowControl": "控流直播",
        "monitor.liveMethod.video": "视频直播",
        "monitor.liveMethod.web": "网页开播",
        "monitor.editCarousel": "编辑轮播",
        "monitor.startCarousel": "开始轮播",
        "monitor.quickOperation.changeClarity": "切换清晰度",
        "monitor.quickOperation.switchStream": "主备流切换",
        "monitor.quickOperation.removeClarity": "摘除清晰度",
        "monitor.quickOperation.pushFlowGasket": "推流垫片",
        "monitor.studioInfo.label.studioTitle": "直播标题",
        "monitor.studioInfo.label.onlineNotification": "上线通知",
        "monitor.studioInfo.label.studioCategory": "直播类目",
        "monitor.studioInfo.placeholder.studioTitle": "的直播间",
        "monitor.studioStatus.title.studioStatus": "直播状态",
        "monitor.studioStatus.title.pictureInfo": "画面信息",
        "monitor.studioStatus.smooth": "流畅",
        "monitor.studioStatus.frameRate": "帧率",
        "monitor.studioStatus.bitRate": "码率",
        "monitor.studioStatus.mainstream": "主流",
        "monitor.studioStatus.hotStandby": "热备",
        "monitor.studioStatus.coldStandby": "冷备",
        "monitor.studioStatus.line": "线路",
        "monitor.studioStatus.play": "播放格式",
        "monitor.studioStatus.pictureQuality": "画质",
        "monitor.studioPreview.studio": "直播间",
        "monitor.studioPreview.watching": "在看"
    }
};
const _default = i18n;

},});