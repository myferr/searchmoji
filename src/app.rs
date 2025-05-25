use gloo_timers::callback::Timeout;
use gloo_net::http::Request;
use js_sys::Promise;
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Emoji {
    emoji: String,
    name: String,
    keywords: Vec<String>,
}

#[function_component(App)]
pub fn app() -> Html {
    let emojis = use_state(|| vec![]);
    let search = use_state(|| "".to_string());

    let toast = use_state(|| None::<String>);

    {
        let emojis = emojis.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let fetched: Vec<Emoji> = Request::get("/emojis.json")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                emojis.set(fetched);
            });
            || ()
        });
    }

    let filtered: Vec<Emoji> = (*emojis)
        .iter()
        .filter(|e| {
            let search_lower = search.to_lowercase();
            e.name.to_lowercase().contains(&search_lower)
                || e.keywords.iter().any(|kw| kw.to_lowercase().contains(&search_lower))
        })
        .cloned()
        .collect();

    let on_input = {
        let search = search.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                search.set(input.value());
            }
        })
    };

    let toast_clone = toast.clone();
    let copy_to_clipboard = Callback::from(move |symbol: String| {
        if let Some(window) = window() {
            let clipboard = window.navigator().clipboard();
            let promise: Promise = clipboard.write_text(&symbol);
            let toast = toast_clone.clone();
            spawn_local(async move {
                if let Err(err) = wasm_bindgen_futures::JsFuture::from(promise).await {
                    web_sys::console::log_1(&JsValue::from(format!("❌ Clipboard error: {:?}", err)));
                } else {
                    // Show toast message
                    toast.set(Some("✅ Copied to clipboard!".to_string()));

                    // Hide toast after 2 seconds
                    let toast = toast.clone();
                    Timeout::new(2000, move || {
                        toast.set(None);
                    })
                    .forget();
                }
            });
        }
    });

    html! {
        <div class="max-w-2xl mx-auto p-4 mt-20 font-sans">
            <h1 class="text-3xl font-bold mb-6 text-center">{"Searchmoji 😀"}</h1>
            <p class="text-lg text-gray-700 text-center">
                {"Search for emojis by name or keywords. Click to copy!"}
            </p>
            <p class="text-gray-700 mb-6 text-center">
                {"Total emojis: "}
                <span class="font-bold">{emojis.len().to_string()}</span>
            </p>
            <input
                type="text"
                placeholder="Search emojis..."
                value={(*search).clone()}
                oninput={on_input}
                class="btn w-full mb-4 p-2 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <div class="grid grid-cols-4 sm:grid-cols-6 md:grid-cols-8 gap-4">
                {
                    for filtered.iter().map(|emoji| {
                        let symbol = emoji.emoji.clone();
                        let name = emoji.name.clone();
                        let onclick = {
                            let symbol = symbol.clone();
                            let cb = copy_to_clipboard.clone();
                            Callback::from(move |_| cb.emit(symbol.clone()))
                        };
                        html! {
                            <div
                                {onclick}
                                title={name}
                                class="btn bg-gray-100 hover:bg-gray-200 text-center rounded-lg p-4 cursor-pointer transition duration-200 ease-in-out shadow-2xl"
                            >
                                { &emoji.emoji }
                            </div>
                        }
                    })
                }
            </div>
            <div class="mt-6 flex gap-3 justify-center items-center" id="footer">
                <p class="text-sm text-gray-500 text-center">
                    {"Made with ❤️ by "}
                    <a
                        href="https://github.com/myferr/"
                        class="text-blue-500 hover:text-blue-700">
                        {"myferr"}
                    </a>
                </p>
                
            </div>

    if let Some(message) = &*toast {
        <div
            class="fixed bottom-6 left-1/2 transform -translate-x-1/2 bg-green-500 text-white px-6 py-3 rounded-lg shadow-lg font-semibold select-none"
        >
            { message }
        </div>
    } else {
        <></>
    }
        </div>
    }
}
