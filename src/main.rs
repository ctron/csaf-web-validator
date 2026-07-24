mod validator;

use base64::prelude::*;
use csaf::validation::{TestResult, TestResultStatus, ValidationResult};
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use leptos::prelude::*;
use std::io::{Read, Write};
use wasm_bindgen::prelude::*;

fn main() {
    browser_panic_hook::set_once_default();
    mount_to_body(|| view! { <App /> });
}

fn compress_to_url_param(input: &str) -> String {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(input.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    BASE64_URL_SAFE_NO_PAD.encode(&compressed)
}

fn decompress_from_url_param(encoded: &str) -> Option<String> {
    let compressed = BASE64_URL_SAFE_NO_PAD.decode(encoded).ok()?;
    let mut decoder = DeflateDecoder::new(&compressed[..]);
    let mut result = String::new();
    decoder.read_to_string(&mut result).ok()?;
    Some(result)
}

fn build_share_url(json_input: &str, preset: &str) -> Option<String> {
    if json_input.is_empty() {
        return None;
    }
    let window = web_sys::window()?;
    let origin = window.location().origin().ok()?;
    let pathname = window.location().pathname().ok()?;
    let encoded = compress_to_url_param(json_input);
    Some(format!("{origin}{pathname}?preset={preset}&doc={encoded}"))
}

fn update_url(json_input: &str, preset: &str) {
    if let Some(url) = build_share_url(json_input, preset) {
        if let Some(window) = web_sys::window() {
            let _ = window
                .history()
                .ok()
                .and_then(|h| h.replace_state_with_url(&JsValue::NULL, "", Some(&url)).ok());
        }
    }
}

fn load_from_url() -> Option<(String, String)> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    let doc_encoded = params.get("doc")?;
    let doc = decompress_from_url_param(&doc_encoded)?;
    let preset = params.get("preset").unwrap_or_else(|| "full".to_string());
    Some((doc, preset))
}

#[component]
fn App() -> impl IntoView {
    let json_input = RwSignal::new(String::new());
    let preset = RwSignal::new("full".to_string());
    let result: RwSignal<Option<Result<ValidationResult, String>>> = RwSignal::new(None);
    let copied = RwSignal::new(false);

    let do_validate = move || {
        let r = validator::validate(&json_input.get(), &preset.get());
        result.set(Some(r));
        update_url(&json_input.get(), &preset.get());
    };

    let on_validate = move |_: leptos::ev::MouseEvent| {
        do_validate();
    };

    if let Some((doc, p)) = load_from_url() {
        json_input.set(doc);
        preset.set(p);
        do_validate();
    }

    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    let on_pick_file = move |_: leptos::ev::MouseEvent| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    let on_file_change = move |_: leptos::ev::Event| {
        let Some(input) = file_input_ref.get() else {
            return;
        };
        let Some(files) = input.files() else { return };
        let Some(file) = files.get(0) else { return };

        let reader = web_sys::FileReader::new().unwrap();
        let reader_clone = reader.clone();
        let onload = Closure::<dyn Fn()>::new(move || {
            if let Ok(text) = reader_clone.result() {
                if let Some(s) = text.as_string() {
                    json_input.set(s);
                }
            }
        });
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        reader.read_as_text(&file).unwrap();
    };

    let on_share = move |_: leptos::ev::MouseEvent| {
        if let Some(window) = web_sys::window() {
            if let Ok(url) = window.location().href() {
                let clipboard = window.navigator().clipboard();
                let _ = clipboard.write_text(&url);
                copied.set(true);
                set_timeout(
                    move || copied.set(false),
                    std::time::Duration::from_secs(2),
                );
            }
        }
    };

    view! {
        <div class="app">
            <header>
                <h1>"CSAF Web Validator"</h1>
                <p class="subtitle">
                    "Paste a CSAF document and validate it in your browser "
                    <a href="https://github.com/ctron/csaf-web-validator" target="_blank">"GitHub"</a>
                </p>
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
                        <input
                            type="file"
                            accept=".json"
                            style="display:none"
                            node_ref=file_input_ref
                            on:change=on_file_change
                        />
                        <button class="pick-file-btn" on:click=on_pick_file>
                            "Pick file"
                        </button>
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
                        <button
                            class="share-btn"
                            on:click=on_share
                            disabled=move || result.get().is_none()
                        >
                            {move || if copied.get() { "Copied!" } else { "Share" }}
                        </button>
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

    let total = result.test_results.len();
    let mut test_errors = 0usize;
    let mut test_warnings = 0usize;
    let mut test_infos = 0usize;
    let mut test_skipped = 0usize;
    let mut failed_tests = Vec::new();

    for t in result.test_results {
        match &t.status {
            TestResultStatus::Success => {}
            TestResultStatus::Failure {
                errors,
                warnings,
                ..
            } => {
                if !errors.is_empty() {
                    test_errors += 1;
                } else if !warnings.is_empty() {
                    test_warnings += 1;
                } else {
                    test_infos += 1;
                }
                failed_tests.push(t);
            }
            TestResultStatus::Skipped | TestResultStatus::NotFound => {
                test_skipped += 1;
            }
        }
    }

    let test_passed = total - test_errors - test_warnings - test_infos - test_skipped;

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

    view! {
        <div class=banner_class>
            <DonutChart
                total
                passed=test_passed
                errors=test_errors
                warnings=test_warnings
                infos=test_infos
                skipped=test_skipped
            />
            <div class="summary-text">
                <span class="status">{status_text}</span>
                <div class="counts">
                    <span class="count-error">{format!("{num_errors} errors")}</span>
                    <span class="count-warning">{format!("{num_warnings} warnings")}</span>
                    <span class="count-info">{format!("{num_infos} infos")}</span>
                </div>
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
fn DonutChart(
    total: usize,
    passed: usize,
    errors: usize,
    warnings: usize,
    infos: usize,
    skipped: usize,
) -> impl IntoView {
    if total == 0 {
        return view! { <div /> }.into_any();
    }

    let pct = |n: usize| (n as f64 / total as f64) * 100.0;
    let segments: Vec<(&str, f64)> = [
        ("#16a34a", pct(passed)),
        ("#dc3545", pct(errors)),
        ("#f59e0b", pct(warnings)),
        ("#3b82f6", pct(infos)),
        ("#9ca3af", pct(skipped)),
    ]
    .into_iter()
    .filter(|(_, p)| *p > 0.0)
    .collect();

    let mut offset = 25.0; // start at 12 o'clock (SVG circles start at 3 o'clock, 25 = -75%)
    let circles: Vec<_> = segments
        .iter()
        .map(|(color, pct)| {
            let dash = format!("{pct} {}", 100.0 - pct);
            let o = offset;
            offset -= pct;
            (*color, dash, o)
        })
        .collect();

    view! {
        <svg class="donut-chart" viewBox="0 0 36 36">
            <circle cx="18" cy="18" r="15.9155" fill="none" stroke="#e5e7eb" stroke-width="3" />
            {circles
                .into_iter()
                .map(|(color, dash, o)| {
                    view! {
                        <circle
                            cx="18"
                            cy="18"
                            r="15.9155"
                            fill="none"
                            stroke=color
                            stroke-width="3"
                            stroke-dasharray=dash
                            stroke-dashoffset=o.to_string()
                        />
                    }
                })
                .collect::<Vec<_>>()}
            <text x="18" y="18" class="donut-center-text">
                {total.to_string()}
            </text>
        </svg>
    }
    .into_any()
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
