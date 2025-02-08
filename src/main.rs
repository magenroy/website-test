use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! {
    <div class="container">

        <h1>"Welcome to Leptos"</h1>
        <h2><i>"On Github Pages"</i></h2>

    </div>
    }
}

fn main() {
    mount_to_body(|| {
        view! {
            <App />
        }
    })
}
