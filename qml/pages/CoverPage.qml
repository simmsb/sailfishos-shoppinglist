import QtQuick 2.6
import Sailfish.Silica 1.0
import ShoppingList 1.0

import Opal.Delegates 1.0 as D

import "../components"

CoverBackground {
    id: cover

    property var shoppingLists

    D.DelegateColumn {
        model: shoppingLists ? shoppingLists.items : null

        delegate: D.TwoLineDelegate {
            text: shoppingListName
            description: preview
            descriptionLabel.wrapMode: Text.Wrap
        }
    }
}
