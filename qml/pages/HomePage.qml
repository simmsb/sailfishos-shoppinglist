import QtQuick 2.6
import Sailfish.Silica 1.0
import "../components"

Page {
    id: page

    property var shoppingLists
    allowedOrientations: defaultAllowedOrientations

    SilicaListView {
        id: view
        anchors.fill: parent
        model: shoppingLists.items

        header: PageHeader {
            title: Tr.tr("Shopping lists")
        }

        PullDownMenu {
            MenuItem {
                text: Tr.tr("About")
                onClicked: pageStack.push(Qt.resolvedUrl("AboutPage.qml"))
            }
            MenuItem {
                text: Tr.tr("Import list")
                onClicked: page.importShoppingList()
            }
            MenuItem {
                text: Tr.tr("New list")
                onClicked: page.createShoppingList()
            }
        }

        ViewPlaceholder {
            enabled: shoppingLists.items.count === 0
            text: Tr.tr("No shopping lists")
            hintText: Tr.tr("Pull down to create or import a list")
        }

        delegate: ListItem {
            id: row
            contentHeight: content.height + Theme.paddingMedium
            width: view.width

            onClicked: {
                page.openShoppingList(shoppingListId)
            }

            Column {
                id: content
                anchors {
                    left: parent.left
                    right: parent.right
                    leftMargin: Theme.horizontalPageMargin
                    rightMargin: Theme.horizontalPageMargin
                    verticalCenter: parent.verticalCenter
                }
                spacing: Theme.paddingSmall

                // TODO: use an opal delegate here
                Item {
                    width: parent.width
                    height: Math.max(nameLabel.height, timeLabel.height)

                    Label {
                        id: timeLabel
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        visible: text.length > 0
                        text: LiveTimeFormat.formatDate(new Date(updatedAt), Formatter.DurationElapsed)
                        color: row.highlighted ? Theme.secondaryHighlightColor : Theme.secondaryColor
                        font.pixelSize: Theme.fontSizeExtraSmall
                    }

                    Label {
                        id: nameLabel
                        width: timeLabel.visible
                               ? parent.width - timeLabel.width - Theme.paddingMedium
                               : parent.width
                        anchors.verticalCenter: parent.verticalCenter
                        text: shoppingListName.length > 0 ? shoppingListName : Tr.tr("<unnamed list>")
                        color: row.highlighted ? Theme.highlightColor : Theme.primaryColor
                        truncationMode: TruncationMode.Fade
                    }
                }

                Label {
                    width: parent.width
                    visible: preview.length > 0
                    text: preview
                    color: row.highlighted ? Theme.secondaryHighlightColor : Theme.secondaryColor
                    font.pixelSize: Theme.fontSizeSmall
                    truncationMode: TruncationMode.Fade
                }
            }
        }
    }

    RemorsePopup {
        id: remorse
    }

    function deleteList(shoppingListId) {
        var message = Tr.tr("List deleted")
        remorse.execute(message, function() {
            shoppingLists.removeShoppingList(shoppingListId)
        })
    }

    function openShoppingList(shoppingListId) {
        pageStack.push(Qt.resolvedUrl("ShoppingListPage.qml"), {
            "shoppingListId": shoppingListId,
            "homePage": page,
        })
    }

    function createShoppingList() {
        var dialog = pageStack.push(Qt.resolvedUrl("NameDialog.qml"), {
            "titleText": Tr.tr("Set list name"),
            "initialText": "",
            "label": "Name"
        })
        dialog.nameAccepted.connect(function(name) {
            shoppingLists.createShoppingList(name)
        })
    }

    function importShoppingList() {
        var dialog = pageStack.push(Qt.resolvedUrl("ImportDialog.qml"))
        dialog.linkAccepted.connect(function(text) {
            shoppingLists.importLink(text)
        })
    }
}
