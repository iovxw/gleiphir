import QtQuick 2.8
import QtQuick.Layouts 1.3
import QtQuick.Controls 2.3
import QtGraphicalEffects 1.0
import Qt.labs.platform 1.0

ApplicationWindow {
    id: window
    visible: true
    //: Window title
    title: qsTr("Gleipnir")

    minimumWidth: 1000
    minimumHeight: 500

    function formatBytes(bytes, decimals = 2) {
        if (bytes === 0) return '0 Bytes';

        const k = 1024;
        const dm = decimals < 0 ? 0 : decimals;
        const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB'];

        const i = Math.floor(Math.log(bytes) / Math.log(k));

        return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
    }

    TextMetrics {
        id: defaultFont
        text: "O"
    }

    TabBar {
        id: bar
        width: parent.width
        currentIndex: 0

        TabButton {
            text: qsTr("Monitor")
        }
        TabButton {
            text: qsTr("Firewall")
        }
    }

    StackLayout {
        width: parent.width
        height: parent.height - bar.height
        currentIndex: bar.currentIndex
        anchors.top: bar.bottom

        Item {
            GroupBox {
                id: history
                width: parent.width - traffic.width
                height: parent.height * 0.7
                title: "History"
            }

            Frame {
                id: logs
                width: parent.width - traffic.width
                height: parent.height * 0.3
                anchors.top: history.bottom
                topPadding: 0

                ColumnLayout {
                    anchors.fill: parent

                    RowLayout {
                        Layout.fillWidth: true
                        height: separator.implicitHeight
                        spacing: 0

                        Pane {
                            id: trafficTitle0
                            implicitWidth: 40
                            padding: 0
                            Label {
                                text: "R"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {
                            id: separator
                        }
                        Pane {
                            id: trafficTitle1
                            topPadding: 0
                            bottomPadding: 0
                            Layout.fillWidth: true
                            Label {
                                text: "Program"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {}
                        Pane {
                            id: trafficTitle2
                            implicitWidth: defaultFont.width * 2
                            padding: 0
                            Label {
                                text: "↹"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {}
                        Pane {
                            id: trafficTitle3
                            implicitWidth: defaultFont.width * 15
                            padding: 0
                            Label {
                                text: "Address"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {}
                        Pane {
                            id: trafficTitle4
                            implicitWidth: defaultFont.width * 7
                            padding: 0
                            Label {
                                text: "Protocol"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {}
                        Pane {
                            id: trafficTitle5
                            implicitWidth: defaultFont.width * 10
                            padding: 0
                            Label {
                                text: "Size"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {}
                        Pane {
                            id: trafficTitle6
                            implicitWidth: defaultFont.width * 4
                            padding: 0
                            Label {
                                text: "Rule"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                    }

                    ListView {
                        clip: true
                        Layout.fillHeight: true
                        Layout.fillWidth: true
                        model: ListModel {
                            ListElement {
                                dropped: false
                                input: true
                                exe: "/usr/bin/curl"
                                protocol: "TCP"
                                addr: "1.1.1.1:443"
                                len: 42
                                matched_rule: 1
                            }
                            ListElement {
                                dropped: true
                                input: false
                                exe: "/usr/bin/curl"
                                protocol: "TCP"
                                addr: "1.1.1.1:443"
                                len: 42
                                matched_rule: 2
                            }
                            ListElement {
                                dropped: false
                                input: true
                                exe: "/usr/bin/curl"
                                protocol: "UDPLite"
                                addr: "1.1.1.1:443"
                                len: 42
                                matched_rule: 1
                            }
                        }
                        delegate: Item {
                            width: parent.width
                            height: trafficStatus.height

                            Rectangle {
                                id: trafficStatus
                                width: 40
                                height: 40
                                color: if (model.dropped) { "red" } else { "green" }
                            }
                            Label {
                                x: trafficTitle1.x
                                width: trafficTitle1.width
                                clip: true
                                text: model.exe
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            Label {
                                x: trafficTitle2.x + (trafficTitle2.width - width) / 2
                                text: (model.input ?  "⇤" : "↦")
                                font.pointSize: defaultFont.font.pointSize * 1.5
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            Label {
                                x: trafficTitle3.x
                                text: model.addr
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            Label {
                                x: trafficTitle4.x + (trafficTitle4.width - width) / 2
                                text: model.protocol
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            Label {
                                x: trafficTitle5.x + trafficTitle5.width - width
                                text: formatBytes(model.len)
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            Label {
                                x: trafficTitle6.x
                                text: model.matched_rule
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                    }
                }
            }

            Frame {
                id: traffic
                width: Math.max(parent.width * 0.3, 300)
                height: parent.height
                anchors.left: history.right
                topPadding: 0

                ColumnLayout {
                    anchors.fill: parent

                    RowLayout {
                        Layout.fillWidth: true
                        height: separator.implicitHeight
                        spacing: 0

                        Pane {
                            id: historyTitle0
                            topPadding: 0
                            bottomPadding: 0
                            Layout.fillWidth: true
                            Label {
                                text: "Program"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {
                        }
                        Pane {
                            id: historyTitle1
                            implicitWidth: defaultFont.width * 8
                            padding: 0
                            Label {
                                text: "↑"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                        ToolSeparator {}
                        Pane {
                            id: historyTitle2
                            implicitWidth: defaultFont.width * 8
                            padding: 0
                            Label {
                                text: "↓"
                                font.bold: true
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }
                    }

                    ListView {
                        clip: true
                        Layout.fillHeight: true
                        Layout.fillWidth: true
                        model: ListModel {
                            ListElement {
                                exe: "/usr/bin/curl"
                                sending: 1000000
                                receiving: 90000000
                            }
                            ListElement {
                                exe: "/usr/bin/curl"
                                sending: 123
                                receiving: 500000
                            }
                        }
                        delegate: Item {
                            width: parent.width
                            height: separator.implicitHeight

                            Label {
                                clip: true
                                width: historyTitle0.width
                                text: model.exe
                            }
                            Label {
                                x: historyTitle1.x + historyTitle1.width - width
                                text: formatBytes(model.sending) + "/s"
                            }
                            Label {
                                anchors.right: parent.right
                                text: formatBytes(model.receiving) + "/s"
                            }
                        }
                    }
                }
            }
        }
    }
}
