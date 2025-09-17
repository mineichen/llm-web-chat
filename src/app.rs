use leptos::{prelude::*, task::spawn_local};
use leptos_meta::*;
use leptos_router::{
    components::Route,
    components::{Router, Routes},
    *,
};

/// Mock async inference function
async fn call_model(_model: String, _input: String) -> String {
    "Yes, Master".to_string()
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
    let models = vec!["gpt-4", "gpt-4o", "gpt-3.5-turbo"];

    let (selected_model, set_selected_model) = signal(models[0].to_string());
    let (input, set_input) = signal(String::new());
    let (messages, set_messages) = signal::<Vec<(String, String)>>(vec![]);

    let on_submit = move || {
        let model = selected_model.get();
        let user_input = input.get();
        if user_input.trim().is_empty() {
            return;
        }

        set_input.set(String::new());
        set_messages.update(|msgs| msgs.push(("You".to_string(), user_input.clone())));

        spawn_local(async move {
            let reply = call_model(model, user_input).await;
            set_messages.update(|msgs| msgs.push(("AI".to_string(), reply)));
        });
    };

    view! {
        <div class="max-w-2xl mx-auto bg-white shadow rounded-2xl p-6">
            <h1 class="text-2xl font-bold mb-4">Leptos Chat Demo</h1>

            <div class="mb-4">
                <label class="block text-sm font-medium text-gray-700 mb-1">Select Model</label>
                <select
                    class="w-full p-2 border rounded-md"
                    on:change=move |ev| {
                        if let Some(value) = event_target_value(&ev).as_str().to_owned().into() {
                            set_selected_model.set(value);
                        }
                    }
                >
                    {models.into_iter().map(|m| view! {
                        <option value=m selected={m == selected_model.get()}>{m}</option>
                    }).collect_view()}
                </select>
            </div>

            <div class="h-64 overflow-y-auto border p-2 mb-4 rounded-md bg-gray-50">
                <For
                    each=move || messages.get()
                    key=|(sender, _)| sender.clone() + &uuid::Uuid::new_v4().to_string()
                    children=move |(sender, text)| {
                        view! {
                            <div class="mb-2">
                                <span class="font-semibold">{sender}: </span>
                                <span>{text}</span>
                            </div>
                        }
                    }
                />
            </div>

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
                    Send
                </button>
            </div>
        </div>
    }
}
