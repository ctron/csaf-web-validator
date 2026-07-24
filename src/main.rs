mod validator;

use csaf::validation::{TestResult, TestResultStatus, ValidationResult};
use leptos::prelude::*;

fn main() {
    browser_panic_hook::set_once_default();
    mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    let json_input = RwSignal::new(String::new());
    let preset = RwSignal::new("full".to_string());
    let result: RwSignal<Option<Result<ValidationResult, String>>> = RwSignal::new(None);

    let on_validate = move |_: leptos::ev::MouseEvent| {
        let r = validator::validate(&json_input.get(), &preset.get());
        result.set(Some(r));
    };

    view! {
        <div class="app">
            <header>
                <h1>"CSAF Web Validator"</h1>
                <p class="subtitle">"Paste a CSAF document and validate it in your browser"</p>
            </header>
            <main>
                <section class="input-section">
                    <label for="csaf-input">"CSAF Document (JSON)"</label>
                    <textarea
                        id="csaf-input"
                        placeholder="Paste your CSAF JSON document here..."
                        prop:value=move || json_input.get()
                        on:input=move |ev| json_input.set(event_target_value(&ev))
                    />
                    <div class="controls">
                        <label for="preset">"Preset"</label>
                        <select
                            id="preset"
                            prop:value=move || preset.get()
                            on:change=move |ev| preset.set(event_target_value(&ev))
                        >
                            <option value="basic">"Basic"</option>
                            <option value="extended">"Extended"</option>
                            <option value="full">"Full"</option>
                        </select>
                        <button class="validate-btn" on:click=on_validate>"Validate"</button>
                    </div>
                </section>

                {move || {
                    result.get().map(|r| match r {
                        Ok(vr) => view! { <ValidationResults result=vr /> }.into_any(),
                        Err(e) => view! {
                            <div class="error-banner">{e}</div>
                        }
                        .into_any(),
                    })
                }}
            </main>
        </div>
    }
}

#[component]
fn ValidationResults(result: ValidationResult) -> impl IntoView {
    let success = result.success;
    let version = result.version.clone();
    let num_errors = result.num_errors;
    let num_warnings = result.num_warnings;
    let num_infos = result.num_infos;

    let banner_class = if success {
        "summary-banner pass"
    } else {
        "summary-banner fail"
    };
    let status_text = if success {
        format!("CSAF {version} \u{2014} Valid")
    } else {
        format!("CSAF {version} \u{2014} Invalid")
    };

    let failed_tests: Vec<TestResult> = result
        .test_results
        .into_iter()
        .filter(|t| matches!(&t.status, TestResultStatus::Failure { .. }))
        .collect();

    view! {
        <div class=banner_class>
            <span class="status">{status_text}</span>
            <div class="counts">
                <span class="count-error">{format!("{num_errors} errors")}</span>
                <span class="count-warning">{format!("{num_warnings} warnings")}</span>
                <span class="count-info">{format!("{num_infos} infos")}</span>
            </div>
        </div>

        <div class="results">
            {failed_tests
                .into_iter()
                .map(|test| view! { <TestResultEntry test_result=test /> })
                .collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn TestResultEntry(test_result: TestResult) -> impl IntoView {
    let TestResultStatus::Failure {
        errors,
        warnings,
        infos,
    } = test_result.status
    else {
        unreachable!()
    };

    let (severity, severity_label) = if !errors.is_empty() {
        ("error", "Error")
    } else if !warnings.is_empty() {
        ("warning", "Warning")
    } else {
        ("info", "Info")
    };

    let all_findings: Vec<_> = errors.into_iter().chain(warnings).chain(infos).collect();
    let count = all_findings.len();
    let badge_class = format!("severity-badge {severity}");
    let count_text = if count == 1 {
        "1 finding".to_string()
    } else {
        format!("{count} findings")
    };

    view! {
        <details class="test-entry" open>
            <summary>
                <span class="test-id">{test_result.test_id}</span>
                <span class=badge_class>{severity_label}</span>
                <span class="finding-count">{count_text}</span>
            </summary>
            <div class="findings">
                {all_findings
                    .into_iter()
                    .map(|f| {
                        view! {
                            <div class="finding">
                                <p class="message">{f.message}</p>
                                <code class="instance-path">{f.instance_path}</code>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </details>
    }
}
