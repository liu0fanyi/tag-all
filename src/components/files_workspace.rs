use leptos::prelude::*;
use crate::components::FileTree;
use crate::models::FileViewItem;
use crate::components::EditTarget;
use crate::app::FilterMode;

#[component]
pub fn FilesWorkspace(
    set_selected_file: WriteSignal<Option<FileViewItem>>,
    set_editing_target: WriteSignal<Option<EditTarget>>,
    selected_tags: ReadSignal<Vec<u32>>,
    filter_mode: ReadSignal<FilterMode>,
) -> impl IntoView {
    view! {
        <div class="files-workspace">
             <FileTree
                workspace_id=2 // Files workspace ID
                set_selected_file=set_selected_file
                set_editing_target=set_editing_target
                selected_tags=selected_tags
                filter_mode=filter_mode
             />

             // Empty state overlay removed as FileTree handles empty states per folder
        </div>
    }
}
