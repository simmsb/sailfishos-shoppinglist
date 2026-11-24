import QtQuick 2.6
import Sailfish.Silica 1.0
import Nemo.DBus 2.0
import ShoppingList 1.0

import "pages"

ApplicationWindow {
    id: window

    property Item homePageInstance: null
    property var pendingStackAction: null

    allowedOrientations: defaultAllowedOrientations
    initialPage: Component {
        HomePage {
            id: homePage
            shoppingLists: shoppingListsModel

            Component.onCompleted: window.homePageInstance = homePage
        }
    }

    cover: Component {
        CoverPage {
            shoppingLists: shoppingListsModel
        }
    }

    ShoppingListsModel {
        id: shoppingListsModel
        registry: ActorRegistry
        onOpen: window.openShoppingList(shopping_list_id)
        onReadyChanged: window.importLaunchLink()
    }

    Connections {
        target: pageStack
        onBusyChanged: {
            if (!pageStack.busy && window.pendingStackAction) {
                var action = window.pendingStackAction
                window.pendingStackAction = null
                action()
            }
        }
    }

    // This is a bit funky. If the app is launched with a share url, we store
    // the url in the Qt UI to then invoke the same `importLink` method that the
    // dbus interface uses.
    property string launchLink: LaunchUrl

    function importLaunchLink() {
        if (!shoppingListsModel.ready || launchLink.length === 0)
            return
        var link = launchLink
        launchLink = ""
        shoppingListsModel.importLink(link)
    }

    function openShoppingList(shoppingListId) {
        // This can be called when we're in the middle of transitioning from the finished dialog.
        window.maybeDeferredPageStackAction(function() {
            pageStack.push(Qt.resolvedUrl("pages/ShoppingListPage.qml"), {
                "shoppingListId": shoppingListId,
                "homePage": window.homePageInstance
            })
        })
    }

    function maybeDeferredPageStackAction(action) {
        if (pageStack.busy) {
            window.pendingStackAction = action
        } else {
            action()
        }
    }

    DBusAdaptor {
        service: DBusName
        path: DBusPath
        iface: DBusName.replace(/-/g, "_")
        bus: DBus.SessionBus

        function openLink(url) {
            console.log("openLink called with", url)
            window.activate()
            if (url && url.length > 0)
                shoppingListsModel.importLink(url)
        }
    }
}
