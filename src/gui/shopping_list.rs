use std::pin::Pin;
use std::sync::Arc;

use actor_macro::{active_model, with_executor};
use chrono::{DateTime, Utc};
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use fracindex::Fracindex;
use itertools::Itertools;
use ractor::{ActorRef, call};
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use uuid::Uuid;

use shoppinglist_core::{
    actor::{FlattenCall, tell},
    mesh::{
        shoppinglist::{ShoppingListMsg, store::ShoppingList},
        synced_state::SyncedStateMsg,
    },
    models::{ShoppingListId, ShoppingListItem, ShoppingListItemId, ShoppingListItemIndex},
};

use crate::actor::{ActorWiringModel, Actors, ModelRef};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel;

        include!("sailfishos-shoppinglist/src/actor/registry.cxxqt.h");
        type ActorRegistry = crate::actor::ActorRegistry;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qproperty(*mut ActorRegistry, registry, READ = get_registry, WRITE = set_registry)]
        #[qproperty(bool, ready, READ = get_ready, NOTIFY)]
        #[qproperty(QString, name, READ, NOTIFY)]
        #[qproperty(bool, shared, READ, NOTIFY)]
        #[qproperty(i64, updated_at, cxx_name = "updatedAt", READ, NOTIFY)]
        #[qproperty(*mut ItemListModel, incomplete_items_model, cxx_name = "incompleteItems", READ = get_incomplete_items_model, CONSTANT)]
        #[qproperty(*mut ItemListModel, completed_items_model, cxx_name = "completedItems", READ = get_completed_items_model, CONSTANT)]
        #[qproperty(QString, share_link, cxx_name = "shareLink", READ, NOTIFY)]
        type ShoppingListModel = super::ShoppingListModelRust;

        fn get_registry(self: &ShoppingListModel) -> *mut ActorRegistry;
        unsafe fn set_registry(self: Pin<&mut ShoppingListModel>, registry: *mut ActorRegistry);
        fn get_ready(self: &ShoppingListModel) -> bool;
        fn get_incomplete_items_model(self: &ShoppingListModel) -> *mut ItemListModel;
        fn get_completed_items_model(self: &ShoppingListModel) -> *mut ItemListModel;
    }

    #[qenum(ItemListModel)]
    enum ItemRoles {
        ItemId,
        ItemName,
        Completed,
        CreatedAt,
        CompletedAt,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qproperty(i32, count, READ, NOTIFY)]
        type ItemListModel = super::ItemListModelRust;

        #[cxx_override]
        fn data(self: &ItemListModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &ItemListModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &ItemListModel, parent: &QModelIndex) -> i32;
        #[cxx_override]
        #[cxx_name = "removeRows"]
        fn qt_remove_rows(
            self: Pin<&mut ItemListModel>,
            row: i32,
            count: i32,
            parent: &QModelIndex,
        ) -> bool;
        #[cxx_override]
        #[cxx_name = "moveRows"]
        fn qt_move_rows(
            self: Pin<&mut ItemListModel>,
            source_parent: &QModelIndex,
            source_row: i32,
            count: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;
        // #[cxx_override]
        // #[cxx_name = "move"]
        // fn move_items(self: &ItemListModel, from_index: usize, to_index: usize, _unknown: i32);

        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut ItemListModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut ItemListModel>);
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        fn begin_insert_rows(
            self: Pin<&mut ItemListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endInsertRows"]
        fn end_insert_rows(self: Pin<&mut ItemListModel>);
        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        fn begin_remove_rows(
            self: Pin<&mut ItemListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        fn end_remove_rows(self: Pin<&mut ItemListModel>);
        #[inherit]
        #[cxx_name = "beginMoveRows"]
        fn begin_move_rows(
            self: Pin<&mut ItemListModel>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;
        #[inherit]
        #[cxx_name = "endMoveRows"]
        fn end_move_rows(self: Pin<&mut ItemListModel>);
        #[inherit]
        fn index(self: &ItemListModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut ItemListModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");

        #[cxx_name = "make_unique"]
        fn new_item_list_model() -> UniquePtr<ItemListModel>;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qinvokable]
        fn set_shopping_list_id(self: Pin<&mut ShoppingListModel>, shopping_list_id: &QString);
        #[qinvokable]
        fn add_item(self: Pin<&mut ShoppingListModel>, name: &QString);
        #[qinvokable]
        fn mark_item_completed(
            self: Pin<&mut ShoppingListModel>,
            item_id: &QString,
            completed: bool,
        );
        #[qinvokable]
        fn move_incomplete_item_from_to(
            self: Pin<&mut ShoppingListModel>,
            from_index: i32,
            to_index: i32,
        );
        #[qinvokable]
        fn rename_item(self: Pin<&mut ShoppingListModel>, item_id: &QString, name: &QString);
        #[qinvokable]
        fn delete_item(self: Pin<&mut ShoppingListModel>, item_id: &QString);
        #[qinvokable]
        fn clear_completed(self: Pin<&mut ShoppingListModel>);
        #[qinvokable]
        fn rename_shopping_list(self: Pin<&mut ShoppingListModel>, name: &QString);
        #[qinvokable]
        fn undo_last(self: Pin<&mut ShoppingListModel>);
        #[qinvokable]
        fn share(self: Pin<&mut ShoppingListModel>);
    }
}

pub use qobject::{ItemListModel, ItemRoles, ShoppingListModel};

#[derive(Clone, PartialEq, Eq)]
struct ItemRow {
    item_id: String,
    item_name: String,
    completed: bool,
    created_at: DateTime<chrono::Local>,
    completed_at: Option<DateTime<chrono::Local>>,
}

list_model! {
    model: ItemListModel,
    rust: ItemListModelRust,
    roles: ItemRoles,
    row: ItemRow,
    new: qobject::new_item_list_model,
    owned: OwnedItemList,
    columns: [
        (ItemId, "itemId", item_id),
        (ItemName, "itemName", item_name),
        (Completed, "completed", completed),
        (CreatedAt, "createdAt", created_at),
        (CompletedAt, "completedAt", completed_at),
    ],
}

impl ItemRow {
    fn from_item(item: &ShoppingListItem) -> Self {
        Self {
            item_id: serde_json::to_string(&item.id).unwrap(),
            item_name: item.name.clone(),
            completed: item.completed_at.is_some(),
            created_at: item.created_at.with_timezone(&chrono::Local),
            completed_at: item
                .completed_at
                .map(|time| time.with_timezone(&chrono::Local)),
        }
    }
}

#[active_model(qobject::ShoppingListModel)]
#[derive(Default)]
pub struct ShoppingListModelRust {
    synced: Option<ActorRef<SyncedStateMsg>>,
    shopping_list_actor: Option<ActorRef<ShoppingListMsg>>,

    /// A bit of a funny dance, we can't know the shopping list actor until it's
    /// connected, but Qt is able to call our open() method before that.
    ///
    /// So we stash the shopping list id here to use once we're connected.
    shopping_list_id: Option<ShoppingListId>,

    state: Option<ShoppingList>,

    // When we remove items by deleting them or clearing completed, we set this
    // list to contain the items just removed so that we can restore them.
    last_items_removed: Vec<ShoppingListItem>,

    name: QString,
    shared: bool,
    updated_at: i64,

    incomplete_items: Vec<ShoppingListItem>,

    incomplete_items_model: OwnedItemList,
    completed_items_model: OwnedItemList,

    share_link: QString,
}

fn decode_id<T: serde::de::DeserializeOwned>(id: &QString, what: &str) -> Option<T> {
    match serde_json::from_str(id.to_string().trim()) {
        Ok(id) => Some(id),
        Err(err) => {
            tracing::warn!("Failed to decode {what} id: {err:?}");
            None
        }
    }
}

fn partition_shopping_list_items(
    list: &ShoppingList,
) -> (Vec<ShoppingListItem>, Vec<ShoppingListItem>) {
    let (mut incomplete, completed): (Vec<_>, Vec<_>) =
        list.items.values().cloned().partition_map(|i| {
            if i.completed_at.is_some() {
                itertools::Either::Right(i)
            } else {
                itertools::Either::Left(i)
            }
        });

    incomplete.sort_by_key(|i| i.incomplete_item_order_key());
    let completed = completed
        .into_iter()
        .sorted_by_key(|i| i.completed_item_order_key())
        .collect();

    (incomplete, completed)
}

impl ShoppingListModel {
    fn get_incomplete_items_model(&self) -> *mut ItemListModel {
        self.incomplete_items_model.as_ptr()
    }

    fn get_completed_items_model(&self) -> *mut ItemListModel {
        self.completed_items_model.as_ptr()
    }

    fn shopping_list_actor(&self) -> ActorRef<ShoppingListMsg> {
        self.shopping_list_actor
            .clone()
            .expect("Shopping list actor not initialised for shopping list model")
    }

    #[with_executor]
    fn set_shopping_list_id(self: Pin<&mut Self>, shopping_list_id: &QString) {
        tracing::debug!("Opening shopping list {shopping_list_id}");
        // TODO: we'll want to turn these error cases into a toast or something
        let Some(shopping_list_id) = decode_id(shopping_list_id, "shopping list") else {
            return;
        };
        self.rust_mut().shopping_list_id = Some(shopping_list_id);
    }

    fn refresh(&self) {
        tokio::task::spawn_local(refresh(self.model_ref(), self.shopping_list_actor()));
    }

    fn finish_open(mut self: Pin<&mut Self>, shopping_list_actor: ActorRef<ShoppingListMsg>) {
        tracing::debug!("Shopping list finishing open: {:?}", self.shopping_list_id);

        tokio::task::spawn_local(follow_changes(
            self.model_ref(),
            shopping_list_actor.clone(),
        ));
        self.as_mut().rust_mut().shopping_list_actor = Some(shopping_list_actor);
        self.refresh();
    }

    fn set_state(mut self: Pin<&mut Self>, state: ShoppingList) {
        let (incomplete, complete) = partition_shopping_list_items(&state);

        let incomplete_models: Vec<_> = incomplete.iter().map(ItemRow::from_item).collect();
        let complete_models: Vec<_> = complete.iter().map(ItemRow::from_item).collect();
        let name = QString::from(&state.name);

        {
            let mut this = self.as_mut().rust_mut();
            this.name = name;
            this.state = Some(state);
            this.incomplete_items = incomplete;
            this.incomplete_items_model
                .patch_rows(incomplete_models, |row| row.item_id.clone());
            this.completed_items_model
                .patch_rows(complete_models, |row| row.item_id.clone());
        }
        self.name_changed();
    }

    fn set_share_url(mut self: Pin<&mut Self>, url: String) {
        self.as_mut().rust_mut().share_link = QString::from(&url);
        self.share_link_changed();
    }

    #[with_executor]
    fn add_item(self: Pin<&mut Self>, names: &QString) {
        // NOTE: I'm treating each line in the input as a separate item. The
        // keyboard enter key triggers an item, so the only way to get a newline
        // into the input is to paste.

        let names = names.to_string();
        let names = names
            .trim()
            .lines();

        let mut index_to_place_below = self.incomplete_items.last().map(|item| item.index.index());

        for name in names {
            // I think this is helpful?
            let name = name.trim().trim_prefix("- ").trim_prefix("+ ").trim();
            tracing::debug!("Adding item: {name}");

            let index = index_to_place_below
                .map(|index| {
                    Fracindex::builder()
                        .jitter()
                        .after(&index)
                        .build()
                        .expect("Building fractional index failed, should be impossible")
                })
                .unwrap_or_else(|| Fracindex::default());

            index_to_place_below = Some(index.clone());

            // TODO: Decide when to rebalance the fractional indexes?

            let item = ShoppingListItem {
                id: ShoppingListItemId::new(Uuid::new_v4()),
                index: ShoppingListItemIndex::new(index),
                name: name.to_string(),
                created_at: Utc::now(),
                completed_at: None,
            };

            tell(&self.shopping_list_actor(), "Adding item", move |reply| {
                ShoppingListMsg::AddItem(item, reply)
            });
        }
    }

    #[with_executor]
    fn mark_item_completed(self: Pin<&mut Self>, item_id: &QString, completed: bool) {
        let Some(id) = decode_id(item_id, "shopping item") else {
            return;
        };
        tell(
            &self.shopping_list_actor(),
            "Setting item completion",
            move |reply| ShoppingListMsg::SetCompleted {
                id,
                completed_at: completed.then(Utc::now),
                reply,
            },
        );
    }

    #[with_executor]
    fn move_incomplete_item_from_to(self: Pin<&mut Self>, from_index: i32, to_index: i32) {
        let from_index = from_index as usize;
        let to_index = to_index as usize;
        let Some(item_from_id) = self.incomplete_items.get(from_index).map(|i| i.id) else {
            return;
        };

        let before = if to_index == 0 {
            None
        } else if to_index <= from_index {
            self.incomplete_items.get(to_index - 1)
        } else {
            self.incomplete_items.get(to_index)
        };

        let after = if to_index < from_index {
            self.incomplete_items.get(to_index)
        } else if to_index > from_index {
            self.incomplete_items.get(to_index + 1)
        } else {
            self.incomplete_items.get(from_index + 1)
        };

        let before = before.map(|i| i.index.index());
        let after = after.map(|i| i.index.index());

        let new_index = Fracindex::builder().jitter();
        let new_index = match (before.as_ref(), after.as_ref()) {
            (Some(before), Some(after)) => new_index.between(before, after),
            (Some(before), None) => new_index.after(before),
            (None, Some(after)) => new_index.before(after),
            _ => new_index,
        };
        let new_index = new_index
            .build()
            .expect("Fractional index should only fail if we pass it unsorted items");

        tell(
            &self.shopping_list_actor(),
            "Setting item completion",
            move |reply| ShoppingListMsg::SetIndex {
                id: item_from_id,
                new_index: ShoppingListItemIndex::new(new_index),
                reply,
            },
        );
    }

    #[with_executor]
    fn rename_item(self: Pin<&mut Self>, item_id: &QString, name: &QString) {
        let Some(id) = decode_id(item_id, "shopping item") else {
            return;
        };
        let name = name.to_string();
        tell(&self.shopping_list_actor(), "Renaming item", move |reply| {
            ShoppingListMsg::UpdateItem { id, name, reply }
        });
    }

    #[with_executor]
    fn delete_item(self: Pin<&mut Self>, item_id: &QString) {
        let Some(id) = decode_id(item_id, "shopping item") else {
            return;
        };
        tell(&self.shopping_list_actor(), "Deleting item", move |reply| {
            ShoppingListMsg::DeleteItem(id, reply)
        });

        let Some(item) = self.state.as_ref().and_then(|state| state.items.get(&id)) else {
            return;
        };
        let removed = vec![item.clone()];
        self.rust_mut().last_items_removed = removed;
    }

    #[with_executor]
    fn clear_completed(self: Pin<&mut Self>) {
        tell(
            &self.shopping_list_actor(),
            "Clearing completed items",
            ShoppingListMsg::ClearCompleted,
        );

        let Some(state) = self.state.as_ref() else {
            return;
        };
        let removed = state
            .items
            .values()
            .filter(|i| i.completed_at.is_some())
            .cloned()
            .collect();
        self.rust_mut().last_items_removed = removed;
    }

    #[with_executor]
    fn rename_shopping_list(self: Pin<&mut Self>, name: &QString) {
        let name = name.to_string();
        tell(
            &self.shopping_list_actor(),
            "Renaming shopping list",
            move |reply| ShoppingListMsg::Rename(name, reply),
        );
    }

    #[with_executor]
    fn undo_last(mut self: Pin<&mut Self>) {
        let to_restore = std::mem::take(&mut self.as_mut().rust_mut().last_items_removed);
        tell(
            &self.shopping_list_actor(),
            "Restoring items",
            move |reply| ShoppingListMsg::RestoreItems(to_restore, reply),
        );
    }

    #[with_executor]
    fn share(self: Pin<&mut Self>) {
        let model = self.model_ref();
        let shopping_list_actor = self.shopping_list_actor();

        tokio::task::spawn_local(async move {
            match call!(shopping_list_actor, ShoppingListMsg::Share).flatten_call() {
                Ok(url) => {
                    model.with(|model| {
                        tracing::debug!("Setting share url to: {url:?}");
                        model.set_share_url(url);
                    });
                }
                Err(err) => {
                    tracing::warn!("Failed to share shopping list for some reason: {err:?}")
                }
            }
        });
    }
}

impl ActorWiringModel for ShoppingListModel {
    fn connect_actors(mut self: Pin<&mut Self>, actors: Arc<Actors>) {
        tracing::debug!("Shopping list actor connecting");
        let shopping_list_id = self
            .shopping_list_id
            .expect("setShoppingListId should've been called");
        let synced = actors.synced_state.clone();
        self.as_mut().rust_mut().synced = Some(synced.clone());
        let model = self.model_ref();

        tokio::task::spawn_local(async move {
            let result = call!(synced, |reply| SyncedStateMsg::OpenShoppingList {
                id: shopping_list_id,
                ticket: None,
                reply,
            })
            .flatten_call();

            match result {
                Ok(shopping_list_actor) => {
                    model.with(|model| model.finish_open(shopping_list_actor));
                }
                Err(err) => tracing::warn!("Failed to open shopping list: {err:?}"),
            }
        });
    }
}

async fn follow_changes(
    model: ModelRef<ShoppingListModel>,
    shopping_list_actor: ActorRef<ShoppingListMsg>,
) {
    let changes = match call!(shopping_list_actor, ShoppingListMsg::SubscribeChanges) {
        Ok(changes) => changes,
        Err(err) => {
            tracing::warn!("Failed to subscribe to shopping list changes: {err:?}");
            return;
        }
    };

    let mut events = BroadcastStream::new(changes).filter_map(|x| x.ok());
    while let Some(event) = events.next().await {
        tracing::info!("received shopping list event: {event:?}");

        if model.with(|_| ()).is_none() {
            tracing::debug!("Model got dropped, no longer following changes.");
            return;
        }
        refresh(model.clone(), shopping_list_actor.clone()).await;
    }
}

async fn refresh(
    model: ModelRef<ShoppingListModel>,
    shopping_list_actor: ActorRef<ShoppingListMsg>,
) {
    tracing::debug!("refreshing");

    match call!(shopping_list_actor, ShoppingListMsg::GetState).flatten_call() {
        Ok(state) => {
            model.with(|model| model.set_state(state));
        }
        Err(err) => tracing::warn!("Failed to refresh shopping list for some reason: {err:?}"),
    }
}
