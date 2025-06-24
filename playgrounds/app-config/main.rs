use serde::Deserialize;

use figment::Figment;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Config {
    email: EmailConfig,
    pod_domain: String,
    dashboard_url: String,
    #[serde(default)]
    other: bool,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct EmailConfig {
    smtp_host: String,
    address: String,
}

fn main() {
    let _merged = Figment::new()
        .merge(("email.smtp_host", "example.org"))
        .merge(("email.address", "prose@other.com"))
        .merge(("pod_domain", "email.example.org"))
        .merge(("dashboard_url", "https://admin.example.org"))
        .extract::<Config>()
        .unwrap();

    let _joined = Figment::new()
        .join(("email.smtp_host", "example.org"))
        .join(("email.address", "prose@other.com"))
        .join(("pod_domain", "email.example.org"))
        .join(("dashboard_url", "https://admin.example.org"))
        .extract::<Config>()
        .unwrap();

    test_dynamic_defaults();
}

/// Sometimes we want dynamic default values, like to provide a default for the
/// Dashboard URL (`https://admin.{pod_domain}`) or for the API email address
/// (`prose@{smtp_host}`). Here is an example of how to do that (using a
/// partial struct) and manual additions.
fn test_dynamic_defaults() {
    #[derive(Deserialize)]
    struct PartialConfig {
        pod_domain: String,
        // NOTE: Not possible, unfortunately.
        // #[serde(rename = "email.smtp_host")]
        // smtp_host: String,
        email: PartialEmailConfig,
    }
    #[derive(Deserialize)]
    struct PartialEmailConfig {
        smtp_host: String,
    }

    let config = Figment::new()
        .merge(("pod_domain", "example.org"))
        .merge(("email.smtp_host", "email.example.org"));

    let PartialConfig {
        pod_domain,
        email: PartialEmailConfig { smtp_host },
    } = config.extract().unwrap();

    _ = Figment::new()
        .merge(config)
        .merge(("email.address", format!("prose@{smtp_host}")))
        .merge(("dashboard_url", format!("https://admin.{pod_domain}")))
        .extract::<Config>()
        .unwrap();
}
