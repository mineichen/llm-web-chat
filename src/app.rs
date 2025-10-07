use futures_util::{StreamExt, TryStreamExt};

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet};
use leptos_router::{
    components::Route,
    components::{Router, Routes},
    *,
};

use crate::ollama;

/// Mock async inference function
async fn call_model(
    input: String,
    model: String,
    set_messages: WriteSignal<Vec<(String, String)>>,
) -> crate::Result<()> {
    let stream = ollama::send_request(input, model)
        .await?
        .map(|x| x.map(|x| x.response));
    let mut stream = std::pin::pin!(stream);
    if let Some(first) = stream.try_next().await? {
        let mut first = first.to_string();
        //first.reserve(reply.len() - first.len());
        first.reserve(1000);
        set_messages.update(|msgs| msgs.push(("AI".to_string(), first)));
        while let Some(word) = stream.try_next().await? {
            if set_messages
                .try_update(|x| {
                    let (_, msg) =
                        &mut x.last_mut().expect("There is always one msg at this point");
                    *msg += word.as_str();
                })
                .is_none()
            {
                break;
            }
        }
    }
    Ok(())
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/leptos_chat_demo.css"/>
        <Router>
            <Routes fallback=move || view! { "Page not found" }>
                <Route path=path!("/") view=move || view! { <ChatPage/> }/>
            </Routes>
        </Router>
    }
}

#[component]
fn ChatPage() -> impl IntoView {
    let models = vec!["qwen3:30B".to_string(), "qwen2.5:14B".to_string()];

    let (selected_model, set_selected_model) = signal(models[0].clone());
    let (model_list, set_model_list) = signal(models);
    let (input, set_input) = signal(String::new());
    let (messages, set_messages) = signal::<Vec<(String, String)>>(vec![]);
    let llm_action = Action::new_local(move |user_input: &String| {
        let user_input = user_input.clone();
        call_model(user_input.clone(), selected_model.get(), set_messages)
    });

    let on_submit = move || {
        if llm_action.pending().get() {
            return;
        }
        let mut user_input = String::new();
        set_input.update(|cur| {
            if !cur.trim().is_empty() {
                user_input = std::mem::take(cur);
            }
        });

        if user_input.is_empty() {
            return;
        }

        set_messages.update(|msgs| msgs.push(("You".to_string(), user_input.clone())));

        llm_action.dispatch_local(user_input);
    };

    let add_next_model = move |_| {
        let new_model = format!("next-model-{}", uuid::Uuid::new_v4());
        set_model_list.update(|list| list.push(new_model.clone()));
        set_selected_model.set(new_model);
    };

    view! {
        <div class="max-w-2xl mx-auto bg-white shadow rounded-2xl p-6">
            <h1 class="text-2xl font-bold mb-4">Leptos Chat Demo</h1>

                    <label class="block text-sm font-medium text-gray-700">Select Model</label>
            <div class="mb-4 flex items-center space-x-2">
                <div class="flex-1">
                    <select
                        class="w-full p-2 border rounded-md"
                        on:change=move |ev| {
                            if let Some(value) = event_target_value(&ev).as_str().to_owned().into() {
                                set_selected_model.set(value);
                            }
                        }
                    >
                    <For
                           each=move || model_list.get()
                           key=|m| m.clone()
                           children=move |m| {
                               let m_clone = m.clone();
                               view! {
                                   <option value=m selected=move || m_clone == selected_model.get()>{m.clone()}</option>
                               }
                           }
                       />
                    </select>
                </div>
                <button
                    class="px-3 py-2 border rounded-md bg-gray-200 hover:bg-gray-300 text-sm"
                    on:click=add_next_model
                    title="Add next model"
                >
                    "↻"
                </button>
            </div>

            <div class="h-64 overflow-y-auto border p-2 mb-4 rounded-md bg-gray-50">
                <For
                    each=move || messages.get()
                    key=|(sender, _)| sender.clone() + &uuid::Uuid::new_v4().to_string()
                    let((sender, text))
                >
                    <div class="mb-2">
                        <span class="font-semibold">{sender}: </span>
                        <span>{text}</span>
                    </div>
                </For>
            </div>
            { move || {
                (!llm_action.pending().get()).then_some(())?;

                match llm_action.value().read().as_ref()? {
                    Ok(_) => None,
                    Err(e) => Some(view! {
                        <div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
                            <p class="font-medium">Error occurred</p>
                            <p class="text-sm mt-1">{e.to_string()}</p>
                        </div>
                    })
                }
            } }
            <div class="flex space-x-2">
                <input
                    class="flex-1 border rounded-md p-2"
                    prop:value=input
                    on:input=move |ev| set_input.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            ev.prevent_default();
                            on_submit();
                        }
                    }

                    placeholder="Type a message..."
                />
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                    on:click=move |_| on_submit()
                >
                { move|| if llm_action.pending().get() { "Pending" } else {"Send"} }
                </button>
            </div>
        </div>
    }
}
