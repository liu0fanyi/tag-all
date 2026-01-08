use leptos::prelude::*;
use crate::components::FileTree;
use crate::models::FileViewItem;
use crate::commands;
use leptos::task::spawn_local;

use crate::components::EditTarget;

#[component]
pub fn FilesWorkspace(
    set_selected_file: WriteSignal<Option<FileViewItem>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
) -> impl IntoView {
    view! {
        <div class="files-workspace">
             <FileTree
                workspace_id=2 // Files workspace ID
                set_selected_file=set_selected_file
                set_editing_target=set_editing_target
             />

             // Empty state overlay removed as FileTree handles empty states per folder
        </div>
    }
}
