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
