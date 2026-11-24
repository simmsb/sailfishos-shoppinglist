import QtQuick 2.6
import Sailfish.Silica 1.0
import Opal.Delegates 1.0 as D
import Opal.DragDrop 1.0

import ShoppingList 1.0
import "../components"

Page {
    id: root

    property string shoppingListId
    property var homePage
    allowedOrientations: defaultAllowedOrientations

    readonly property string displayName: shoppingListModel.name

    function removeThisShoppingList() {
        var id = shoppingListId
        pageStack.pop(null, PageStackAction.Immediate)
        homePage.deleteList(id)
    }

    function editItem(itemId, itemName) {
        var dialog = pageStack.push(Qt.resolvedUrl("NameDialog.qml"), {
            "titleText": Tr.tr("Editing item"),
            "initialText": itemName,
            "label": "New item value"
        })
        dialog.nameAccepted.connect(function(name) {
            shoppingListModel.renameItem(itemId, name)
        })
    }

    function renameThisShoppingList() {
        var dialog = pageStack.push(Qt.resolvedUrl("NameDialog.qml"), {
            "titleText": Tr.tr("Renaming list"),
            "initialText": shoppingListModel.name,
            "label": "New list name"
        })
        dialog.nameAccepted.connect(function(name) {
            shoppingListModel.renameShoppingList(name)
        })
    }

    ShoppingListModel {
        id: shoppingListModel
        registry: ActorRegistry
        Component.onCompleted: setShoppingListId(root.shoppingListId)
    }

    SilicaFlickable {
        id: flick
        anchors.fill: parent
        contentHeight: Math.max(column.height, height)

        PullDownMenu {
            MenuItem {
                text: Tr.tr("Delete list")
                onClicked: root.removeThisShoppingList()
            }
            MenuItem {
                text: Tr.tr("Clear completed")
                enabled: shoppingListModel.completedItems.count > 0
                onClicked: {
                    shoppingListModel.clearCompleted()
                    undo.show(Tr.tr("Cleared completed items"), function() {}, function() {
                        shoppingListModel.undoLast()
                    })
                }
            }
            MenuItem {
                text: Tr.tr("Share")
                onClicked: pageStack.push(Qt.resolvedUrl("SharePage.qml"), { "shoppingListModel": shoppingListModel })
            }
            MenuItem {
                text: Tr.tr("Rename")
                onClicked: root.renameThisShoppingList()
            }
        }

        Column {
            id: column
            width: parent.width

            PageHeader {
                title: page.displayName
            }

            Label {
                anchors {
                    left: parent.left
                    right: parent.right
                    leftMargin: Theme.horizontalPageMargin
                    rightMargin: Theme.horizontalPageMargin
                }
                visible: shoppingListModel.incompleteItems.count === 0 && shoppingListModel.completedItems.count === 0
                text: Tr.tr("No items yet")
                color: Theme.secondaryColor
                font.pixelSize: Theme.fontSizeSmall
            }

            SilicaListView {
                model: shoppingListModel.incompleteItems
                width: parent.width
                contentHeight: model.count * Theme.itemSizeSmall
                height: contentHeight

                ViewDragHandler {
                    id: incompleteItemsDragHandler
                    listView: parent
                    active: true

                    handleMove: false

                    onItemMoved: {
                        shoppingListModel.moveIncompleteItemFromTo(fromIndex, toIndex)
                    }
                }

                delegate: D.OneLineDelegate {
                    text: itemName

                    leftItem: D.DelegateIconItem {
                        source: "image://theme/icon-s-checkmark"
                        color: Theme.rgba(Theme.primaryColor, 0.2)
                    }

                    menu: ContextMenu {
                        MenuItem {
                            text: Tr.tr("Edit")
                            onClicked: root.editItem(itemId, itemName)
                        }
                        MenuItem {
                            text: Tr.tr("Delete")
                            onClicked: {
                                var id = itemId

                                // need to bind shoppingListModel here so that it's available in the callback
                                var lm = shoppingListModel
                                lm.deleteItem(id)
                                undo.show(Tr.tr("Deleted"), function() {}, function() {
                                    lm.undoLast()
                                })
                            }
                        }
                        MenuLabel {
                            text: Tr.tr("Added: %1").arg(Format.formatDate(createdAt, Format.Timepoint))
                        }
                    }

                    dragHandler: incompleteItemsDragHandler

                    onClicked: {
                        var id = itemId
                        shoppingListModel.markItemCompleted(id, !completed)
                    }
                }
            }

            Item {
                width: parent.width
                height: Math.max(composer.implicitHeight, addButton.height)

                TextField {
                    id: composer
                    width: parent.width - addButton.width
                    placeholderText: Tr.tr("New item")
                    label: Tr.tr("New item")
                    EnterKey.enabled: text.trim().length > 0
                    EnterKey.iconSource: "image://theme/icon-m-add"
                    EnterKey.onClicked: root.addCurrentItem()
                }

                IconButton {
                    id: addButton
                    anchors.right: parent.right
                    anchors.verticalCenter: composer.verticalCenter
                    icon.source: "image://theme/icon-m-add"
                    enabled: composer.text.trim().length > 0
                    onClicked: root.addCurrentItem()
                }
            }

            SectionHeader {
                visible: shoppingListModel.completedItems.count > 0
                text: Tr.tr("Completed")
            }

            SilicaListView {
                model: shoppingListModel.completedItems
                width: parent.width
                contentHeight: model.count * Theme.itemSizeSmall
                height: contentHeight

                delegate: D.OneLineDelegate {
                    text: itemName

                    textLabel.font.strikeout: true

                    leftItem: D.DelegateIconItem {
                        source: "image://theme/icon-s-checkmark"
                        color: Theme.highlightColor
                    }

                    menu: ContextMenu {
                        MenuItem {
                            text: Tr.tr("Edit")
                            onClicked: root.editItem(itemId, itemName)
                        }
                        MenuItem {
                            text: Tr.tr("Delete")
                            onClicked: {
                                var id = itemId

                                // need to bind shoppingListModel here so that it's available in the callback
                                var lm = shoppingListModel
                                lm.deleteItem(id)
                                undo.show(Tr.tr("Deleted"), function() {}, function() {
                                    lm.undoLast()
                                })
                            }
                        }
                        MenuLabel {
                            text: Tr.tr("Added: %1").arg(Format.formatDate(createdAt, Format.Timepoint))
                        }
                        MenuLabel {
                            text: Tr.tr("Completed: %1").arg(Format.formatDate(completedAt, Format.Timepoint))
                        }
                    }

                    onClicked: {
                        var id = itemId
                        shoppingListModel.markItemCompleted(id, !completed)
                    }
                }
            }


            Item {
                width: 1
                height: Theme.paddingLarge
            }
        }

        VerticalScrollDecorator {}
    }

    UndoRemorse {
        id: undo
    }

    function addCurrentItem() {
        var name = composer.text.trim()
        if (!name.length)
            return
        shoppingListModel.addItem(name)
        composer.text = ""
        composer.forceActiveFocus()
    }
}
